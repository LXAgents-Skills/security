//! Embed the published set.
//!
//! Every `.md` under `content/` becomes one entry of a generated table, `(path, text)`, with
//! the text pulled in by `include_str!`. The binary therefore serves the set without knowing
//! where it was built, and the request path never touches the filesystem.
//!
//! This script only lists files. Naming a tool, and refusing a set that cannot be routed on,
//! is the library's job (`src/instructions/`), done once at startup, so the rules live in one
//! place and are unit-tested there.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn markdown_files(dir: &Path, base: &Path, found: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            markdown_files(&path, base, found);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let relative = path.strip_prefix(base).expect("path under content/");
            let parts: Vec<_> = relative
                .iter()
                .map(|part| part.to_string_lossy().into_owned())
                .collect();
            found.push(parts.join("/"));
        }
    }
}

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let content = root.join("content");
    println!("cargo::rerun-if-changed=content");

    let mut files = Vec::new();
    markdown_files(&content, &content, &mut files);
    files.sort();

    let mut table = String::from("&[\n");
    for file in &files {
        let absolute = content.join(file);
        table.push_str(&format!(
            "    ({file:?}, include_str!({:?})),\n",
            absolute.display().to_string()
        ));
    }
    table.push_str("]\n");

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("content_table.rs");
    fs::write(out, table).expect("write the content table");
}
