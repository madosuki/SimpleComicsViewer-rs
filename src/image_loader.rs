use std::fs::File;
use std::io::Read;
use std::cell::Cell;
use std::cell::RefCell;

use gtk4 as gtk;

use gtk::prelude::{WidgetExt, FileExt};
use gdk_pixbuf;
use gdk_pixbuf::prelude::PixbufLoaderExt;

enum PictureDirectionType {
    Vertical,
    Horizontal,
    Square
}

#[derive(Default, Clone)]
pub struct ImageContainer {
    pixbuf_data_for_modify: RefCell<Option<gdk_pixbuf::Pixbuf>>,
    orig_pixbuf_data: RefCell<Option<gdk_pixbuf::Pixbuf>>,
}

#[derive(Default)]
pub struct AspectRatioCollection {
    for_width: f64,
    for_height: f64,
}

pub trait ImageContainerEx {
    fn set_pixbuf_from_file(&self, file: &gio::File, window_width: i32, window_height: i32);
    fn get_modified_pixbuf_data(&self) -> Option<gdk_pixbuf::Pixbuf>;
    fn get_orig_width(&self) -> i32;
    fn get_orig_height(&self) -> i32;
    fn scale(&self, target_width: i32, target_height: i32);
}

impl ImageContainerEx for ImageContainer {
    fn get_modified_pixbuf_data(&self) -> Option<gdk_pixbuf::Pixbuf> {
        let Some(v) = self.pixbuf_data_for_modify.borrow().clone() else {
            return None;
        };

        Some(v)
    }

    fn set_pixbuf_from_file(&self, file: &gio::File, window_width: i32, window_height: i32) {
        let Some(pixbuf_data) = create_pixbuf_from_file(file) else { return };

        let _ = self.pixbuf_data_for_modify.replace_with(|_| Some(pixbuf_data.clone()));

        let _ = self.orig_pixbuf_data.replace_with(|_| Some(pixbuf_data.clone()));

    }

    fn get_orig_width(&self) -> i32 {
        let tmp = self.orig_pixbuf_data.borrow();
        let Some(v) = tmp.as_ref() else {
            return -1;
        };

        v.width()
    }

    fn get_orig_height(&self) -> i32 {
        let tmp = self.orig_pixbuf_data.borrow();
        let Some(v) = tmp.as_ref() else {
            return -1;
        };

        v.height()
    }

    fn scale(&self, target_width: i32, target_height: i32) {
        if target_width < 1 || target_height < 1 {
            return;
        }
        
        let Some(pixbuf_data) = self.orig_pixbuf_data.borrow().clone() else { return };

        let width = pixbuf_data.width() as f64;
        let height = pixbuf_data.height() as f64;

        let mut picture_direction: PictureDirectionType = PictureDirectionType::Square;
        if width < height {
            picture_direction = PictureDirectionType::Vertical;
        }
        if height < width {
            picture_direction = PictureDirectionType::Horizontal;
        }

        let tmp_target_width = target_width as f64;
        let tmp_target_height = target_height as f64;

        let aspect_ratio = calc_aspect_raito(width, height);
        let mut result_height: i32 = 0;
        let mut result_width: i32 = 0;

        match picture_direction {
            PictureDirectionType::Vertical => {
                result_height = target_height;
                result_width = (tmp_target_height / aspect_ratio.for_height).ceil() as i32;
                println!("scaled! {}, {}", result_width, result_height);
            },
            PictureDirectionType::Horizontal => {
                result_width = target_width;
                result_height = (tmp_target_width / aspect_ratio.for_width).ceil() as i32;
            },
            PictureDirectionType::Square => {
                result_height = target_height;
                result_width = (tmp_target_height / aspect_ratio.for_height).ceil() as i32;
            }
        }

        let Some(scaled) = pixbuf_data.scale_simple(result_width, result_height, gdk_pixbuf::InterpType::Bilinear) else { return };
        let _ = self.pixbuf_data_for_modify.replace_with(|_| Some(scaled.clone()));
    }
}

pub fn read_bytes_from_file(path_str: &str) -> Option<Vec<u8>> {
    let path = Some(std::path::Path::new(path_str)).unwrap();
    let mut f = File::open(path).unwrap();
    let mut buf: Vec<u8> = vec!();
    match f.read_to_end(&mut buf) {
        Ok(_) => Some(buf),
        _ => None,
    }
}

pub fn create_pixbuf_from_bytes(bytes: &[u8]) -> Option<gdk_pixbuf::Pixbuf> {
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    let result_of_pixbuf_loader_write = pixbuf_loader.write(bytes);
    if result_of_pixbuf_loader_write.is_err() { return None }

    let Some(pixbuf_data) = pixbuf_loader.pixbuf() else {
        return None;
    };

    let result_of_loder_close = pixbuf_loader.close();
    if result_of_loder_close.is_err() { return None }

    Some(pixbuf_data)
}


pub fn create_pixbuf_from_file_path(path_str: String) -> Option<gdk_pixbuf::Pixbuf> {
    let Some(buf) = read_bytes_from_file(&path_str) else {
        return None
    };

    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    let result_of_pixbuf_loader_write = pixbuf_loader.write(&buf);
    if result_of_pixbuf_loader_write.is_err() { return None };

    let Some(pixbuf_data) = pixbuf_loader.pixbuf() else {
        return None
    };

    let result_of_loader_close = pixbuf_loader.close();
    if result_of_loader_close.is_err() {
        return None;
    }
                
    Some(pixbuf_data)
}

pub fn create_pixbuf_from_file(file: &gio::File) -> Option<gdk_pixbuf::Pixbuf> {
    let Ok((bytes, s)) = file.load_bytes(gio::Cancellable::NONE) else {
        return None
    };

    let buf: Vec<u8> = bytes.to_vec();
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    let result_of_pixbuf_loader_write = pixbuf_loader.write(&buf);
    if result_of_pixbuf_loader_write.is_err() { return None };

    let Some(pixbuf_data) = pixbuf_loader.pixbuf() else {
        return None
    };

    let result_of_loader_close = pixbuf_loader.close();
    if result_of_loader_close.is_err() {
        return None;
    }
                
    Some(pixbuf_data)
}


fn calc_aspect_raito(width: f64, height: f64) -> AspectRatioCollection {
    let for_width: f64 = width / height;
    let for_height: f64 = height / width;

    AspectRatioCollection {
        for_width: for_width,
        for_height: for_height,
    }
}
