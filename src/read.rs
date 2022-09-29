use std::path::PathBuf;
use std::fs::File;
use serde_json::{Result, Value};

extern crate dirs;

#[cfg(target_os = "macos")]
fn get_path(path: &mut PathBuf) -> PathBuf {
    path.push("Library/Application Support/Google/Chrome/Local State");
    path.to_path_buf()
}

#[cfg(target_os = "linux")]
fn get_path (path: &mut PathBuf) -> PathBuf {
    path.push(".config/google-chrome/Local State");
    path.to_path_buf()
}

pub fn run() {
    let mut path: PathBuf = dirs::home_dir().unwrap();
    let path = get_path(&mut path);
    let root = load_json(path);
    let root = root.unwrap();
    let info_cache = &root["profile"]["info_cache"];
    let mut counter: u32 = 0;
    for (key, value) in info_cache.as_object().unwrap() {
        counter += 1;
        println!("{}.{} ({})", counter, key, value["gaia_name"]);
    }
}

fn load_json(path: PathBuf) -> Result<Value> {
    // let data = fs::read_to_string(path).expect("cannot read");
    // let root : Value = serde_json::from_str(&data).expect("Error");
    let file = File::open(path).unwrap();
    let root: Value = serde_json::from_reader(file).unwrap();
    Ok(root)
}