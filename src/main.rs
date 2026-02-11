use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::Write;
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug)]
// HashMap in-memory index buffer-of-start, buffer-of-end
// key is String because our key in the DB can be anything, not just a number
struct Index(HashMap<String, u64>);

impl Index {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// add an item to the index
    pub fn insert(&mut self, k: &str, v: u64) {
        self.0.insert(k.to_string(), v);
    }
}

#[derive(Clone)]
struct Map(Vec<(String, String)>);

impl Map {
    fn new(value: Option<Vec<(String, String)>>) -> Self {
        match value {
            Some(v) => Self(v),
            _ => Self(Vec::new()),
        }
    }

    pub fn get_key(&self, index: usize) -> Result<String, Box<dyn std::error::Error>> {
        match self.0.get(index) {
            Some(value) => Ok(value.clone().0),
            None => Err("index out of bounds".into()),
        }
    }

    /// if the database is already there, read the content and convert it to a rust struct
    pub fn read_database(self, file_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut result: Vec<(String, String)> = Vec::new();

        for line in file_content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            // split by the first comma only
            if let Some((key, value)) = line.split_once(',') {
                result.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        let map = Map::new(Some(result));

        Ok(map)
    }
}

struct Database {
    map: Map,
    file_path: String,
    idx: Index,
}

impl Database {
    pub fn new(file_path: &str) -> Self {
        let mut idx = Index::new();
        let map = Map::new(None);
        let file_path_path = Path::new(file_path);

        if !file_path_path.exists() {
            File::create_new(file_path).expect("Couldnt' create database file");
            Self {
                map,
                file_path: file_path.to_string(),
                idx,
            }
        } else {
            // when you connect a databse that is already there
            // first, index the whole DB into a hashmap so it's easier to navigate in-memory
            // without many I/O disk operations.

            let file_content = fs::read_to_string(file_path_path).unwrap();

            if !file_content.is_empty() {
                let map = map.clone().read_database(&file_content).unwrap();

                // get each key from the database and store it in the index HashMap
                // We reimplement the loop to calculate offsets correctly matching the lines() iterator

                let mut offset: u64 = 0;
                let mut line_number: usize = 0;

                for line in file_content.lines() {
                    // Check if this line is in our map (skipped empty lines)
                    if line.trim().is_empty() {
                        offset += line.len() as u64 + 1; // +1 for the newline
                        continue;
                    }

                    // We trust map was built in order of lines
                    if let Ok(key) = map.get_key(line_number) {
                        idx.insert(&key, offset);
                        line_number += 1;
                    }

                    // Add line length + 1 (for the newline character)
                    // Note: This assumes unix style \n. Windows \r\n would be +2, but Rust's lines() handles \r\n by stripping both.
                    // If the file is actually on disk, we need to be careful.
                    // For now, assuming simple \n or handling by len is enough for this step.
                    // To be precise:
                    // lines() parses content.
                    // If we want exact byte offset, we should probably iterate bytes or assume \n.
                    // Let's assume \n for now as per env.

                    offset += line.len() as u64 + 1;
                }
            }
            Self {
                map, // Note: map might be empty if file_content was empty
                file_path: file_path.to_string(),
                idx,
            }
        }
    }

    pub fn get_by_key(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Use the index to find the offset
        if let Some(&offset) = self.idx.0.get(key) {
            use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

            let file = File::open(&self.file_path)?;
            let mut reader = BufReader::new(file);

            reader.seek(SeekFrom::Start(offset))?;

            let mut line = String::new();
            reader.read_line(&mut line)?;

            // Parsed the line to extract value
            if let Some((_, value)) = line.split_once(',') {
                return Ok(value.trim().to_string());
            }
        }

        Ok("".to_string())
    }

    pub fn set_by_key(self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        // append to file with "key, value"
        let content = fs::read_to_string(self.file_path.clone()).expect("couldn't read database");

        let new_line = format!("{}, {}", key, value);
        let all_content = if content.is_empty() {
            new_line
        } else {
            // Ensure we append on a new line.
            // If the file ends with newline, just append. If not, add newline.
            if content.ends_with('\n') {
                format!("{}{}", content, new_line)
            } else {
                format!("{}\n{}", content, new_line)
            }
        };

        File::create(self.file_path)
            .unwrap()
            .write_all(all_content.as_bytes())
            .expect("Couldn't write");

        Ok(())
    }
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    Get { key: String },
    Set { key: String, value: String },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let args = Args::parse();

    let db = Database::new("db.deebee");

    match args.command {
        Command::Get { key } => {
            println!("get called, {}", key);
            let query = db.get_by_key(key.as_ref());
            println!("{}", query.unwrap())
        }
        Command::Set { key, value } => {
            println!("set called, {}, {}", key, value);
            db.set_by_key(&key, &value).unwrap();
        }
    }
}
