extern crate dirs;
extern crate serde;
extern crate serde_json;
extern crate sha3;

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    hash: String,
    exe: String,
    args: String,
    stdout: String,
    stderr: String,
    status: i32,
    filename: String,
    path: PathBuf,
}

impl Item {
    fn new(command: Vec<String>) -> Self {
        let hash = hashit(&command);
        let args = command[1..].join(" ");
        let exe = command[0].clone();
        let filename = format!("{}_{}.json", exe.clone(), hash.clone());
        let path = datadir().join(filename.clone());
        Self {
            hash: hash.clone(),
            exe: exe.clone(),
            args: args,
            filename: filename.clone(),
            path: path,
            stdout: "".to_string(),
            stderr: "".to_string(),
            status: 0,
        }
    }
    fn execute(&mut self) {
        let output = Command::new(self.exe.clone())
            .args(self.args.clone().split(" "))
            .output()
            .expect("command failed to execute");
        self.stdout = String::from_utf8(output.stdout).unwrap();
        self.stderr = String::from_utf8(output.stderr).unwrap();
        self.status = output.status.code().unwrap();
    }
    fn save(&self) {
        let db = datadir();
        mkdir(db);
        let file = File::create(&self.path).expect("could not create the file");
        serde_json::to_writer_pretty(&file, &self).expect("could write the file");
    }
    fn find_backup(&self) -> Option<Item> {
        if self.path.exists() {
            let file = File::open(&self.path).expect("failed to open old");
            let reader = BufReader::new(file);
            let u = serde_json::from_reader(reader).expect("cannot deserialize");
            Some(u)
        } else {
            None
        }
    }
    fn replay(&self) {
        println!("{}", self.stdout);
        eprintln!("{}", self.stderr);
    }
}

fn datadir() -> PathBuf {
    let home = dirs::home_dir().unwrap();
    home.join(".cache").join("ccache")
}

fn mkdir(s: PathBuf) {
    if !s.exists() {
        fs::create_dir(s).unwrap();
    }
}

fn hashit(v: &Vec<String>) -> String {
    let args = v.join(" ");
    let data = args.as_bytes();
    let hash = Sha3_256::digest(data);
    format!("{:x}", hash)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        process::exit(1);
    }

    let mut item = Item::new(args[1..].into());
    item.execute();
    eprintln!("{}  {}", item.status, item.hash);
    if item.status == 0 {
        item.save();
        item.replay();
        process::exit(0);
    }

    match item.find_backup() {
        Some(old) => old.replay(),
        None => item.replay(),
    }
}
