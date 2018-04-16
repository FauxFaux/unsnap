use std::fs;
use std::io::Read;
use std::path::Path;

use errors::*;

pub fn load_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(200);
    fs::File::open(path)?.read_to_end(&mut buf)?;
    Ok(buf)
}
