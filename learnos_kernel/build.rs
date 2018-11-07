// build.rs

extern crate nasm_rs;

fn main() {
    nasm_rs::compile_library_args("libboot.a", &["src/bootcode/header.asm", "src/bootcode/boot_bsp.asm"], &["-f", "elf64"]);
    println!("cargo:rustc-link-lib=static=boot");
}
