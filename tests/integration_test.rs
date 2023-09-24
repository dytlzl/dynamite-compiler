use std::{
    fs::{self, remove_file},
    io::Write,
    process::Command,
};

use dynamite_compiler::{Arch, Os};

extern crate dynamite_compiler;

extern crate rand;

use rand::distributions::{Alphanumeric, DistString};

#[test]
fn it_defines_one_global_variable() {
    let code = "int a = 7; int main() { printf(\"%d\", a); return 0; }";
    let got = compile_and_get_stdout(code);
    assert_eq!(got, "7")
}

#[test]
fn it_compiles_simple_c() {
    let code = &fs::read_to_string("./tests/c/simple.c").unwrap();
    let got = compile_and_get_stdout(code);
    assert_eq!(got, "12, 2, 35, 5, 7\n")
}

#[test]
fn it_compiles_character_constant_c() {
    let code = &fs::read_to_string("./tests/c/character_constant.c").unwrap();
    let got = compile_and_get_stdout(code);
    assert_eq!(got, "k, ', \\, \n, %\n\t")
}

#[test]
fn it_compiles_string_c() {
    let code = &fs::read_to_string("./tests/c/string.c").unwrap();
    let got = compile_and_get_stdout(code);
    assert_eq!(got, "a = 777, b = 755, c = 222\n")
}

#[test]
fn it_compiles_expr_c() {
    let code = &fs::read_to_string("./tests/c/expr.c").unwrap();
    let got = compile_and_get_stdout(code);
    got.split("\n")
        .filter(|s| !s.is_empty() && !s.ends_with("OK"))
        .for_each(|s| panic!("assertion failed:\n  {}\n", s));
}

#[test]
fn it_compiles_many_functions_c() {
    let code = &fs::read_to_string("./tests/c/many_functions.c").unwrap();
    let got = compile_and_get_stdout(code);
    got.split("\n")
        .filter(|s| !s.is_empty() && !s.ends_with("OK"))
        .for_each(|s| panic!("assertion failed:\n  {}\n", s));
}

fn compile_and_get_stdout(code: &str) -> String {
    #[cfg(target_os = "linux")]
    let target_os = Os::Linux;
    #[cfg(target_os = "macos")]
    let target_os = Os::MacOS;
    #[cfg(target_arch = "x86_64")]
    let target_arch = Arch::X86_64;
    #[cfg(target_arch = "aarch64")]
    let target_arch = Arch::Aarch64;
    let assembly = dynamite_compiler::run(code, target_arch, target_os, false);
    let mut rng = rand::thread_rng();
    fs::create_dir_all("./tests/temp").unwrap();
    let binary_name = &format!("./tests/temp/{}", Alphanumeric.sample_string(&mut rng, 32));
    let child = Command::new("cc")
        .args(["-x", "assembler", "-o", binary_name, "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_ref()
        .map(|mut stdin| stdin.write_all(assembly.as_bytes()))
        .expect("Failed to open stdin")
        .expect("Failed to write to stdin");
    let cc_output = child.wait_with_output().unwrap();
    println!("{}", String::from_utf8_lossy(&cc_output.stderr));
    let output = Command::new(binary_name).output().unwrap();
    remove_file(binary_name).unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}
