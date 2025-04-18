use clap::Parser;
use serde_json::{Map, Value, json};
use std::{env, path, process};

mod parse;
mod scan;
mod soup;
mod utils;

use scan::dir_scan;
use soup::model::SoupContexts;

/// Scans a given repository for software of unknown provenance (SOUP) and outputs them in a file.
#[derive(Parser)]
#[clap(version)]
struct Cli {
    /// Output file to print report in
    #[clap(short = 'o', long = "output-file", value_parser)]
    file: path::PathBuf,

    /// Directory to scan
    #[clap(short = 'd', long = "directory", value_parser)]
    root_dir: Option<path::PathBuf>,

    /// Directory to exclude
    #[clap(short = 'e', long = "exclude-directory", value_parser)]
    exclude_dirs: Vec<path::PathBuf>,

    // Key to add in meta property
    #[clap(short = 'm', long = "meta-key")]
    meta_keys: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    let output_file = parse_output_file(args.file);
    let mut current_contexts = match output_file.is_file() {
        true => match SoupContexts::read_from_file(&output_file) {
            Ok(contexts) => contexts,
            Err(e) => {
                eprintln!(
                    "Not able to parse output file: {} ({})",
                    output_file.display(),
                    e
                );
                process::exit(1);
            }
        },
        false => SoupContexts::empty(),
    };

    let root_dir = parse_root_dir(args.root_dir);
    let exclude_dirs = args.exclude_dirs;
    let default_meta = args
        .meta_keys
        .into_iter()
        .map(|meta_key| (meta_key, json!("")))
        .collect::<Map<String, Value>>();
    let scanned_contexts = match dir_scan::scan(&root_dir, &exclude_dirs, default_meta) {
        Ok(result) => result,
        Err(e) => {
            eprintln!(
                "Error while scanning directory: {} ({})",
                root_dir.display(),
                e
            );
            process::exit(1);
        }
    };

    current_contexts.apply(scanned_contexts);
    if let Err(e) = current_contexts.write_to_file(&output_file) {
        eprintln!("Error while writing to file: {}", e);
        process::exit(1);
    }
}

fn parse_root_dir(dir: Option<path::PathBuf>) -> path::PathBuf {
    let root_dir = match dir {
        Some(target_dir) => target_dir,
        None => match env::current_dir() {
            Ok(current_dir) => current_dir,
            Err(e) => {
                eprintln!("Could not obtain current directory: {}", e);
                process::exit(1);
            }
        },
    };
    if !root_dir.exists() || !root_dir.is_dir() {
        eprintln!("Invalid directory: {}", root_dir.display());
        process::exit(1);
    }
    root_dir
}

fn parse_output_file(file_path: path::PathBuf) -> path::PathBuf {
    if file_path.exists() && !file_path.is_file() {
        eprintln!("Invalid output file: {}", file_path.display());
        process::exit(1);
    }
    file_path
}
