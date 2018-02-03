extern crate digest;
extern crate md5;

mod cache_buster {

    use md5::{Digest, Md5};
    use std::fs::File;
    use std::io::Read;
    use std::fmt::Write;
    use std::result::Result;

    fn hex_bytes(sum: &[u8]) -> String {
        let mut s = String::new();
        for &byte in sum {
            write!(&mut s, "{:X}", byte).expect("Unable to write");
        }
        s
    }

    pub fn hash_file(file_path: String) -> Result<String, String> {
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
}

#[cfg(test)]
mod tests {
    use cache_buster;
    #[test]
    fn it_works() {
        println!("{:?}", cache_buster::hash_file("Cargo.toml".to_string()));
        println!("\n");
        println!("{:?}", cache_buster::hash_file(".gitignore".to_string()));
    }
}
