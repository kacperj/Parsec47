use std::path::Path;
use std::process::Command;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        // llvm-mingw uses libunwind instead of libgcc_eh for exception handling.
        // Rust's x86_64-pc-windows-gnu target links -lgcc_eh expecting GCC's
        // unwind runtime, so we explicitly link libunwind statically to provide
        // _Unwind_* symbols without requiring libunwind.dll at runtime.
        //
        // `-bundle`: don't pull libunwind.a into the rlib at compile time (rustc
        // can't locate it in the mingw sysroot); defer it to the final binary
        // link, where the mingw linker resolves it. Still statically linked.
        println!("cargo:rustc-link-lib=static:-bundle=unwind");

        embed_windows_icon();
    }
    // BulletML is now the pure-Rust `bulletml` crate (a normal Cargo dependency),
    // so there is no shared library to locate or set an rpath for.
}

/// Compile `resource/p47.rc` (the application icon) with windres and link the
/// resulting COFF object into the executable. windres comes from the llvm-mingw
/// toolchain as `x86_64-w64-mingw32-windres`; override with the `WINDRES` env
/// var. The project builds the Windows target only via the cross-compile Docker
/// image, where that tool is always present.
fn embed_windows_icon() {
    let res_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../resource");
    let rc = res_dir.join("p47.rc");
    let out = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let obj = format!("{}/p47_res.o", out);

    println!("cargo:rerun-if-changed={}", rc.display());
    println!("cargo:rerun-if-changed={}", res_dir.join("p47.ico").display());

    let windres = std::env::var("WINDRES").unwrap_or_else(|_| "x86_64-w64-mingw32-windres".into());
    let status = Command::new(&windres)
        .arg("-O")
        .arg("coff")
        .arg(&rc)
        .arg(&obj)
        .status()
        .unwrap_or_else(|e| panic!("failed to run {windres}: {e}"));
    if !status.success() {
        panic!("{windres} failed to compile {}", rc.display());
    }

    println!("cargo:rustc-link-arg={}", obj);
}
