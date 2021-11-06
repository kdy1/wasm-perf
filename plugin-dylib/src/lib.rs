use abi_stable::std_types::{RResult, RStr, RString, RVec};
use anyhow::Context;
use common::{deserialize_ast, serialize_ast};
use plugin::transform;
pub use plugin_dylib_api::*;
use serde::de::DeserializeOwned;
use swc_ecmascript::ast::Program;

#[doc(hidden)]
pub fn invoke_js_plugin<C, F>(
    op: fn(C) -> F,
    config_json: RStr,
    ast: RVec<u8>,
) -> RResult<RVec<u8>, RString>
where
    C: DeserializeOwned,
    F: swc_ecmascript::visit::Fold,
{
    use swc_ecmascript::visit::FoldWith;

    let config = serde_json::from_str(config_json.as_str())
        .context("failed to deserialize config string as json");
    let config: C = match config {
        Ok(v) => v,
        Err(err) => return RResult::RErr(format!("{:?}", err).into()),
    };

    let ast = deserialize_ast(ast.as_slice());
    let ast: Program = match ast {
        Ok(v) => v,
        Err(err) => return RResult::RErr(format!("{:?}", err).into()),
    };

    let mut tr = op(config);

    let ast = ast.fold_with(&mut tr);

    let res = match serialize_ast(&ast) {
        Ok(v) => v,
        Err(err) => {
            return RResult::RErr(
                format!(
                    "failed to serialize swc_ecma_ast::Program as json: {:?}",
                    err
                )
                .into(),
            )
        }
    };

    RResult::ROk(res.into())
}

#[macro_export]
macro_rules! define_js_plugin {
    ($fn_name:ident) => {
        #[abi_stable::export_root_module]
        pub fn swc_library() -> $crate::SwcPluginRef {
            extern "C" fn swc_js_plugin(
                config_json: abi_stable::std_types::RStr,
                ast: abi_stable::std_types::RVec<u8>,
            ) -> abi_stable::std_types::RResult<
                abi_stable::std_types::RVec<u8>,
                abi_stable::std_types::RString,
            > {
                $crate::invoke_js_plugin($fn_name, config_json, ast)
            }
            use abi_stable::prefix_type::PrefixTypeTrait;

            $crate::SwcPlugin {
                process_js: Some(swc_js_plugin),
            }
            .leak_into_prefix()
        }
    };
}

define_js_plugin!(transform);
