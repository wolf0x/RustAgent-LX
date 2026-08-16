use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Escape a string for embedding as a Rust string literal.
fn escape_for_rust(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn embed_workspace_files(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("embedded_files.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    let files = ["AGENTS.md", "SOUL.md", "TOOLS.md", "USER.md"];

    write!(f, "&[").unwrap();
    for name in &files {
        let path = Path::new(name);
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            let escaped = escape_for_rust(&content);
            write!(f, "(\"{}\", \"{}\"),", name, escaped).unwrap();
        }
    }
    write!(f, "]").unwrap();

    for name in &files {
        println!("cargo:rerun-if-changed={}", name);
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    embed_workspace_files(&out_dir);

    // Embed Windows application icon (only on Windows target)
    #[cfg(target_os = "windows")]
    {
        let icon_path = "RustAgent.ico";
        if Path::new(icon_path).exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon(icon_path);
            if let Err(e) = res.compile() {
                eprintln!("Failed to compile Windows resource: {}", e);
            }
            println!("cargo:rerun-if-changed={}", icon_path);
        }
    }
}
