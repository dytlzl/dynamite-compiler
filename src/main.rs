use dynamite_compiler::{generator::Os, tokenizer::Tokenizer};
use dynamite_compiler::ast::AstBuilder;
use dynamite_compiler::generator::AsmGenerator;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        std::process::exit(1)
    }
    let is_debug: bool = &args[1][..] == "--debug";
    if is_debug {
        if args.len() < 3 {
            std::process::exit(1)
        }
    }
    let code: &str = if is_debug { &args[2][..] } else { &args[1][..] };
    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(code);
    if is_debug {
        eprintln!("[tokens]");
        tokenizer.print_tokens();
    }
    let mut builder = AstBuilder::new(code, &tokenizer.tokens);
    let node_stream = builder.stream();
    if is_debug {
        eprintln!("[ast]");
        for node in &node_stream {
            eprintln!("{}", node.format());
        }
        eprintln!("[asm]");
    }
    let mut generator = AsmGenerator::new(code, &node_stream, Os::MacOS);
    generator.gen_asm(builder.offset_size).unwrap();
    print!("{}", String::from_utf8(generator.buf).unwrap());
}
