use serde::Deserialize;
use swc_common::{errors::HANDLER, BytePos, Span};
use swc_ecmascript::visit::{noop_fold_type, Fold};

pub fn transform(_config: Config) -> impl Fold {
    TestPlugin
}

struct TestPlugin;

impl Fold for TestPlugin {
    noop_fold_type!();

    fn fold_span(&mut self, mut span: Span) -> Span {
        HANDLER.with(|handler| {
            handler.struct_span_err(span, "test").emit();
        });

        span.lo = span.lo + BytePos(1);
        span.hi = span.hi + BytePos(1);

        span
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {}
