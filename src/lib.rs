pub mod assembly;
pub mod ast;
pub mod ctype;
pub mod error;
pub mod func;
pub mod generator;
pub mod global;
pub mod instruction;
pub mod node;
pub mod token;
pub mod tokenizer;
pub mod trie;

pub fn run(code: &str, target_os: generator::Os, is_debug: bool) -> String {
    let error_printer = error::ErrorPrinter::new(code);
    let tokens = tokenizer::Tokenizer::tokenize(code).unwrap_or_else(|e| {
        error::ErrorLogger::print_syntax_error_position(&error_printer, e);
        std::process::exit(1)
    });
    if is_debug {
        tokenizer::Tokenizer::print_tokens(&tokens);
    }
    let mut builder = ast::AstBuilderImpl::new(&error_printer, &tokens);
    builder.build();
    if is_debug {
        builder.print_functions();
    }
    let mut generator = generator::AsmGenerator::new(&builder, &error_printer, target_os);
    generator.gen();
    generator.generate_string()
}
