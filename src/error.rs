pub fn error(err: &str) {
    eprintln!("{}", err);
    std::process::exit(1);
}

pub fn error_at(code: &str, i: usize, err: &str) {
    eprintln!("{}", code);
    eprintln!("{}^ {}", " ".repeat(i), if i >= code.len() { "unexpected eof while parsing" } else { err });
    std::process::exit(1);
}