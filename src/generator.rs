use crate::{aarch64, ast::ProgramAst, error::ErrorPrinter, x86_64, Arch, Os};

pub trait Assembly {
    fn to_string(&self, target_os: Os) -> String;
}

pub trait Generator {
    fn generate(&self, ast: ProgramAst) -> Box<dyn Assembly>;
}

pub fn new<'a>(
    target_arch: Arch,
    error_printer: &'a ErrorPrinter,
    target_os: Os,
) -> Box<dyn Generator + 'a> {
    match target_arch {
        Arch::Aarch64 => Box::new(aarch64::generator::AsmGenerator::new(
            error_printer,
            target_os,
        )),
        Arch::X86_64 => Box::new(x86_64::generator::AsmGenerator::new(
            error_printer,
            target_os,
        )),
    }
}
