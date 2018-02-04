extern crate clap;
extern crate digest;
extern crate glob;
extern crate md5;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod cache_buster {

    use std::vec::Vec;
    use std::collections::HashMap;
    use serde_json::{Error, Value};
    use serde_json;
    use std::result::Result;
    use md5::{Digest, Md5};
    use std::fs::File;
    use std::fs;
    use std::io::Read;
    use std::fmt::Write;
    use std::path::Path;
    use std::path::PathBuf;
    use glob::glob;

    fn hex_bytes(sum: &[u8]) -> String {
        let mut s = String::new();
        for &byte in sum {
            write!(&mut s, "{:X}", byte).expect("Unable to write");
        }
        s
    }

    const DEBUG: bool = false;

    type PathDict = HashMap<String, String>;

    pub fn hash_file(file_path: &str) -> Result<String, String> {
        const BUFFER_SIZE: usize = 1024;
        match File::open(file_path) {
            Ok(mut file) => {
                let mut hasher = Md5::default();
                let mut buffer = [0u8; BUFFER_SIZE];
                loop {
                    let n = match file.read(&mut buffer) {
                        Ok(n) => n,
                        Err(e) => return Err(e.to_string()),
                    };
                    hasher.input(&buffer[..n]);
                    if n == 0 || n < BUFFER_SIZE {
                        break;
                    }
                }
                Ok(hex_bytes(&hasher.result()))
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn hash_and_copy(
        pconfig: &ProcessedConfig,
        acc: &mut PathDict,
        root: &Path,
        origin_path: &Path,
    ) {
        // simplify this copying mess
        let asset_path = pconfig.asset_path;
        let origin_buffer = origin_path.to_path_buf();
        let origin_buffer_1 = origin_buffer.clone();
        let origin_copy = origin_buffer_1.as_path();
        let origin_buffer_2 = origin_buffer.clone();
        let origin_copy_2 = origin_buffer_2.as_path();
        if let Some(path_str) = origin_buffer.to_str() {
            if let Some(file_name) = origin_copy.file_stem() {
                if let Some(file_name) = file_name.to_str() {
                    let mut file_name = file_name.to_string();
                    let mut target_path = PathBuf::new();
                    target_path.push(&root);
                    match hash_file(path_str) {
                        Ok(hash) => {
                            file_name.push_str(".");
                            file_name.push_str(&hash);
                            if let Some(extension) = origin_copy.extension() {
                                if let Some(extension) = extension.to_str() {
                                    file_name.push_str(".");
                                    file_name.push_str(extension);
                                }
                            }
                            target_path.push(file_name);
                        }
                        _ => (),
                    };
                    if let Some(parent) = target_path.parent() {
                        match fs::create_dir_all(parent) {
                            Ok(_) => (),
                            Err(e) => print!("{:?}", e.to_string()),
                        }
                    }
                    if let Some(origin_path_str) = origin_copy_2.to_str() {
                        let origin_path_kv = origin_path.clone();
                        let target_path_kv = target_path.clone();
                        let mut relative_origin_path_kv = origin_path.clone();
                        let mut relative_target_path_kv = target_path.clone();
                        if let Some(asset_path) = asset_path {
                            if DEBUG {
                                println!("asset_path");
                                println!("{:?}", asset_path);
                            };
                            match origin_path_kv.strip_prefix(asset_path) {
                                Ok(relative_origin_path) => {
                                    relative_origin_path_kv = relative_origin_path;
                                }
                                _ => (),
                            }
                            match target_path_kv.strip_prefix(pconfig.original_root_path) {
                                Ok(target_path_kv) => match target_path_kv.strip_prefix(asset_path)
                                {
                                    Ok(relative_target_path) => {
                                        relative_target_path_kv =
                                            relative_target_path.to_path_buf();
                                    }
                                    _ => (),
                                },
                                _ => (),
                            }
                        }
                        if let Some(target_path_str) = target_path.to_str() {
                            if let Some(relative_origin_path_str) = relative_origin_path_kv.to_str()
                            {
                                if let Some(relative_target_path_str) =
                                    relative_target_path_kv.to_str()
                                {
                                    if DEBUG {
                                        println!("origin");
                                        println!("{}", relative_origin_path_str);
                                        println!("target");
                                        println!("{}", relative_target_path_str);
                                    };
                                    acc.insert(
                                        String::from(relative_origin_path_str),
                                        String::from(relative_target_path_str),
                                    );
                                }
                            }
                            fs::copy(origin_path_str, target_path_str);
                        }
                    }
                }
            };
        }
    }

    pub fn hash_and_copy_dir(
        pconfig: &ProcessedConfig,
        acc: &mut PathDict,
        root: &Path,
        dir: &Path,
    ) {
        match dir.read_dir() {
            Ok(read_dir) => for dir_entry in read_dir {
                match dir_entry {
                    Ok(dir_entry) => {
                        let path_buf = dir_entry.path();
                        let path = path_buf.as_path();
                        if let Some(new_parent) = path.parent() {
                            let root_buf = root.join(new_parent);
                            let new_root = root_buf.as_path();
                            if path.is_dir() {
                                hash_and_copy_dir(pconfig, acc, &new_root, path);
                            } else {
                                hash_and_copy(pconfig, acc, &new_root, path);
                            }
                        }
                    }
                    _ => (),
                }
            },
            _ => (),
        }
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Config {
        target_path: String,
        patterns: Vec<String>,
        dictionary: String,
        asset_path: Option<String>,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct ConfigFile {
        cache_buster: Config,
    }

    fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
        match File::open(path) {
            Ok(file) => {
                let v: Result<ConfigFile, serde_json::Error> = serde_json::from_reader(file);
                match v {
                    Ok(config) => Ok(config.cache_buster),
                    Err(e) => Err(e.to_string()),
                }
            }
            _ => Err("can't open file".to_string()),
        }
    }

    #[derive(Debug, Clone)]
    pub struct ProcessedConfig<'a> {
        target_path: &'a Path,
        patterns: Vec<String>,
        dictionary: &'a Path,
        asset_path: Option<&'a Path>,
        original_root_path: &'a Path,
    }

    fn process_config<'a>(config: &'a Config) -> ProcessedConfig<'a> {
        // why did I have to do it this way?
        // can't nest the match clause inside of ProcessedConfig { asset_path: ... } because of the borrow checker
        match config.asset_path {
            Some(ref asset_path_string) => ProcessedConfig {
                target_path: Path::new(&config.target_path),
                patterns: config.patterns.clone(),
                dictionary: Path::new(&config.dictionary),
                asset_path: Some(Path::new(asset_path_string)),
                original_root_path: Path::new(&config.target_path),
            },
            None => ProcessedConfig {
                target_path: Path::new(&config.target_path),
                patterns: config.patterns.clone(),
                dictionary: Path::new(&config.dictionary),
                asset_path: None,
                original_root_path: Path::new(&config.target_path),
            },
        }
    }

    pub fn fingerprint_and_copy(config_path: &str) {
        let mut generated_paths = HashMap::new();
        match read_config(config_path) {
            Ok(config) => {
                let pconfig = process_config(&config);
                for pattern in &config.patterns {
                    for entry in glob(pattern).expect("Failed to read glob pattern") {
                        match entry {
                            Ok(origin_path) => {
                                if origin_path.is_dir() {
                                    // recur and do whatever you were going to do for a file
                                    hash_and_copy_dir(
                                        &pconfig,
                                        &mut generated_paths,
                                        pconfig.original_root_path,
                                        &origin_path,
                                    );
                                } else {
                                    if let Some(new_parent) = origin_path.parent() {
                                        let root_buf = pconfig.original_root_path.join(new_parent);
                                        let new_root = root_buf.as_path();
                                        hash_and_copy(
                                            &pconfig,
                                            &mut generated_paths,
                                            new_root,
                                            &origin_path,
                                        );
                                    }
                                }
                            }
                            Err(e) => println!("{:?}", e),
                        }
                    }
                }
                // write generated_paths to file
                match File::create(&config.dictionary) {
                    Ok(mut output_file) => {
                        serde_json::to_writer(output_file, &generated_paths);
                    }
                    _ => (),
                }
            }
            Err(err) => {
                print!("{:?}", err);
            }
        }
    }
}

use clap::{App, Arg};

fn main() {
    let matches = App::new("cache_buster")
        .version("0.1.0")
        .author("Sebastian Bensusan <sbensu@gmail.com>")
        .about("Adds content hashing to file names to ensure HTTP protocols cache them")
        .arg(
            Arg::with_name("config")
                .short("c")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("file that contains the config"),
        )
        .get_matches();
    let config_file = matches.value_of("config").unwrap_or("package.json");
    cache_buster::fingerprint_and_copy(&config_file);
}

#[cfg(test)]
mod tests {
    use cache_buster;

    #[test]
    fn find_md5_hash() {
        assert_eq!(
            Ok("D41D8CD98F0B24E980998ECF8427E".to_string()),
            cache_buster::hash_file("examples/empty_file.js")
        );
        assert_eq!(
            Ok("C0F781B05E475681EAF474CB242F".to_string()),
            cache_buster::hash_file("examples/full_file.css")
        );
    }
}
