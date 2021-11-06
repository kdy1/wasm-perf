use anyhow::{Context, Error};
use common::{deserialize_ast, serialize_ast};
use std::path::Path;
use swc_ecmascript::ast::Program;
use wasmtime::{Engine, Instance, Memory, Store};

fn alloc(
    instance: &Instance,
    store: &mut Store<()>,
    memory: &Memory,
    bytes: &[u8],
) -> Result<isize, Error> {
    // The module is not using any bindgen libraries,
    // so it should export its own alloc function.
    //
    // Get the guest's exported alloc function, and call it with the
    // length of the byte array we are trying to copy.
    // The result is an offset relative to the module's linear memory,
    // which is used to copy the bytes into the module's memory.
    // Then, return the offset.
    let alloc = instance
        .get_typed_func::<u32, u32, _>(&mut *store, "__wbindgen_malloc")
        .expect("expected alloc function not found");
    let alloc_result = alloc.call(&mut *store, bytes.len() as _)?;

    let guest_ptr_offset = alloc_result as isize;
    unsafe {
        let raw = memory.data_ptr(&*store).offset(guest_ptr_offset);
        raw.copy_from(bytes.as_ptr(), bytes.len());
    }
    return Ok(guest_ptr_offset);
}

pub fn apply_js_plugin(
    path: &Path,
    config_json: &str,
    program: &Program,
) -> Result<Program, Error> {
    (|| -> Result<_, Error> {
        let engine = Engine::default();
        let module =
            wasmtime::Module::from_file(&engine, path).context("failed to load wasm file")?;

        let ast_serde = serialize_ast(&program).context("failed to serialize ast")?;

        let mut store = Store::new(&engine, ());

        let instance = Instance::new(&mut store, &module, &[])
            .context("failed to create instance of a wasm module")?;

        let ret_ptr = instance
            .get_typed_func::<i32, i32, _>(&mut store, "__wbindgen_add_to_stack_pointer")?
            .call(&mut store, -16)?;

        let mem = instance.get_memory(&mut store, "memory").unwrap();
        let ast_ptr = alloc(&instance, &mut store, &mem, &ast_serde)?;
        let config_ptr = alloc(&instance, &mut store, &mem, &config_json.as_bytes())?;

        let plugin_fn =
            instance.get_typed_func::<(i32, i32, i32, i32, i32), (), _>(&mut store, "process")?;

        plugin_fn
            .call(
                &mut store,
                (
                    ret_ptr,
                    ast_ptr as _,
                    ast_serde.len() as _,
                    config_ptr as _,
                    config_json.as_bytes().len() as _,
                ),
            )
            .context("failed to invoke `process`")?;

        // TODO: Actually use the return value

        // FIXME: This is wrong, but I think time will be similar
        let new: Program = deserialize_ast(ast_serde.as_slice())
            .with_context(|| format!("plugin generated invalid ast`"))?;

        Ok(new)
    })()
    .with_context(|| {
        format!(
            "failed to invoke `{}` as js transform plugin",
            path.display()
        )
    })
}
