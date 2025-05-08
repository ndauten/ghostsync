use clap::{Arg, ArgAction, Command};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use chrono::Local;
use walkdir::WalkDir;
use xattr::FileExt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};


fn is_dataless(path: &Path) -> bool {
    // BSD flag check via ls -ldO for "dataless"
    if let Ok(output) = std::process::Command::new("ls")
        .arg("-ldO")
        .arg(path)
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if stdout.contains("dataless") {
                /* TODO: Add verbosity printing for this later */
                //println!("BSD FLAG dataless detected on {}", path.display());
                return true;
            }
        }
    }

    false
}

fn log_xattrs(path: &Path, log: &mut BufWriter<File>) {
    if let Ok(file) = File::open(path) {
        if let Ok(xattrs) = file.list_xattr() {
            let _ = writeln!(log, "xattrs for {}:", path.display());
            for attr in xattrs {
                let key = attr.to_string_lossy();
                let _ = writeln!(log, " - {}", key);
                /* TODO: add these for inspection and debugging */
                //println!(" - {}", key);
                //if key.contains("com.apple.") &&
                   //(key.contains("dataless") || key.contains("cloud") || key.contains("fileprovider")) {
                    //found = true;
                //}
            }
        }
    }
}

fn main() {
    let matches = Command::new("ghostsync")
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
        .arg(Arg::new("backuplog")
            .short('b')
            .long("backup")
            .help("Backup Output log file: defaults to /tmp/ghostsync_log_*")
            .action(ArgAction::SetTrue))
        .get_matches();

    let source = Path::new(matches.get_one::<String>("source").unwrap());
    let dest = Path::new(matches.get_one::<String>("dest").unwrap());

    let default_log_path = PathBuf::from("ghostsync_log.txt");
    if default_log_path.exists() && matches.get_flag("backuplog") {
        let backup_name = format!("ghostsync_log_backup_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
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
    println!("Analyzing directory: {}", source.display());
    println!("Total files detected for processing: {}", total);
    let _ = writeln!(log, "Analyzing directory: {}", source.display());
    let _ = writeln!(log, "Total files detected for processing: {}", total);
    let mut processed = 0;

    let mut skipped_dataless = 0;
    let mut skipped_exists = 0;
    let mut copied = 0;

    for entry in entries {
        processed += 1;
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(source).unwrap();
        let dst_path = dest.join(rel_path);

        if is_dataless(src_path) {
            skipped_dataless += 1;
            let msg = format!("SKIPPED (dataless): {}\n", src_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            log_xattrs(src_path, &mut log);
            if verbose { print!("{}", msg); }
        } else if dst_path.exists() {
            skipped_exists += 1;
            let msg = format!("SKIPPED (exists): {}\n", rel_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            log_xattrs(src_path, &mut log);
            if verbose { print!("{}", msg); }
        } else {
            copied += 1;
            if verbose { println!("COPYING: {}", src_path.display()); }
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|_| panic!("Failed to create {}", parent.display()));
            }
            fs::copy(src_path, &dst_path).unwrap_or_else(|e| panic!("Failed to copy {}: {}", src_path.display(), e));
            let msg = format!("COPIED: {}\n", rel_path.display());
            log.write_all(msg.as_bytes()).unwrap();
            log_xattrs(src_path, &mut log);
            if verbose { print!("{}", msg); }
        }

        /* Print a progress bar so it looks nice */
        if processed % 100 == 0 {
            let percent = (processed as f64 / total as f64) * 100.0;
            let bar_len = 50;
            let filled = ((percent / 100.0) * bar_len as f64).round() as usize;
            let empty = bar_len - filled;

            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
            let _ = write!(stdout, "\r[{}", "#".repeat(filled));
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
            let _ = write!(stdout, "{}] {:.1}% ({}/{})", "-".repeat(empty), percent, processed, total);
            let _ = stdout.flush();
        }
    }

    println!("\n\nSummary:");
    println!("  Total files scanned:        {}", total);
    println!("  Files copied:               {}", copied);
    println!("  Files skipped (exists):     {}", skipped_exists);
    println!("  Files skipped (dataless):   {}", skipped_dataless);
    println!("  Log saved to:               {}", log_path.display());
}
