use gtk4 as gtk;

use gtk::prelude::{FileExt};

pub fn get_value_with_option_from_ref_cell_option<T, R, F>(data: &std::cell::RefCell<Option<T>>, f: F) -> Option<R>
where F: Fn(&T) -> R {
    match data.borrow().as_ref() {
        Some(v) => Some(f(v)),
        None => Option::None
    }
}

pub enum FileType {
    ZIP,
    SPANNED_ZIP,
    PNG,
    JPG,
    NONE,
}

pub fn detect_file_type_from_bytes(bytes: &[u8]) -> FileType {
    if bytes.len() < 4 {
        return FileType::NONE;
    }

    let first = bytes[0];
    let second = bytes[1];
    let third = bytes[2];
    let fourth = bytes[3];

    if first == 0xFF && second == 0xD8 && third == 0xFF {
        return FileType::JPG;
    }

    if first == 0x89 && second == 0x50 && third == 0x4E && fourth == 0x47 {
        return FileType::PNG;
    }

    if first == 0x50 && second == 0x4B && third == 0x3 && fourth == 0x4 {
        if third == 0x3 && fourth == 0x4 {
            return FileType::ZIP;
        }

        if third == 0x7 && fourth == 0x8 {
            return FileType::SPANNED_ZIP;
        }
    }

    FileType::NONE
}

pub fn detect_file_type_from_file(file: &gio::File) -> FileType {
    let Ok((bytes, _s)) = file.load_bytes(gio::Cancellable::NONE) else {
        return FileType::NONE
    };

    let tmp = bytes.to_vec();
    detect_file_type_from_bytes(&tmp)
}

