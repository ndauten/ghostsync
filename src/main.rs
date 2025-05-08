use clap::{Arg, ArgAction, Command};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use chrono::Local;
use walkdir::WalkDir;
use xattr::FileExt;

fn is_dataless(path: &Path) -> bool {
    if let Ok(file) = File::open(path) {
        if let Ok(xattrs) = file.list_xattr() {
            let mut found = false;
            println!("xattrs for {}:", path.display());
            for attr in xattrs {
                let key = attr.to_string_lossy();
                println!(" - {}", key);
                if key.contains("com.apple.") &&
                   (key.contains("dataless") || key.contains("cloud") || key.contains("fileprovider")) {
                    found = true;
                }
            }
            return found;
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
            .help("Print to stdout as well as log file")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("logfile")
            .short('l')
            .long("logfile")
            .value_name("FILE")
            .help("Output log file path"))
        .get_matches();

    let source = Path::new(matches.get_one::<String>("source").unwrap());
    let dest = Path::new(matches.get_one::<String>("dest").unwrap());

    let default_log_path = PathBuf::from("copy_log_latest.txt");
    if default_log_path.exists() {
        let backup_name = format!("copy_log_backup_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
        fs::rename(&default_log_path, backup_name).expect("Failed to create log backup");
    }

    let log_path = matches
        .get_one::<String>("logfile")
        .map(PathBuf::from)
        .unwrap_or(default_log_path);

    let verbose = matches.get_flag("verbose");

    let log_file = File::create(&log_path).expect("Unable to create log file");
    let mut log = BufWriter::new(log_file);

    let entries: Vec<_> = WalkDir::new(source)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .collect();

    let total = entries.len();
    let mut processed = 0;

    for entry in entries {
        processed += 1;
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(source).unwrap();
        let dst_path = dest.join(rel_path);

        if is_dataless(src_path) {
            let msg = format!("SKIPPED (dataless): {}\n", src_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            if verbose { print!("{}", msg); }
        } else if dst_path.exists() {
            let msg = format!("SKIPPED (exists): {}\n", rel_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            if verbose { print!("{}", msg); }
        } else {
            println!("COPYING: {}", src_path.display());
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|_| panic!("Failed to create {}", parent.display()));
            }

            fs::copy(src_path, &dst_path).unwrap_or_else(|e| panic!("Failed to copy {}: {}", src_path.display(), e));
            let msg = format!("COPIED: {}\n", rel_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            if verbose { print!("{}", msg); }
        }

        if verbose && processed % 100 == 0 {
            let percent = (processed as f64 / total as f64) * 100.0;
            println!("Progress: {:.1}% ({}/{})", percent, processed, total);
        }
    }

    println!("Done. Processed {} files. Log saved to {}", total, log_path.display());
}
