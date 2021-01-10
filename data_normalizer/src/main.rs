use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

// CLI tools
use anyhow::{Context, Result};
use console::style;
use indicatif::{HumanBytes, ProgressBar};
use structopt::StructOpt;

#[derive(StructOpt)]
struct CliInput {
    /// Directory to read the data from.
    /// Currently expects each entry to be in its own directory from what is given
    #[structopt(parse(from_os_str))]
    dir: PathBuf,

    /// Where to output the normalized data
    #[structopt(parse(from_os_str))]
    output_dir: PathBuf,

    /// Removes lines that start with this patterns
    #[structopt(short, long, default_value = ">")]
    pattern: String,

    /// If given, only the accepted characters are included
    /// and assemblies containing other chars are discarded
    #[structopt(short = "c", long = "accepted-characters")]
    accepted_characters_str: Option<String>,

    /// Denotes the file extension to look at
    #[structopt(short, long, default_value = "fna")]
    ext: String,

    /// Denotes whether to accept only if a single file with `ext` is there, or if multiple should be accepted and concatenated
    #[structopt(short, long)]
    multiple_files: bool,
}

impl CliInput {
    pub fn accepted_characters(&self) -> Option<Vec<char>> {
        self.accepted_characters_str
            .as_ref()
            .map(|s| s.to_uppercase().chars().collect())
    }
}

fn main() -> Result<()> {
    let args = CliInput::from_args();

    eprintln!("Input: {}", args.dir.display());
    eprintln!("Output: {}", args.output_dir.display());

    if !args.multiple_files {
        single_file_impl(&args)?;
    } else {
        multiple_files_impl(&args)?;
    }

    // Debug info (folders)
    // * List # of acceptable folders out of total
    // * List the set of file extensions in all folders
    // * List total size of acceptable files
    // ---- Above first
    // Debug info (files)
    // * Find the set of characters in the files after removing pattern lines
    // *

    Ok(())
}

fn single_file_impl(args: &CliInput) -> Result<()> {
    let dir = fs::read_dir(&args.dir)
        .with_context(|| format!("Could not read directory `{}`", args.dir.display()))?
        .collect::<Vec<_>>();

    // Loop through folders in dir, find ones that are "acceptable"
    // Acceptable being only contains a single .fna file

    eprintln!(
        "{} Looping through directory...",
        style("[1/2]").bold().dim()
    );

    let mut acceptable_dir: Vec<PathBuf> = Vec::new();
    let mut denied_dir: Vec<PathBuf> = Vec::new();
    let mut extensions: HashSet<OsString> = HashSet::new();
    for dir_entry in dir {
        let assembly_path = dir_entry?.path();
        if assembly_path.is_dir() {
            let mut found_fna = false;
            let mut denied = false;
            for assembly_entry in fs::read_dir(&assembly_path)? {
                let ext_path = assembly_entry?.path();
                let ext = ext_path.extension().unwrap();
                extensions.insert(ext.to_os_string());
                let ext = ext.to_str().unwrap();

                if found_fna && ext == &args.ext {
                    denied = true;
                }

                if ext == &args.ext {
                    found_fna = true;
                }
            }

            if denied {
                denied_dir.push(assembly_path);
            } else {
                acceptable_dir.push(assembly_path);
            }
        }
    }

    // Second pass
    // Loop through acceptable dirs and make sure they only contain
    // accepted characters after lines matching the pattern have been filtered out
    if let Some(ac) = args.accepted_characters() {
        eprintln!(
            "{} 2nd pass through directory, this time looking at all characters...",
            style("[1.5/2]").bold().dim()
        );

        let pb = ProgressBar::new(acceptable_dir.len() as u64);

        // Gets the '&args.ext' file
        fn get_file(p: &PathBuf, ext: &str) -> Option<File> {
            for entry in fs::read_dir(&p).unwrap() {
                let p = entry.unwrap().path();
                if p.extension()?.to_str()? == ext {
                    return Some(File::open(&p).unwrap());
                }
            }
            None
        }
        acceptable_dir.retain(|dir_entry| {
            pb.inc(1);

            let mut delete = false;
            let file = get_file(&dir_entry, &args.ext).unwrap();
            // let file = File::open(&ext_path)?;
            let buf_reader = BufReader::new(file);
            for line in buf_reader.lines() {
                if let Ok(s) = line {
                    if !s.starts_with(&args.pattern) {
                        if s.to_uppercase().chars().any(|c| !ac.contains(&c)) {
                            denied_dir.push(dir_entry.clone());
                            delete = true;
                        }
                    }
                }
            }
            !delete
        });
        pb.finish_and_clear();
    }

    // Debug
    eprintln!("Ext: {:#?}", extensions);
    eprintln!(
        "Accepted {} out of {}",
        acceptable_dir.len(),
        acceptable_dir.len() + denied_dir.len()
    );

    eprintln!(
        "{} Looping through accepted directories & writing the formatted files to the output directory...",
        style("[2/2]").bold().dim()
    );

    let pb = ProgressBar::new(acceptable_dir.len() as u64);

    // File todo:
    // Take pattern to remove lines
    // Remove newlines
    // Write to file with dir name
    let mut total_size = 0;
    let mut characters: HashMap<char, usize> = HashMap::new();
    for dir_entry in acceptable_dir {
        pb.inc(1);

        let folder_name = dir_entry
            .file_name()
            .unwrap_or_else(|| panic!("Could not read foldername of `{}`", dir_entry.display()));
        for entry in fs::read_dir(&dir_entry)? {
            let path = entry?.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_str().unwrap();
                if ext != &args.ext {
                    continue;
                }
            }

            let file = File::open(&path)?;
            let mut contents = match file.metadata() {
                Ok(d) => String::with_capacity(d.len() as usize),
                Err(_) => String::new(),
            };
            let buf_reader = BufReader::new(file);
            for line in buf_reader.lines() {
                if let Ok(s) = line {
                    if !s.starts_with(&args.pattern) {
                        let formatted_string = s.to_uppercase();
                        contents.push_str(&formatted_string);

                        formatted_string.chars().for_each(|c| {
                            // Insert returns a boolean whether it existed
                            // previously or not, but we just want all characters
                            // added to the hash set for debugging.
                            *characters.entry(c).or_insert(0) += 1;
                        });
                    }
                }
            }
            contents.shrink_to_fit();
            total_size += contents.len();

            // Add to output directory
            let output_path = Path::new(&args.output_dir).join(folder_name);
            fs::write(&output_path, contents).with_context(|| {
                format!(
                    "Could not create or write output file at `{}`",
                    output_path.display()
                )
            })?;
        }
    }

    pb.finish_and_clear();

    eprintln!("Total size: {}", HumanBytes(total_size as u64));
    eprintln!("Unique characters: {:#?}", characters);

    Ok(())
}

