use anyhow::Result;

use libarchive_extractor_rs;
use libarchive_extractor_rs::{ ArchiveExt, DecompressedData };

use crate::utils;

pub fn load_from_compressed_file_to_memory(pathname: &str) -> Result<Vec<DecompressedData>> {
    let archive = libarchive_extractor_rs::Archive::new()?;

    let result: Vec<DecompressedData> = archive.extract_to_memory(pathname)?
        .into_iter().flat_map(|v| {
            match utils::detect_file_type_from_bytes(&v.value) {
                utils::FileType::PNG => Some(v),
                utils::FileType::JPG => Some(v),
                _ => None
            }
        }).collect();
    
    Ok(result)
}
