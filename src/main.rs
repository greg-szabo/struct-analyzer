mod core;
mod fs;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Input folder for the Tendermint source code.
    #[structopt(name = "source_directory", parse(from_os_str))]
    rust_path: PathBuf,

    /// Use this flag to omit printing the CSV header.
    /// This is useful when you want to concatenate multiple outputs.
    #[structopt(short, long)]
    no_header: bool,

    /// Print only JSON serializable structs and implicit serialization dependency links.
    #[structopt(short, long)]
    json: bool,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let args = Cli::from_args();
    let rust_path: &str = args.rust_path.to_str().unwrap();
    let no_header: bool = args.no_header;
    let only_json: bool = args.json;
    let output: Option<PathBuf> = args.output;

    let files = fs::find_rust_files(&rust_path);
    let mut collection = core::db::Collection::new();
    for file in files {
        let syntax = fs::parse_file(file.clone());

        let id_prefix = if rust_path.ends_with(".rs") {
            file.file_name().unwrap().to_str()
        } else {
            file.strip_prefix(&rust_path).unwrap().to_str()
        }
        .unwrap()
        .strip_suffix(".rs")
        .unwrap();

        collection.add_items(syntax.items, id_prefix);
    }
    if let Some(o) = output {
        let mut f = File::create(o).expect("file creation failed");
        f.write_all(collection.parse_to_csv(only_json, no_header).as_bytes())
            .expect("file write failed");
    } else {
        println!("{}", collection.parse_to_csv(only_json, no_header));
    }
}
