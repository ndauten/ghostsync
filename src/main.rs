use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::Local;
use walkdir::WalkDir;
use clap::{Arg, Command};
use xattr::FileExt;

fn is_dataless(path: &Path) -> bool {
    if let Ok(file) = File::open(path) {
        if let Ok(xattrs) = file.list_xattr() {
            for attr in xattrs {
                if attr.to_string_lossy().contains("com.apple.") {
                    if attr.to_string_lossy().contains("dataless") ||
                       attr.to_string_lossy().contains("cloud") ||
                       attr.to_string_lossy().contains("fileprovider") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn main() {
    let matches = Command::new("copy_local")
        .about("Copy non-dataless files from a source to a destination, preserving structure.")
        .arg(Arg::new("source").required(true).help("Source directory"))
        .arg(Arg::new("dest").required(true).help("Destination directory"))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .help("Print to stdout as well as log file"))
        .arg(Arg::new("logfile")
            .short('l')
            .long("logfile")
            .value_name("FILE")
            .help("Output log file path"))
        .get_matches();

    let source = Path::new(matches.get_one::<String>("source").unwrap());
    let dest = Path::new(matches.get_one::<String>("dest").unwrap());

    let log_path = matches
        .get_one::<String>("logfile")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let ts = Local::now().format("copy_log_%Y%m%d_%H%M%S.txt").to_string();
            PathBuf::from(ts)
        });

    let verbose = matches.get_flag("verbose");

    let log_file = File::create(&log_path).expect("Unable to create log file");
    let mut log = BufWriter::new(log_file);

    for entry in WalkDir::new(source).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()) {
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(source).unwrap();
        let dst_path = dest.join(rel_path);

        if is_dataless(src_path) {
            let msg = format!("SKIPPED (dataless): {}\n", src_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            if verbose { print!("{}", msg); }
            continue;
        }

        if dst_path.exists() {
            let msg = format!("SKIPPED (exists): {}\n", rel_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            if verbose { print!("{}", msg); }
            continue;
        }

        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|_| panic!("Failed to create {}", parent.display()));
        }

        fs::copy(src_path, &dst_path).unwrap_or_else(|e| panic!("Failed to copy {}: {}", src_path.display(), e));
        let msg = format!("COPIED: {}\n", rel_path.display());
        log.write_all(msg.as_bytes()).unwrap();
        if verbose { print!("{}", msg); }
    }

    println!("Done. Log saved to {}", log_path.display());
}

