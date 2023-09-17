use dynamite_compiler::{generator::Os, tokenizer::Tokenizer};
use dynamite_compiler::ast::ASTBuilder;
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
    let tokens = Tokenizer::tokenize(&code);
    if is_debug {
        Tokenizer::print_tokens(&tokens);
    }
    let mut builder = ASTBuilder::new(&code, &tokens);
    builder.build();
    if is_debug {
        builder.print_functions();
    }
    #[cfg(target_os = "linux")]
        let target_os = Os::Linux;
    #[cfg(target_os = "macos")]
        let target_os = Os::MacOS;
    let mut generator = AsmGenerator::new(
        &builder, &code, target_os);
    generator.gen();
    if is_debug {
        return;
    }
    if let Os::MacOS = target_os {
        generator.assemblies.iter().for_each(|ass| println!("{}", ass.to_string()));
    } else {
        generator.assemblies.iter().for_each(|ass| println!("{}", ass.to_string4linux()));
    }
}
