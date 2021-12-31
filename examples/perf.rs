#![feature(bench_black_box)]

use std::{env, hint::black_box, path::Path, time::Instant};
use swc_common::input::SourceFileInput;
use swc_ecmascript::{
    ast::{EsVersion, Program},
    parser::{lexer::Lexer, Parser},
};
use wasm_perf::{dylib, wasm};

fn input() -> Program {
    testing::run_test(false, |cm, _handler| {
        let fm = cm.load_file(Path::new("benches/input.js")).unwrap();

        let lexer = Lexer::new(
            Default::default(),
            EsVersion::latest(),
            SourceFileInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);

        let program = parser.parse_program().unwrap();

        Ok(program)
    })
    .unwrap()
}

fn main() {
    let program = input();

    let now = Instant::now();

    if env::var("WASM").unwrap_or_default() == "1" {
        let instance = wasm::load(Path::new("plugin-wasm/pkg/plugin_wasm_bg.wasm")).unwrap();

        for _ in 0..100 {
            let new = wasm::apply_js_plugin(&instance, "{}", &program).unwrap();

            black_box(new);
        }
    } else {
        for _ in 0..100 {
            let new = dylib::apply_js_plugin(
                Path::new("target/release/libplugin_dylib.dylib"),
                "{}",
                &program,
            )
            .unwrap();

            black_box(new);
        }
    }

    eprintln!("Done in {:?}", now.elapsed());
}
