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
    fn line_from_position(&self, pos: usize) -> (usize, usize, &str);
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
    fn line_from_position(&self, pos: usize) -> (usize, usize, &str) {
        let (line_count, start_index) = self.code[..pos]
            .char_indices()
            .filter(|(_, c)| *c == '\n')
            .fold((1, 0), |(line_count, _), (i, c)| {
                (line_count + 1, i + c.len_utf8())
            });
        let end_index = self.code[pos..]
            .char_indices()
            .find(|(_, c)| *c == '\n')
            .map(|(i, _)| pos + i)
            .unwrap_or(self.code.len());
        (
            line_count,
            pos - start_index,
            &self.code[start_index..end_index],
        )
    }
    fn print_error_position(&self, pos: usize, msg: &str) {
        let (row, col, line) = self.line_from_position(pos);
        let row_number = format!("{} | ", row);
        eprintln!("{}{}{}{}", row_number, COLOR_CYAN, line, COLOR_RESET);
        eprintln!(
            "{}{}^ {}{}",
            " ".repeat(col + row_number.len()),
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
#[derive(Default)]
pub struct NopLogger {}

impl ErrorLogger for NopLogger {
    fn print_syntax_error_position(&self, _: SyntaxError) {}
    fn print_error_position(&self, _: usize, _: &str) {}
    fn line_from_position(&self, _: usize) -> (usize, usize, &str) {
        (0, 0, "")
    }
}
