# LearnOS

An OS (kernel) written in Rust to learn OS development.

## Architecture

The kernel uses the multiboot2 standard in order to be booted by Grub.
The 32 bit startup code sets up an identity page mapping for the lowest 1 GiB
of memory and switches to long mode. It then enters 64 bit Rust code.

## Prerequisites

- Rust nightly
- Cargo
- `cargo-xbuild`
- `nasm`
- Grub (specifically `grub-mkrescue`)

## Build process

The rust multiboot2 binary is compiled as static library and only later
transformed into an executable elf binary using `ld`, because I could
not figure out how to use a custom linker script for a specific rust
binary via cargo.

This project currently uses a plain and simple make file, and why I would like
to use a better build system, it's not a top priority.

To run the kernel in Qemu, execute

```
make run
```

That will build the rust staticlib, convert it to an executable ELF file with
the correct layout, build an ISO image containing grub and the kernel, and
boot it using qemu.