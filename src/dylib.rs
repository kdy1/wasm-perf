use abi_stable::{
    library::RootModule,
    std_types::{RResult, RStr},
};
use anyhow::{anyhow, Context, Error};
use common::{deserialize_ast, serialize_ast};
use plugin_dylib_api::SwcPluginRef;
use std::path::Path;
use swc_ecmascript::ast::Program;

pub fn apply_js_plugin(
    path: &Path,
    config_json: &str,
    program: &Program,
) -> Result<Program, Error> {
    (|| -> Result<_, Error> {
        let plugin = SwcPluginRef::load_from_file(path).context("failed to load plugin")?;

        let ast_serde = serialize_ast(&program).context("failed to serialize ast")?;

        let plugin_fn = plugin
            .process_js()
            .ok_or_else(|| anyhow!("the plugin does not support transforming js"))?;

        let new_ast = plugin_fn(RStr::from(config_json), ast_serde.into());

        let new = match new_ast {
            RResult::ROk(v) => v,
            RResult::RErr(err) => return Err(anyhow!("plugin returned an error\n{}", err)),
        };
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
