use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn read_file_content<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    fs::read_to_string(path)
}

pub fn read_file_bytes<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}

// pub fn get_path_list_of_all_file_and_dir_in_one_dir<P: AsRef<Path>>(
//     path: P,
// ) -> std::io::Result<Vec<String>> {
//     let mut path_list: Vec<String> = Vec::new();
//     let list = fs::read_dir(path).unwrap();
//     for entry in list {
//         let entry = entry?;
//         let path = entry.path();
//         path_list.push(path.to_str().unwrap().to_string());
//     }
//     Ok(path_list)
// }

pub fn get_current_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}
