//! OS entry point (replaces the D `P47Boot` shim's `WinMain` / `main`).
//!
//! `windows_subsystem = "windows"` makes the Windows build a GUI binary with no
//! console window, replacing the old `-L=/SUBSYSTEM:WINDOWS` linker flag.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(p47rust::boot::run(args));
}
