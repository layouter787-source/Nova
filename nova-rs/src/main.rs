use std::fs;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::process;
use std::thread;

use nova::NovaError;

fn report(src: &str, e: &NovaError) {
    eprintln!("{}", e);
    if let Some(l) = e.line {
        if let Some(text) = src.lines().nth(l.saturating_sub(1)) {
            eprintln!("  --> {}", text.trim());
        }
    }
}

fn run(src: &str, base_dir: PathBuf) -> i32 {
    let out = Box::new(BufWriter::new(io::stdout()));
    match nova::run_source(src, out, base_dir) {
        Ok(()) => 0,
        Err(e) => {
            report(src, &e);
            1
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: nova <file.nv>");
        process::exit(1);
    }
    let path = args[1].clone();
    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Cannot read '{}': {}", path, e);
            process::exit(1);
        }
    };

    let base_dir = Path::new(&path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    // Run on a thread with a big stack so deep recursion in Nova programs
    // hits our call-depth limit instead of crashing the process.
    let handle = thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || run(&src, base_dir))
        .expect("failed to start interpreter thread");
    let code = handle.join().unwrap_or(1);
    process::exit(code);
}
