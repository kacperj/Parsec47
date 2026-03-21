fn main() {
    // llvm-mingw uses libunwind instead of libgcc_eh for exception handling.
    // Rust's x86_64-pc-windows-gnu target links -lgcc_eh expecting GCC's unwind
    // runtime, so we explicitly link libunwind statically to provide _Unwind_*
    // symbols without requiring libunwind.dll at runtime.
    println!("cargo:rustc-link-lib=static=unwind");
}
