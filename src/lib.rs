extern crate digest;
extern crate glob;
extern crate md5;

mod cache_buster {

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

    pub fn hash_and_copy(root: &Path, origin_path: &Path) {
        // simplify this copying mess
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
                        print!("{:?}", parent);
                        match fs::create_dir_all(parent) {
                            Ok(_) => (),
                            Err(e) => print!("{:?}", e.to_string()),
                        }
                    }
                    fs::copy(origin_copy_2, target_path);
                }
            };
        }
    }

    pub fn hash_and_copy_dir(root: &Path, dir: &Path) {
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
                                hash_and_copy_dir(&new_root, path);
                            } else {
                                hash_and_copy(&new_root, path);
                            }
                        }
                    }
                    _ => (),
                }
            },
            _ => (),
        }
    }

    pub fn list_dir() {
        let root = Path::new("target");
        for entry in glob("examples/*").expect("Failed to read glob pattern") {
            match entry {
                Ok(origin_path) => {
                    if origin_path.is_dir() {
                        // recur and do whatever you were going to do for a file
                        hash_and_copy_dir(root, &origin_path);
                    } else {
                        if let Some(new_parent) = origin_path.parent() {
                            let root_buf = root.join(new_parent);
                            let new_root = root_buf.as_path();
                            hash_and_copy(new_root, &origin_path);
                        }
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cache_buster;

    #[test]
    fn find_md5_hash() {
        assert_eq!(
            Ok("D41D8CD98F0B24E980998ECF8427E".to_string()),
            cache_buster::hash_file("examples/empty_file.txt")
        );
        assert_eq!(
            Ok("C0F781B05E475681EAF474CB242F".to_string()),
            cache_buster::hash_file("examples/fib-5.txt")
        );
        cache_buster::list_dir();
    }
}
