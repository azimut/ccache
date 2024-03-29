use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Item {
    args: String,
    exe: String,
    filename: String,
    hash: String,
    status: i32,
    stderr: String,
    stdout: String,
}

impl Item {
    fn new(command: Vec<String>) -> Self {
        let hash = hashit(&command);
        let args = command[1..].join(" ");
        let exe = command[0].clone();
        let filename = format!("{}_{}.json.gz", exe, hash);
        Self {
            args,
            exe,
            filename,
            hash,
            ..Default::default()
        }
    }
    fn path(&self) -> PathBuf {
        datadir().join(&self.filename)
    }
    fn execute(&mut self) {
        let output = Command::new(self.exe.clone())
            .args(self.args.clone().split(' '))
            .output()
            .expect("command failed to execute");
        self.stdout = String::from_utf8(output.stdout).unwrap();
        self.stderr = String::from_utf8(output.stderr).unwrap();
        self.status = output.status.code().unwrap();
    }
    fn save(&self) {
        let data = serde_json::to_vec_pretty(&self).expect("failed to encode");
        let f = File::create(self.path()).expect("could not create new file");
        let mut gz = GzEncoder::new(&f, Compression::default());
        gz.write_all(&data[..]).expect("could not write");
        gz.finish().expect("could not finish to write");
    }
    fn find_backup(&self) -> Option<Item> {
        if !self.path().exists() {
            return None;
        }
        let file = File::open(self.path()).expect("failed to open old");
        let gz = GzDecoder::new(file);
        let u = serde_json::from_reader(gz).expect("cannot deserialize");
        Some(u)
    }
    fn replay(&self) {
        print!("{}", self.stdout);
        eprint!("{}", self.stderr);
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

fn hashit(v: &[String]) -> String {
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

    mkdir(datadir());

    let mut item = Item::new(args[1..].into());
    item.execute();

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
