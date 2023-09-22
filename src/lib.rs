pub mod aarch64;
pub mod ast;
pub mod ctype;
pub mod error;
pub mod func;
pub mod generator;
pub mod global;
pub mod node;
pub mod token;
pub mod tokenizer;
pub mod trie;
pub mod x86_64;

use ast::AstBuilder;

#[derive(Clone, Copy)]
pub enum Os {
    Linux,
    MacOS,
}

pub enum Arch {
    Aarch64,
    X86_64,
}

pub fn run(code: &str, target_arch: Arch, target_os: Os, is_debug: bool) -> String {
    let error_printer = error::ErrorPrinter::new(code);
    let tokens = tokenizer::Tokenizer::tokenize(code, is_debug).unwrap_or_else(|e| {
        error::ErrorLogger::print_syntax_error_position(&error_printer, e);
        std::process::exit(1)
    });
    let mut builder = ast::AstBuilderImpl::new(&error_printer, &tokens);
    let mut generator = generator::new(target_arch, &error_printer, target_os);
    generator
        .generate(builder.build(is_debug))
        .to_string(target_os)
}
