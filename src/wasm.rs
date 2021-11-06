use anyhow::{Context, Error};
use common::{deserialize_ast, serialize_ast};
use std::path::Path;
use swc_ecmascript::ast::Program;
use wasmtime::{Engine, ExternRef, Instance, Store};

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

        let mut store = Store::new(&engine, 4);

        let instance = Instance::new(&mut store, &module, &[])
            .context("failed to create instance of a wasm module")?;

        let ast_ref = ExternRef::new(ast_serde);
        let config_ref = ExternRef::new(config_json.to_string());

        let plugin_fn = instance
            .get_typed_func::<(Option<ExternRef>, Option<ExternRef>), Option<ExternRef>, _>(
                &mut store, "process",
            )?;

        let res = plugin_fn.call(&mut store, (Some(ast_ref), Some(config_ref)))?;
        let res = res.expect("wasm returned none");

        let new = res.data().downcast_ref::<Vec<u8>>().unwrap();

        let new: Program = deserialize_ast(new.as_slice())
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
