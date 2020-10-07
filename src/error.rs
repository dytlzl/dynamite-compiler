const COLOR_RESET: &str = "\x1b[0m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";


pub fn error(err: &str) {
    eprintln!("{}{}{}", COLOR_RED, err, COLOR_RESET);
    std::process::exit(1);
}

pub fn error_at(code: &str, i: usize, err: &str) {
    eprintln!("{}{}{}", COLOR_CYAN, code, COLOR_RESET);
    eprintln!("{}{}^ {}{}", " ".repeat(i),
              COLOR_RED,
              if i >= code.len() { "unexpected eof while parsing" } else { err },
              COLOR_RESET);
    std::process::exit(1);
}