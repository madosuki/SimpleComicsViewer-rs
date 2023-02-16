use std::fs::File;
use std::io::Read;
use std::cell::Cell;

use gtk::prelude::{WidgetExt, ImageExt};
use gdk_pixbuf;
use gdk_pixbuf::prelude::PixbufLoaderExt;

#[derive(Default, Clone)]
pub struct ImageContainer {
    gtk_image: gtk::Image,
    orig_width: Cell<i32>,
    orig_height: Cell<i32>,
}

pub trait ImageContainerEx {
    fn set_image_from_file(&self, path_str: &str);
    fn update_size_info(&self, width: i32, height: i32);
    fn get_image_ptr(&self) -> &gtk::Image;
    fn get_orig_width(&self) -> i32;
    fn get_orig_height(&self) -> i32;
}

impl ImageContainerEx for ImageContainer {
    fn set_image_from_file(&self, path_str: &str) {
        let Some(pixbuf_data) = create_pixbuf_from_file(path_str.to_string()) else { return };
        set_image_from_pixbuf(&self.gtk_image, &pixbuf_data);

        let width = pixbuf_data.width();
        let height = pixbuf_data.height();
        self.update_size_info(width, height);
    }

    fn update_size_info(&self, width: i32, height: i32) {
        self.orig_width.set(width);
        self.orig_height.set(height);
    }

    fn get_image_ptr(&self) -> &gtk::Image {
        &self.gtk_image
    }

    fn get_orig_width(&self) -> i32 {
        self.orig_width.get()
    }

    fn get_orig_height(&self) -> i32 {
        self.orig_height.get()
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


pub fn create_pixbuf_from_file(path_str: String) -> Option<gdk_pixbuf::Pixbuf> {
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

pub fn set_image_from_pixbuf(_image: &gtk::Image, _pixbuf_data: &gdk_pixbuf::Pixbuf) {
    _image.set_from_pixbuf(Some(_pixbuf_data));
    _image.set_vexpand(true);
}

