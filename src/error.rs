const COLOR_RESET: &str = "\x1b[0m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_CYAN: &str = "\x1b[36m";


pub fn error(err: &str) {
    eprintln!("{}{}{}", COLOR_RED, err, COLOR_RESET);
    std::process::exit(1);
}

pub fn error_at(code: &str, pos: usize, err: &str) {
    let mut start_index = 0;
    let mut line_count = 1;
    for (i, c) in (&code[..pos]).char_indices().into_iter() {
        if c == '\n' {
            line_count += 1;
            start_index = i+c.len_utf8();
        }
    }
    let row_number = format!("{} | ", line_count);
    let mut end_index = code.len();
    for (i, c) in (&code[pos..]).char_indices().into_iter() {
        if c == '\n' {
            end_index = pos+i;
            break
        }
    }
    eprintln!("{}{}{}{}", row_number, COLOR_CYAN, &code[start_index..end_index], COLOR_RESET);
    eprintln!("{}{}^ {}{}", " ".repeat((pos-start_index)+row_number.len()),
              COLOR_RED,
              if pos >= code.len() { "unexpected eof while parsing" } else { err },
              COLOR_RESET);
    std::process::exit(1);
}