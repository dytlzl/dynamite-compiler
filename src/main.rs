use dynamite_compiler::{generator::Os, tokenizer::Tokenizer};
use dynamite_compiler::ast::AstBuilder;
use dynamite_compiler::generator::AsmGenerator;
use std::fs::File;
use std::io::Read;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        std::process::exit(1)
    }
    let is_debug: bool = &args[1][..] == "debug";
    if is_debug {
        if args.len() < 3 {
            std::process::exit(1)
        }
    }
    let mut code = if is_debug { args[2].clone() } else { args[1].clone() };
    if let Ok(mut f) = File::open(&code) {
        code.clear();
        f.read_to_string(&mut code)
            .expect("something went wrong reading the file");
    }
    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(&code);
    if is_debug {
        eprintln!("[tokens]");
        tokenizer.print_tokens();
    }
    let mut builder = AstBuilder::new(&code, &tokenizer.tokens);
    let node_stream = builder.stream();
    if is_debug {
        eprintln!("[ast]");
        for node in &node_stream {
            eprintln!("{}", node.format());
        }
        eprintln!("[asm]");
    }
    #[cfg(target_os = "linux")]
        let target_os = Os::Linux;
    #[cfg(target_os = "macos")]
        let target_os = Os::MacOS;
    let mut generator = AsmGenerator::new(
        &builder, &code, &node_stream, target_os);
    generator.gen().unwrap();
    let asm = String::from_utf8(generator.buf).unwrap();
    print!("{}", &asm);
}
