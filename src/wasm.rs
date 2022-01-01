use std::path::Path;

use anyhow::{Context, Error};
use swc_ecmascript::ast::Program;
use wasmer::{imports, Instance, Memory, Store, Value};

fn alloc(instance: &Instance, memory: &Memory, bytes: &[u8]) -> Result<isize, Error> {
    // The module is not using any bindgen libraries,
    // so it should export its own alloc function.
    //
    // Get the guest's exported alloc function, and call it with the
    // length of the byte array we are trying to copy.
    // The result is an offset relative to the module's linear memory,
    // which is used to copy the bytes into the module's memory.
    // Then, return the offset.

    let alloc = instance
        .exports
        .get_function("__wbindgen_malloc")
        .expect("expected alloc function not found");

    let alloc_result = alloc.call(&[Value::I32(bytes.len() as _)])?;

    let guest_ptr_offset = match &alloc_result[0] {
        Value::I32(offset) => *offset as _,
        _ => return Err(anyhow::anyhow!("expected i32 result")),
    };
    unsafe {
        let raw = memory.data_ptr().offset(guest_ptr_offset);
        raw.copy_from(bytes.as_ptr(), bytes.len());
    }
    return Ok(guest_ptr_offset);
}

pub fn load(path: &Path) -> Result<Instance, Error> {
    let store = Store::default();

    let cache_path = path.with_file_name("cached.swc-wasm-bytes");

    let module = if cache_path.exists() {
        unsafe { wasmer::Module::deserialize_from_file(&store, &cache_path)? }
    } else {
        let m = wasmer::Module::from_file(&store, path)?;

        m.serialize_to_file(&cache_path)?;

        m
    };

    // let output = Pipe::new();
    // let input = Pipe::new();

    // let mut wasi_env = WasiState::new("Lapce")
    //     .stdin(Box::new(input))
    //     .stdout(Box::new(output))
    //     .finalize()?;

    let import_object = imports! {};

    // let wasi = wasi_env.import_object(&module)?;

    let instance = Instance::new(&module, &import_object)?;

    Ok(instance)
}

pub fn apply_js_plugin(
    instance: &Instance,
    config_json: &str,
    program: &Program,
) -> Result<Program, Error> {
    (|| -> Result<_, Error> {
        let config = bincode::config::Configuration::standard();
        let ast_bincode = bincode::encode_to_vec(program, config).unwrap();

        let ret_ptr = instance
            .exports
            .get_function("__wbindgen_add_to_stack_pointer")?
            .call(&[Value::I32(-16)])?;

        let mem = instance.exports.get_memory("memory")?;

        let ast_ptr = alloc(&instance, &mem, &ast_bincode)?;
        let config_ptr = alloc(&instance, &mem, &config_json.as_bytes())?;

        let plugin_fn = instance.exports.get_function("process")?;

        plugin_fn
            .call(&[
                ret_ptr[0].clone(),
                Value::I32(ast_ptr as _),
                Value::I32(ast_bincode.len() as _),
                Value::I32(config_ptr as _),
                Value::I32(config_json.as_bytes().len() as _),
            ])
            .context("failed to invoke `process`")?;

        // TODO: Actually use the return value

        // FIXME: This is wrong, but I think time will be similar
        let (new, _len): (Program, usize) =
            bincode::decode_from_slice(&ast_bincode[..], config).unwrap();

        Ok(new)
    })()
    .with_context(|| format!("failed to invoke js transform plugin",))
}
