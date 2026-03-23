use std::fs;
use std::path::Path;

fn main() {
    // llvm-mingw uses libunwind instead of libgcc_eh for exception handling.
    // Rust's x86_64-pc-windows-gnu target links -lgcc_eh expecting GCC's unwind
    // runtime, so we explicitly link libunwind statically to provide _Unwind_*
    // symbols without requiring libunwind.dll at runtime.
    println!("cargo:rustc-link-lib=static=unwind");

    generate_def();
}

fn generate_def() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let def_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("p47rust.def");

    let mut exports: Vec<String> = Vec::new();

    for entry in fs::read_dir(&src_dir).expect("src/ not found") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());

        let source = fs::read_to_string(&path).unwrap();
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("pub extern \"C\" fn ") {
                if let Some(name) = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next() {
                    if !name.is_empty() {
                        exports.push(name.to_string());
                    }
                }
            }
        }
    }

    exports.sort();

    let mut content = String::from("LIBRARY p47rust.dll\nEXPORTS\n");
    for name in &exports {
        content.push_str("    ");
        content.push_str(name);
        content.push('\n');
    }

    fs::write(&def_path, &content).expect("failed to write p47rust.def");
}
