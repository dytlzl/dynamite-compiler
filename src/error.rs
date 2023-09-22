const COLOR_RESET: &str = "\x1b[0m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";

pub fn error(err: &str) {
    eprintln!("{}{}{}", COLOR_RED, err, COLOR_RESET);
    std::process::exit(1);
}

#[derive(Debug)]
pub struct SyntaxError {
    pos: usize,
    msg: &'static str,
}

impl SyntaxError {
    pub fn new(pos: usize, msg: &'static str) -> Self {
        Self { pos, msg }
    }
}

pub trait ErrorLogger {
    fn print_error_position(&self, pos: usize, msg: &str);
    fn print_syntax_error_position(&self, err: SyntaxError);
}

pub struct ErrorPrinter<'a> {
    code: &'a str,
}

impl<'a> ErrorPrinter<'a> {
    pub fn new(code: &'a str) -> Self {
        Self { code }
    }
}

impl ErrorLogger for ErrorPrinter<'_> {
    fn print_syntax_error_position(&self, err: SyntaxError) {
        self.print_error_position(err.pos, err.msg);
    }
    fn print_error_position(&self, pos: usize, msg: &str) {
        let mut start_index = 0;
        let mut line_count = 1;
        for (i, c) in self.code[..pos].char_indices() {
            if c == '\n' {
                line_count += 1;
                start_index = i + c.len_utf8();
            }
        }
        let row_number = format!("{} | ", line_count);
        let mut end_index = self.code.len();
        for (i, c) in self.code[..pos].char_indices() {
            if c == '\n' {
                end_index = pos + i;
                break;
            }
        }
        eprintln!(
            "{}{}{}{}",
            row_number,
            COLOR_CYAN,
            &self.code[start_index..end_index],
            COLOR_RESET
        );
        eprintln!(
            "{}{}^ {}{}",
            " ".repeat((pos - start_index) + row_number.len()),
            COLOR_RED,
            if pos >= self.code.len() {
                "unexpected eof while parsing"
            } else {
                msg
            },
            COLOR_RESET
        );
        std::process::exit(1);
    }
}
