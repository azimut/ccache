extern crate serde;
extern crate serde_json;
extern crate sha3;

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::env;
// use std::fs;
use std::fs::File;
// use std::path::Path;
use std::process;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    hash: String,
    exe: String,
    args: String,
    stdout: String,
    stderr: String,
    status: u8,
    filename: String,
}

fn hashit(v: &Vec<String>) -> String {
    let args = v[1..].join(" ");
    let data = args.as_bytes();
    let hash = Sha3_256::digest(data);
    format!("{:x}", hash)
}

impl Item {
    fn new(command: Vec<String>) -> Self {
        let hash = hashit(&command);
        let args = command[1..].join(" ");
        let exe = command[0].clone();
        Self {
            hash: hash.clone(),
            exe: exe.clone(),
            args: args,
            stdout: "NO OUTPUT".to_string(),
            stderr: "NO OUTPUT".to_string(),
            status: 99,
            filename: format!("{}_{}.json", exe.clone(), hash.clone()),
        }
    }
    fn execute(&mut self) {
        let output = Command::new(self.exe.clone())
            .args(self.args.clone().split(" "))
            .output()
            .expect("command failed to execute");
        self.stdout = String::from_utf8(output.stdout).unwrap();
        self.stderr = String::from_utf8(output.stderr).unwrap();
        println!("{:?}", output.status);
    }
    fn save(&self) {
        let file = File::create(&self.filename).expect("could not create the file");
        serde_json::to_writer_pretty(&file, &self).expect("could write the file");
    }
}

// fn mkdir(s: &str) {
//     if !Path::new(s).is_dir() {
//         fs::create_dir(s).unwrap();
//     }
// }

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        process::exit(1);
    }
    let command = args[1..].into();
    let mut item = Item::new(command);
    println!("{}", item.hash.clone());
    item.execute();
    item.save();
}
