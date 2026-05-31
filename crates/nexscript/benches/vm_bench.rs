use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nexscript::compiler::Compiler;
use nexscript::lexer::Lexer;
use nexscript::parser::Parser;
use nexscript::vm::Vm;

fn bench_vm_arithmetic(c: &mut Criterion) {
    c.bench_function("vm_arithmetic_1k_ops", |b| {
        b.iter(|| {
            let source = black_box("let x = 0; while x < 1000 { x = x + 1; } x;");
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            let ast = parser.parse().unwrap();
            let mut compiler = Compiler::new();
            let (bytecode, string_pool) = compiler.compile(&ast).unwrap();
            let mut vm = Vm::new(bytecode, string_pool);
            vm.run().unwrap();
        });
    });
}

fn bench_vm_empty(c: &mut Criterion) {
    c.bench_function("vm_empty_program", |b| {
        b.iter(|| {
            let source = black_box("");
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            let ast = parser.parse().unwrap();
            let mut compiler = Compiler::new();
            let (bytecode, string_pool) = compiler.compile(&ast).unwrap();
            let mut vm = Vm::new(bytecode, string_pool);
            vm.run().unwrap();
        });
    });
}

fn bench_vm_if_else(c: &mut Criterion) {
    c.bench_function("vm_if_else_chain", |b| {
        b.iter(|| {
            let source = black_box("let x = 5; if x > 3 { x; } else { 0; }");
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            let ast = parser.parse().unwrap();
            let mut compiler = Compiler::new();
            let (bytecode, string_pool) = compiler.compile(&ast).unwrap();
            let mut vm = Vm::new(bytecode, string_pool);
            vm.run().unwrap();
        });
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    c.bench_function("full_pipeline_lex_parse_compile_run", |b| {
        b.iter(|| {
            let source = black_box("let x = 10; let y = 20; let z = x + y * 2;");
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            let ast = parser.parse().unwrap();
            let mut compiler = Compiler::new();
            let (bytecode, string_pool) = compiler.compile(&ast).unwrap();
            let mut vm = Vm::new(bytecode, string_pool);
            vm.run().unwrap();
        });
    });
}

criterion_group!(benches, bench_vm_arithmetic, bench_vm_empty, bench_vm_if_else, bench_full_pipeline);
criterion_main!(benches);
