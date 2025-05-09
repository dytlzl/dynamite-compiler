use dynamite_compiler::generate;
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
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("", "debug", "print debug info");
    opts.optflag("h", "help", "print this help menu");
    opts.opt(
        "o",
        "output",
        "output format (asm, llvm)",
        "FORMAT",
        getopts::HasArg::Yes,
        getopts::Occur::Optional,
    );
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
    let output_option = matches.opt_str("output").unwrap_or("llvm".to_string());
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
    println!("{}", generate(&code, &output_option, is_debug));
}
