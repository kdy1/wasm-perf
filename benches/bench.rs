#![feature(bench_black_box)]
#![feature(test)]

extern crate test;

use common::{deserialize_ast, serialize_ast};
use std::{hint::black_box, path::Path};
use swc_common::input::SourceFileInput;
use swc_ecmascript::{
    ast::{EsVersion, Program},
    parser::{lexer::Lexer, Parser},
};
use test::Bencher;
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

/// Benchmark for time used by serde
#[bench]
fn serde(b: &mut Bencher) {
    let program = input();
    b.iter(|| {
        let s = serialize_ast(&program).unwrap();

        let s = black_box(s);

        let new: Program = deserialize_ast(&s).unwrap();

        black_box(new);
    })
}

#[bench]
fn dylib(b: &mut Bencher) {
    let program = input();
    b.iter(|| {
        let new = dylib::apply_js_plugin(
            Path::new("target/release/libplugin_dylib.dylib"),
            "{}",
            &program,
        )
        .unwrap();

        black_box(new);
    })
}

#[bench]
fn wasm(b: &mut Bencher) {
    let program = input();
    b.iter(|| {
        let new = wasm::apply_js_plugin(
            Path::new("plugin-wasm/pkg/plugin_wasm_bg.wasm"),
            "{}",
            &program,
        )
        .unwrap();

        black_box(new);
    })
}
