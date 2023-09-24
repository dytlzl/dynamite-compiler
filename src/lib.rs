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

use ast::AstBuilder;

pub fn run(
    code: &str,
    target_arch: generator::Arch,
    target_os: generator::Os,
    is_debug: bool,
) -> String {
    let error_printer = error::ErrorPrinter::new(code);
    let tokens = tokenizer::Tokenizer::tokenize(code, is_debug).unwrap_or_else(|e| {
        error::ErrorLogger::print_syntax_error_position(&error_printer, e);
        std::process::exit(1)
    });
    let mut builder = ast::AstBuilderImpl::new(&error_printer, &tokens);
    let generator = generator::new(target_arch, &error_printer, target_os);
    generator
        .generate(builder.build(is_debug))
        .to_string(target_os)
}
