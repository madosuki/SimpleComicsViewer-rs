use std::fs::File;
use std::io::Read;

use gtk::prelude::{WidgetExt, ImageExt};
use gdk_pixbuf;
use gdk_pixbuf::prelude::PixbufLoaderExt;

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

