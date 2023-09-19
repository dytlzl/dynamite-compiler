pub mod aarch64;
pub mod ast;
pub mod ctype;
pub mod error;
pub mod func;
pub mod global;
pub mod node;
pub mod token;
pub mod tokenizer;
pub mod trie;
pub mod x86_64;

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
    match target_arch {
        Arch::X86_64 => x86_64::generator::AsmGenerator::new(&builder, &error_printer, target_os)
            .generate()
            .to_string(target_os),
        Arch::Aarch64 => aarch64::generator::AsmGenerator::new(&builder, &error_printer, target_os)
            .generate()
            .to_string(target_os),
    }
}
