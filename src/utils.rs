use gtk4 as gtk;
use gtk::gio;
use gtk::prelude::Cast;

use gtk::gdk::{prelude::DisplayExt};

use gtk::prelude::FileExt;
use gtk::prelude::MonitorExt;

pub fn get_current_unixtime() -> Option<u64> {
    if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Some(duration.as_secs())
    } else {
        None
    }
}

pub fn get_xdg_config_home() -> String {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        dir
    } else {
        "/home/user/.config/simple_comics_viewer".to_owned()
    }
}

pub fn get_value_with_option_from_ref_cell_option<T, R, F>(
    data: &std::cell::RefCell<Option<T>>,
    f: F,
) -> Option<R>
where
    F: Fn(&T) -> R,
{
    match data.borrow().as_ref() {
        Some(v) => Some(f(v)),
        None => Option::None,
    }
}

pub fn get_dpi() -> f32 {
    let mut ppi = 144.0f32;
    let display = gtk::gdk::Display::default().unwrap();
    let monitors = display.monitors();
    for monitor in monitors.into_iter() {
        let pre_m = monitor.expect("faild get monitor from monitors iter.");
        let m = pre_m.downcast::<gtk::gdk::Monitor>().expect("failed downcast from object to Monitor");
        let geometry = m.geometry();
        let width_pixel = geometry.width();
        let height_pixel = geometry.height();

        let width_mm = m.width_mm();
        let height_mm = m.height_mm();

        let inch = 25.4f32;

        if width_pixel < 1 || width_mm < 1 {
            break;
        }

        if height_pixel < 1 || height_mm < 1 {
            break;
        }

        let diagonal_mm = (((width_mm ^ 2) + (height_mm ^ 2)) as f32).sqrt();
        let diagonal_pixel = (((width_pixel ^ 2) + (height_pixel ^ 2)) as f32).sqrt();
        let tmp_ppi = diagonal_pixel / (diagonal_mm / inch);

        if tmp_ppi > ppi {
            ppi = tmp_ppi;
        }
    }

    ppi
}

pub enum FileType {
    ZIP,
    SpannedZip,
    PNG,
    JPG,
    PDF,
    NONE,
}

pub fn detect_file_type_from_bytes(bytes: &[u8]) -> FileType {
    if bytes.len() < 5 {
        return FileType::NONE;
    }

    let first = bytes[0];
    let second = bytes[1];
    let third = bytes[2];
    let fourth = bytes[3];
    let fifth = bytes[4];

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
            return FileType::SpannedZip;
        }
    }

    if first == 0x25 && second == 0x50 && third == 0x44 && fourth == 0x46 && fifth == 0x2D {
        return FileType::PDF;
    }

    FileType::NONE
}

pub fn detect_file_type_from_file(file: &gio::File) -> FileType {
    let Ok((bytes, _s)) = file.load_bytes(gio::Cancellable::NONE) else {
        return FileType::NONE;
    };

    let tmp = bytes.to_vec();
    detect_file_type_from_bytes(&tmp)
}