fn multiple_files_impl(args: &CliInput) -> Result<()> {
    let dir = fs::read_dir(&args.dir)
        .with_context(|| format!("Could not read directory `{}`", args.dir.display()))?
        .collect::<Vec<_>>();

    // Loop through folders in dir, find ones that are "acceptable"
    // Acceptable being only contains a single .fna file

    eprintln!(
        "{} Looping through directory...",
        style("[1/2]").bold().dim()
    );

    let mut acceptable_dir: Vec<PathBuf> = Vec::new();
    let mut extensions: HashSet<OsString> = HashSet::new();
    for dir_entry in dir {
        let assembly_path = dir_entry?.path();
        if assembly_path.is_dir() {
            let mut found_ext = false;
            for assembly_entry in fs::read_dir(&assembly_path)? {
                let ext_path = assembly_entry?.path();
                let ext = ext_path.extension().unwrap();
                extensions.insert(ext.to_os_string());
                let ext = ext.to_str().unwrap();

                if ext == &args.ext {
                    found_ext = true;
                }
            }

            if found_ext {
                acceptable_dir.push(assembly_path);
            }
        }
    }

    // Second pass
    // Loop through acceptable dirs and make sure they only contain
    // accepted characters after lines matching the pattern have been filtered out
    if let Some(ac) = args.accepted_characters() {
        eprintln!(
            "{} 2nd pass through directory, this time looking at all characters...",
            style("[1.5/2]").bold().dim()
        );

        let pb = ProgressBar::new(acceptable_dir.len() as u64);

        // Gets the '&args.ext' file
        fn get_files(p: &PathBuf, ext: &str) -> Vec<File> {
            let mut files = vec![];
            for entry in fs::read_dir(&p).unwrap() {
                let p = entry.unwrap().path();
                if p.extension().unwrap().to_str().unwrap() == ext {
                    files.push(File::open(&p).unwrap());
                }
            }
            files
        }
        acceptable_dir.retain(|dir_entry| {
            pb.inc(1);

            let mut delete = false;
            let files = get_files(&dir_entry, &args.ext);
            for file in files {
                let buf_reader = BufReader::new(file);
                for line in buf_reader.lines() {
                    if let Ok(s) = line {
                        if !s.starts_with(&args.pattern) {
                            if s.to_uppercase().chars().any(|c| !ac.contains(&c)) {
                                // Todo: Can probably early return from this double inner for loop, but oh god the time presssure
                                delete = true;
                            }
                        }
                    }
                }
            }
            !delete
        });
        pb.finish_and_clear();
    }

    // Debug
    eprintln!("Ext: {:#?}", extensions);
    eprintln!("Accepted {}", acceptable_dir.len());

    eprintln!(
        "{} Looping through accepted directories & writing the formatted files to the output directory...",
        style("[2/2]").bold().dim()
    );

    let pb = ProgressBar::new(acceptable_dir.len() as u64);
    let mut total_size = 0;
    let mut characters: HashMap<char, usize> = HashMap::new();
    for dir_entry in acceptable_dir {
        pb.inc(1);

        let folder_name = dir_entry
            .file_name()
            .unwrap_or_else(|| panic!("Could not read foldername of `{}`", dir_entry.display()));
        let mut file_content = String::new();
        for entry in fs::read_dir(&dir_entry)? {
            let path = entry?.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_str().unwrap();
                if ext != &args.ext {
                    continue;
                }
            }

            let file = File::open(&path)?;
            let mut contents = match file.metadata() {
                Ok(d) => String::with_capacity(d.len() as usize),
                Err(_) => String::new(),
            };
            let buf_reader = BufReader::new(file);
            for line in buf_reader.lines() {
                if let Ok(s) = line {
                    if !s.starts_with(&args.pattern) {
                        let formatted_string = s.to_uppercase();
                        contents.push_str(&formatted_string);

                        formatted_string.chars().for_each(|c| {
                            // Insert returns a boolean whether it existed
                            // previously or not, but we just want all characters
                            // added to the hash set for debugging.
                            *characters.entry(c).or_insert(0) += 1;
                        });
                    }
                }
            }
            contents.shrink_to_fit();
            file_content.push_str(&contents);
        }
        total_size += file_content.len();

        // Add to output directory
        let output_path = Path::new(&args.output_dir).join(folder_name);
        fs::write(&output_path, file_content).with_context(|| {
            format!(
                "Could not create or write output file at `{}`",
                output_path.display()
            )
        })?;
    }

    pb.finish_and_clear();

    eprintln!("Total size: {}", HumanBytes(total_size as u64));
    eprintln!("Unique characters: {:#?}", characters);

    Ok(())
}
