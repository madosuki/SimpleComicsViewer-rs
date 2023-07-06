use anyhow::Result;

use libarchive_extractor_rs;
use libarchive_extractor_rs::{ ArchiveExt, DecompressedData, FileInfo };
use libarchive_extractor_rs::error::LibArchiveResult;


use crate::utils;
use crate::image_container;
use image_container::ImageContainer;

pub struct ImageLoader {
    image_container_list: Vec<ImageContainer>,
}

pub trait ImageLoaderExt {
    fn load_from_compressed_file_to_memory(pathname: &str) -> Result<()>;
}

impl ImageLoaderExt for ImageLoader {
    fn load_from_compressed_file_to_memory(pathname: &str) -> Result<()> {
        let archive = libarchive_extractor_rs::Archive::new()?;

        let extracted: Vec<DecompressedData> = archive.extract_to_memory(pathname)?;
        // let _ = extracted.iter().map(|v| {
        // }).flatten();
        Ok(())
    }
}
