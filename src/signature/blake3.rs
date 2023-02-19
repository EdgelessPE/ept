use anyhow::Result;
use blake3::{hash, Hasher};
use std::fs::File;
use std::io;

use crate::log;

pub fn compute_hash_blake3(from_file: String) -> Result<String> {
    let file = File::open(&from_file)?;
    let mut hasher = Hasher::new();
    if let Some(mmap) = try_into_memmap_file(&file)? {
        hasher.update_rayon(mmap.get_ref());
    } else {
        copy_wide(file, &mut hasher)?;
    }
    let hash = hasher.finalize();
    let hash = hash.to_hex().to_string();
    log!(
        "Debug:Calculated blake3 hash for '{}' : '{}'",
        &from_file,
        &hash
    );
    Ok(hash)
}

pub fn fast_compute_hash_blake3(raw: &Vec<u8>) -> Result<String> {
    let hash = hash(raw);
    let hash = hash.to_hex().to_string();
    log!("Debug:Got blake3 hash : '{}'", &hash);
    Ok(hash)
}

fn try_into_memmap_file(file: &File) -> anyhow::Result<Option<io::Cursor<memmap2::Mmap>>> {
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    Ok(if !metadata.is_file() {
        None
    } else if file_size > isize::max_value() as u64 {
        None
    } else if file_size == 0 {
        None
    } else if file_size < 16 * 1024 {
        None
    } else {
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .len(file_size as usize)
                .map(file)?
        };

        Some(io::Cursor::new(mmap))
    })
}

fn copy_wide(mut reader: impl io::Read, hasher: &mut blake3::Hasher) -> io::Result<u64> {
    let mut buffer = [0; 65536];
    let mut total = 0;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(total),
            Ok(n) => {
                hasher.update(&buffer[..n]);
                total += n as u64;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

#[test]
fn test_compute_hash_blake3() {
    let res = compute_hash_blake3(
        r"D:\Desktop\Projects\EdgelessPE\ept\VSCode_1.75.0.0_Cno.nep\c1\VSCode_1.75.0.0_Cno.nep"
            .to_string(),
    );
    println!("{:?}", res);
}
