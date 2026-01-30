use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::{collections::HashMap, path::Path};

#[derive(Clone, Debug)]
// HashMap in-memory index buffer-of-start, buffer-of-end
// key is String because our key in the DB can be anything, not just a number
struct Index(HashMap<String, u32>);

impl Index {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// add an item to the index
    pub fn insert(&mut self, k: &str, v: u32) {
        self.0.insert(k.to_string(), v);
    }

    pub fn remove(&mut self, x: &str) {
        self.0.remove(x);
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
        // reverse the operation of reading line-by-line

        let mut result: Vec<(String, String)> = Vec::new();

        for line in file_content.lines() {
            // therfore, we must FORBID using commas `,` in either key or value in our database
            let parts_remove_whitespace = line.split_whitespace().collect::<String>();

            // every line has parts
            let parts = parts_remove_whitespace.split(",").collect::<Vec<&str>>();

            // the usage of a tup binding is a brute-force solution. must be refactored later
            result.push((parts[0].to_string(), parts[1].to_string()));

            // println!("{result:?}");
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
        // in-memory index
        // check each line for data and take first and last line buffer number and store in index
        let mut idx = Index::new();

        let map = Map::new(None);

        let file_path_path = Path::new(file_path);
        if !&file_path_path.exists() {
            File::create_new(file_path).expect("Couldnt' create database file");
        } else {
            // if the database is already there,
            // check if we have content,
            // if there is content --> scan the file line by line, first byte number and last byte
            // number goes into the index,
            // else -- > continue
            //
            // this should be probably delegated to Index struct in a method or something

            let file_content = fs::read_to_string(file_path_path).unwrap();

            if !file_content.is_empty() {
                let mut offset: u32 = 0;

                let map = map.clone().read_database(&file_content).unwrap();

                // get each key from the database and store it in the index HashMap

                let mut line_number: usize = 0;

                for line in file_content.lines() {
                    // let v: u32 = (line.len() as u32) - 1 + key;
                    // let key = (line.len() as u32) + key;

                    let key = map.get_key(line_number).expect("couldn't get key");

                    idx.insert(&key, offset);

                    offset += line.len() as u32;

                    line_number += 1;
                }

                println!("{:?}", idx.0);
            } else {
                todo!();
            }
        }

        Self {
            map,
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
