use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Find *.rs recursively
pub fn find_rust_files(rust_path: &str) -> Vec<PathBuf> {
    WalkDir::new(rust_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().is_some()
                && e.path().extension().unwrap() == "rs"
        })
        .map(|e| e.into_path())
        .collect()
}

/// Parse a rust file into a TokenTree
pub fn parse_file(path: PathBuf) -> syn::File {
    let mut file = File::open(&path).expect("Unable to open file");
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    syn::parse_file(&src).expect("Unable to parse file")
}
