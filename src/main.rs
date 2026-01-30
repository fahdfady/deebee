use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::{collections::HashMap, path::Path};

// HashMap in-memory index buffer-of-start, buffer-of-end
struct Index(HashMap<u32, u32>);

impl Index {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// add an item to the index
    pub fn insert(mut self, x: u32, y: u32) {
        self.0.insert(x, y);
    }

    pub fn remove(mut self, x: &u32) {
        self.0.remove(x);
    }
}

struct Map(Vec<(String, String)>);

impl Map {
    fn new() -> Self {
        Self(Vec::new())
    }
}

struct Database {
    map: Map,
    file_path: String,
    idx: Index,
}

impl Database {
    pub fn new(file_path: &str) -> Self {
        // in-memory index
        let idx = Index::new();
        // check each line for data and take first and last line buffer number and store in index

        let file_path_path = Path::new(file_path);
        if !&file_path_path.exists() {
            File::create_new(file_path).expect("Couldnt' create database file");
        } else {
            // if the database is already there,
            // check if we have content,
            // if there is content --> scan the file line by line, first byte number and last byte
            // number goes into the index,
            // else -- > continue

            let file_content = fs::read_to_string(file_path_path).unwrap();

            if !file_content.is_empty() {
                let mut key: u32 = 0;

                for line in file_content.lines() {
                    // let v: u32 = (line.len() as u32) - 1 + key;
                    // let key = (line.len() as u32) + key;

                    let value = (line.len() as u32) + key;

                    println!("K: {}, V: {}", key, value);

                    key = value + 1;
                    // println!("{}", line.len());
                }
            } else {
                todo!();
            }
        }

        Self {
            map: Map::new(),
            file_path: file_path.to_string(),
            idx,
        }
    }

    pub fn get_by_key(self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(self.file_path).expect("couldn't read database");

        let mut result = String::new();

        for line in content.lines() {
            if line.contains(key) {
                result = line.to_string();
            }
        }

        Ok(result)
    }

    pub fn set_by_key(self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        // append to file with "key, value"
        let content = fs::read_to_string(self.file_path.clone()).expect("couldn't read database");

        let all_content = format!("{}\n{}", content, format!("{key}, {value}"));

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
    println!("dasdasdasdasda");

    let args = Args::parse();

    let db = Database::new("db.deebee");

    println!("{}", db.file_path);

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
