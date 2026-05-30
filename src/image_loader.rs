use anyhow::Result;

use libarchive_extractor_rs;
use libarchive_extractor_rs::{ArchiveExt, DecompressedData};

use crate::natural_sort::compare_by_natural;
use crate::utils;

pub fn load_from_compressed_file_to_memory(pathname: &str) -> Result<Vec<DecompressedData>> {
    let archive = libarchive_extractor_rs::Archive::new()?;

    let mut tmp: Vec<DecompressedData> = archive
        .extract_to_memory(pathname)?
        .into_iter()
        .flat_map(|v| match utils::detect_file_type_from_bytes(&v.value) {
            utils::FileType::PNG => Some(v),
            utils::FileType::JPG => Some(v),
            _ => None,
        })
        .collect();

    tmp.sort_by(|a, b| {
        let a_name = a.file_info.file_name.clone();
        let b_name = b.file_info.file_name.clone();
        return compare_by_natural(&a_name, &b_name);
    });

    Ok(tmp)
}
