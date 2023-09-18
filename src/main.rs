use dynamite_compiler::ast::AstBuilderImpl;
use dynamite_compiler::error::{ErrorLogger, ErrorPrinter};
use dynamite_compiler::generator::AsmGenerator;
use dynamite_compiler::{generator::Os, tokenizer::Tokenizer};
use getopts::Options;
use std::env;
use std::fs::File;
use std::io::Read;
extern crate getopts;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    #[cfg(target_os = "linux")]
    let target_os = Os::Linux;
    #[cfg(target_os = "macos")]
    let target_os = Os::MacOS;
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("", "debug", "print debug info");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if matches.opt_present("help") {
        print_usage(&program, opts);
        return;
    }
    let is_debug = matches.opt_present("debug");
    let path = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        std::process::exit(1)
    };
    let mut code = String::new();
    File::open(&path)
        .unwrap_or_else(|e| panic!("file \"{}\" not found: {}", path, e))
        .read_to_string(&mut code)
        .unwrap_or_else(|e| panic!("failed to read file \"{}\": {}", path, e));
    let error_printer = ErrorPrinter::new(&code);
    let tokens = Tokenizer::tokenize(&code).unwrap_or_else(|e| {
        error_printer.print_syntax_error_position(e);
        std::process::exit(1)
    });
    if is_debug {
        Tokenizer::print_tokens(&tokens);
    }
    let mut builder = AstBuilderImpl::new(&error_printer, &tokens);
    builder.build();
    if is_debug {
        builder.print_functions();
    }
    let mut generator = AsmGenerator::new(&builder, &error_printer, target_os);
    generator.gen();
    if is_debug {
        return;
    }
    generator.print()
}
