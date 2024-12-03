use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
};

use chrono::Utc;
use walkdir::WalkDir;

pub struct Config {
    query: Option<String>,
    repo_url: PathBuf,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // First arg is always name of process

        let query = args.next();

        // Resolve the user's home directory
        let home_dir = env::var("HOME").expect("Could not determine the home directory.");
        let nova_dir = Path::new(&home_dir).join(".nova");

        // Check if the directory exists, and create it if not
        if !nova_dir.exists() {
            if let Err(e) = fs::create_dir(&nova_dir) {
                eprintln!("Cannot create {:?} dir: {}", nova_dir, e);
                return Err("Cannot create {e:?} dir");
            } else {
                println!("{:?} directory created successfully", nova_dir);
            }
        } else {
            println!("{:?} directory already exists", nova_dir);
        }

        Ok(Config {
            query,
            repo_url: nova_dir,
        })
    }
}

pub fn search(dir: &PathBuf, term: &str) -> Vec<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .flat_map(|entry| {
            let path = entry.path().to_path_buf();
            File::open(&path)
                .ok()
                .map(BufReader::new)
                .into_iter()
                .flat_map(|file| file.lines().filter_map(Result::ok))
                .filter(|line| line.contains(term))
                .map(move |line| {
                    println!("{line} in {path:?}");
                    line
                })
        })
        .collect()
}

pub fn open(dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let today = Utc::now();
    let date = today.format("%d-%m-%Y");
    let file_path = date.to_string() + ".md";

    if let Some(path) = dir.to_str() {
        let new_file_path = format!("{path}/{file_path}");
        println!("new file path is {new_file_path}");

        // Create file if it doesn't exist
        if !Path::new(&new_file_path).exists() {
            File::create(&new_file_path)?;
        }

        // Open the file
        Command::new("nvim").arg(&new_file_path).status()?;
    };

    Ok(())
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.query {
        Some(q) => {
            search(&config.repo_url, &q);
            Ok(())
        }
        None => open(&config.repo_url),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default_build_config() {
        let home_dir = env::var("HOME").expect("Could not determine the home directory");
        let nova_dir = Path::new(&home_dir).join(".nova");
        let config = Config::build(env::args()).unwrap();
        assert_eq!(nova_dir, config.repo_url);
    }

    #[test]
    fn search_notes() {
        let config = Config::build(env::args()).unwrap();
        let found = search(&config.repo_url, "hello");
        assert_eq!(vec!["hello wow!"], found);
    }
}
