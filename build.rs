use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["grammars/tree-sitter-php", "src"].iter().collect();

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .warnings(false)
        .compile("tree-sitter-php");
}
