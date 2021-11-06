use swc_common::{BytePos, Span};
use swc_ecmascript::visit::{noop_fold_type, Fold};

pub fn transform() -> impl Fold {
    TestPlugin
}

struct TestPlugin;

impl Fold for TestPlugin {
    noop_fold_type!();

    fn fold_span(&mut self, mut span: Span) -> Span {
        span.lo = span.lo + BytePos(1);
        span.hi = span.hi + BytePos(1);
        span
    }
}
