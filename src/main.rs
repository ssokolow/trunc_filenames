//! Simple tool to rename files and directories to fit length limits.
//!
//! **WARNING:** Will not preserve secondary extensions like `.tar.gz`

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use clap::Parser;
use walkdir::WalkDir;

/// Command-line argument schema
#[derive(Debug, Parser)]
#[clap(version, about = "Rename files and directories to fit length limits.\n\nWARNING: Will not preserve secondary extensions like .tar.gz", long_about = None)]
struct CliArgs {
    /// Paths to rename (recursively, if directories)
    path: Vec<PathBuf>,

    /// Length to truncate to. (Default chosen for rclone name encryption)
    #[clap(long, default_value_t = 140)]
    max_len: usize,

    /// Don't actually rename files. Just print.
    #[clap(short = 'n', long, action, default_value_t = false)]
    dry_run: bool,
}

/// Figure out the new name when truncating a path
///
/// **NOTE:** Handling of non-UTF8-able path is currently hacky
fn trunc_path(path: &Path, max_len: usize) -> Result<Cow<'_, Path>, Box<dyn Error>> {
    let fname = match path.file_name() {
        Some(os_str) => os_str,
        None => return Ok(Cow::from(path)),
    };

    // POSIX-specific but semantically correct. If I ever port this to Windows, I'll need to figure
    // out what RClone considers the length limit to be relative to anyway.
    let raw = fname.as_bytes();

    // Just return if it's already short enough
    let raw_trunc = if let Some(trunc) = raw.get(..max_len) {
        if raw.len() < max_len {
            return Ok(Cow::from(path));
        }
        trunc
    } else {
        return Ok(Cow::from(path));
    };

    let new_fname_len = if std::str::from_utf8(raw).is_ok() {
        // if it's UTF-8-able, then truncate and let the UTF-8 parser figure out where to split
        match std::str::from_utf8(raw_trunc) {
            Ok(_) => raw_trunc.len(),
            Err(e) => e.valid_up_to(),
        }
    } else {
        // For now, just let stuff that's already invalid UTF-8 end in a chopped code point
        //
        // TODO: Implement properly
        raw_trunc.len()
    };

    let path_raw = path.as_os_str().as_bytes();
    let mut new_len = path_raw.len() - (raw.len() - new_fname_len);
    if let Some(ext) = path.extension() {
        new_len = new_len.saturating_sub(ext.len()).saturating_sub(1); // for the dot
    }

    let new_result = path.as_os_str().as_bytes().get(..new_len).expect("truncate within len");

    let mut new_path = PathBuf::from(OsStr::from_bytes(new_result));
    if let Some(ext) = path.extension() {
        new_path.set_extension(ext);
    }
    Ok(Cow::from(new_path))
}

/// Rename a file/directory name to truncate it
fn rename_path(path: &Path, max_len: usize, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let new_path = trunc_path(path, max_len)?;
    if new_path == path {
        return Ok(());
    }

    println!(
        "Truncating name: {:?} → {:?}",
        path.file_name().unwrap_or_else(|| OsStr::new("")),
        new_path.file_name().unwrap_or_else(|| OsStr::new(""))
    );
    if !dry_run {
        std::fs::rename(path, new_path)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    for root in args.path {
        for entry in WalkDir::new(root) {
            let inner_entry = if let Ok(inner) = entry {
                inner
            } else {
                eprintln!("Error getting entry: {:?}", entry);
                continue;
            };

            if let Err(e) = rename_path(inner_entry.path(), args.max_len, args.dry_run) {
                eprintln!("Error while renaming {}: {}", inner_entry.path().display(), e)
            }
        }
    }

    Ok(())
}
