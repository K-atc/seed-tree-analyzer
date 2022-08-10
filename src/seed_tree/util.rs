use super::file_hash::FileHash;

use sha1::{Digest, Sha1};
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

pub fn calc_file_hash<P: AsRef<Path>>(path: P) -> Result<FileHash, io::Error> {
    let mut file = BufReader::new(fs::File::open(path)?);
    let buf = file.fill_buf()?;
    let hash = Sha1::digest(buf);
    Ok(base16ct::lower::encode_string(&hash))
}

#[cfg(test)]
mod test {
    use super::calc_file_hash;
    use crate::seed_tree::FileHash;

    #[test]
    fn test_calc_file_hash() {
        let result = calc_file_hash("test/sample/seeds/fuzzer-test-suite-openssl-1.0.1f/0dafd00a785bd3d2cb36722c29f0dd23497833b0");
        assert!(result.is_ok(), "result={:?}", result);
        assert_eq!(
            result.unwrap(),
            FileHash::from("0dafd00a785bd3d2cb36722c29f0dd23497833b0")
        );
    }
}
