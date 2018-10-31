// build.rs

extern crate nasm_rs;

fn main() {
    nasm_rs::compile_library_args("libboot.a", &["src/header.asm", "src/boot32.asm"], &["-f", "elf64"]);
    println!("cargo:rustc-link-lib=static=boot");
}
