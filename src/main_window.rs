use gtk4 as gtk;
use gdk4 as gdk;

use gtk::prelude::{ApplicationExt, ApplicationWindowExt, GtkApplicationExt, GtkWindowExt, WidgetExt, DialogExt, FileChooserExt, FileChooserExtManual,  MenuModelExt, BoxExt, DrawingAreaExt, DrawingAreaExtManual, SurfaceExt, GdkCairoContextExt, PopoverExt, ActionMapExtManual, FileExt};
use gtk::{Application, ApplicationWindow, Button, Allocation, DrawingArea, cairo, PopoverMenu, gio, glib, EventControllerKey};
use gdk_pixbuf;
use gdk_pixbuf::prelude::PixbufLoaderExt;

use crate::image_container;
use crate::image_loader;
use crate::utils;
use image_container::{ImageContainer, ImageContainerEx};

#[derive(Default)]
struct PagesInfo {
    current_page_index: std::rc::Rc<std::cell::RefCell<usize>>,
    loaded_filename: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    loaded_dirname: std::rc::Rc<std::cell::RefCell<Option<String>>>,
}

#[derive(Default)]
struct Settings {
    is_dual_mode: std::rc::Rc<std::cell::RefCell<bool>>,
}

struct MainWindow {
    window: ApplicationWindow,
    v_box: gtk::Box,
    image_container_list: std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>,
    pages_info: std::rc::Rc<PagesInfo>,
    settings: std::rc::Rc<Settings>,
}

fn calc_margin_for_single(pixbuf_data: &gdk_pixbuf::Pixbuf, target_width: i32, target_height: i32) -> i32 {
    let pic_height = pixbuf_data.height();
    let pic_width = pixbuf_data.width();

    let diff = target_width - pic_width;


    if diff < 0 {
        return -1;
    }

    if diff == 0 {
        diff
    } else {
        diff / 2
    }
}

fn scale_page_for_single(image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, current_page_index: usize, target_width: i32, target_height: i32) {

    if (target_width < 1) || (target_height < 1) {
        return;
    }
    
    if image_container_list.borrow().is_empty() {
        return;
    }

    if current_page_index >= image_container_list.borrow().len() {
        return;
    }

    
    image_container_list.borrow()[current_page_index].scale(target_width, target_height, false);
}

fn scale_page_for_dual(image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, current_page_index: usize, target_width: i32, target_height: i32) {

    if (target_width < 1) || (target_height < 1) {
        return;
    }
    
    if image_container_list.borrow().is_empty() {
        return;
    }

    let next_index = current_page_index + 1;
    let _image_container_list_len = image_container_list.borrow().len();

    let final_target_width = target_width / 2;
    image_container_list.borrow()[current_page_index].scale(final_target_width, target_height, true);

    if next_index != _image_container_list_len {
        image_container_list.borrow()[next_index].scale(final_target_width, target_height, true);
    }
}



fn set_page_from_file_for_single(file: &gio::File, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, page_index: usize, width: i32, height: i32) {
    let _image_container = ImageContainer::default();
    _image_container_list.borrow_mut().push(_image_container);

    _image_container_list.borrow()[page_index].set_pixbuf_from_file(file, width, height);
    _image_container_list.borrow()[page_index].scale(width, height, false);
}

fn set_page_from_bytes_for_single(bytes: &[u8], _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, page_index: usize, width: i32, height: i32) {
    let _image_container = ImageContainer::default();

    _image_container_list.borrow_mut().push(_image_container);
    
    _image_container_list.borrow()[page_index].set_pixbuf_from_bytes(bytes, width, height);
    _image_container_list.borrow()[page_index].scale(width, height, false);
}

fn set_page_from_bytes_for_dual(bytes: &[u8], _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, page_index: usize, width: i32, height: i32) {
    let _image_container = ImageContainer::default();

    _image_container_list.borrow_mut().push(_image_container);
    
    _image_container_list.borrow()[page_index].set_pixbuf_from_bytes(bytes, width, height);

    let half_width = width / 2;
    _image_container_list.borrow()[page_index].scale(half_width, height, true);
}


fn open_and_set_image_from_zip(file: &gio::File, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, _drawing_area_ref: &DrawingArea) {
    let Some(_pathbuf) = file.path() else { return; };
    let Some(_pathname) = _pathbuf.as_path().to_str() else {
        return;
    };

    let _extracted = image_loader::load_from_compressed_file_to_memory(_pathname).unwrap();
    let mut count = 0;
    _extracted.into_iter().for_each(|v| {
        // set_page_from_bytes_for_single(&v.value, &_image_container_list, count, _drawing_area_ref.allocated_width(), _drawing_area_ref.allocated_height());
        set_page_from_bytes_for_dual(&v.value, &_image_container_list, count, _drawing_area_ref.allocated_width(), _drawing_area_ref.allocated_height());
        count = count + 1;
    });
}

fn open_and_set_image(file: &gio::File, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, _drawing_area_ref: &DrawingArea, page_index: usize) {
    println!("page index: {}", &page_index);
    match utils::detect_file_type_from_file(&file) {
        utils::FileType::NONE => { return; },
        _ => {
            set_page_from_file_for_single(&file, &_image_container_list, page_index, _drawing_area_ref.allocated_width(), _drawing_area_ref.allocated_height());
        }
    };
}


fn create_action_entry_for_menu(_window: &gtk::ApplicationWindow,
                                _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>,
                                _pages_info: &std::rc::Rc<PagesInfo>,
                                _drawing_area_ref: &DrawingArea) -> gio::ActionEntry<gtk::Application> {
        let _action_entry: gio::ActionEntry<gtk::Application> = gio::ActionEntry::builder("file_open")
            .activate(glib::clone!(@weak _window, @strong _image_container_list, @strong _pages_info, @strong _drawing_area_ref => move |_app: &gtk::Application, _action: &gio::SimpleAction, _user_data: Option<&glib::Variant>| {
                let dialog = gtk::FileChooserDialog::new(Some("File Select"),
                                                         Some(&_window),
                                                         gtk::FileChooserAction::Open,
                &[("Open", gtk::ResponseType::Ok), ("Cancel", gtk::ResponseType::Cancel)]);

                dialog.connect_response(glib::clone!(@strong _image_container_list, @strong _pages_info, @strong _drawing_area_ref => move |file_dialog, response| {
                    if response == gtk::ResponseType::Ok {
                        println!("ok");
                        let Some(file) = file_dialog.file() else { return };
                        let Some(_path) = file.path() else { return };
                        if !_path.is_file() { return; }

                        let is_zip =
                            match utils::detect_file_type_from_file(&file) {
                                utils::FileType::ZIP => true,
                                _ => false
                            };

                        _image_container_list.borrow_mut().clear();
                        _pages_info.current_page_index.replace(0);
                        if is_zip {
                            open_and_set_image_from_zip(&file, &_image_container_list, &_drawing_area_ref);
                        } else {
                            let Some(_dir) = _path.parent() else {
                                open_and_set_image(&file, &_image_container_list, &_drawing_area_ref, 0);
                                _drawing_area_ref.queue_draw();
                                return;
                            };

                            let mut count: usize = 0;
                            println!("{}", _dir.display());
                            for entry in _dir.read_dir().expect("read_dir call failed") {
                                if let Ok(entry) = entry {
                                    if entry.file_type().unwrap().is_file() {
                                        let tmp_path = entry.path();
                                        let tmp_file = gio::File::for_path(&tmp_path);
                                        open_and_set_image(&tmp_file, &_image_container_list, &_drawing_area_ref, count);
                                        count = count + 1;
                                        println!("{:?}", entry.path());
                                    }
                                }
                            }
                        }
                        // println!("drawing area allocated height: {}", _drawing_area_ref.allocated_height());
                        _drawing_area_ref.queue_draw();

                    }
                    file_dialog.close();
                }));
            
                dialog.show();
            }))
        .build();

    _action_entry
}

fn draw_single_page(_image_container_list: &Vec<ImageContainer>, _pages_info: &PagesInfo, area: &DrawingArea, ctx: &cairo::Context) {
    let _index = _pages_info.current_page_index.as_ref().borrow().clone();

    let Some(modified) = _image_container_list[_index].get_modified_pixbuf_data() else { return; };
    let format = if modified.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };
    let pix_w = modified.width();
    let pix_h = modified.height();
    let Ok(surface) = cairo::ImageSurface::create(format, pix_w, pix_h) else { return; };

    let margin = calc_margin_for_single(&modified, area.allocated_width(), area.allocated_height());
    let margin_f_for_surface = f64::from(margin.clone());
    let margin_f_for_pixbuf = f64::from(margin.clone());

    let _ = ctx.set_source_surface(&surface, margin_f_for_surface, 0.0);
    let _ = ctx.set_source_pixbuf(&modified, margin_f_for_pixbuf, 0.0);
    let _ = ctx.paint();
}

fn draw_dual_page(_image_container_list: &Vec<ImageContainer>, _pages_info: &PagesInfo, _settings: &Settings, area: &DrawingArea, ctx: &cairo::Context) {
    let _index = _pages_info.current_page_index.as_ref().borrow().clone();
    let _right_index = _index;
    let _left_index = _index + 1;

    let Some(_right) = _image_container_list[_right_index].get_modified_pixbuf_data() else { return; };
    let _right_format = if _right.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };
    let pix_w = _right.width();
    let pix_h = _right.height();
    let Ok(surface_for_right) = cairo::ImageSurface::create(_right_format, pix_w, pix_h) else { return; };

    let _right_pos = f64::from(pix_w);
    let _ = ctx.set_source_surface(&surface_for_right, _right_pos, 0.0);
    let _ = ctx.set_source_pixbuf(&_right, _right_pos, 0.0);
    let _ = ctx.paint();

    if _left_index >= _image_container_list.len() {
        return;
    }
    
    let _left = _image_container_list[_left_index].get_modified_pixbuf_data().unwrap();
    let _left_format = if _left.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };
    let Ok(surface_for_left) = cairo::ImageSurface::create(_left_format, _left.width(), _left.height()) else { return; };
    let _ = ctx.set_source_surface(&surface_for_left, 0.0, 0.0);
    let _ = ctx.set_source_pixbuf(&_left, 0.0, 0.0);
    let _ = ctx.paint();
}


impl MainWindow {
    fn new() -> MainWindow {

        let window_ui_src = include_str!("window.ui");
        
        let builder = gtk::Builder::new();
        let _ = builder.add_from_string(window_ui_src);

        let _win: ApplicationWindow = builder.object("window").unwrap();

        let _result = MainWindow {
            window: _win,
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            image_container_list: std::rc::Rc::new(std::cell::RefCell::new(vec!())),
            pages_info: std::rc::Rc::new(PagesInfo::default()),
            settings: std::rc::Rc::new(Settings::default()),
        };

        _result
    }

    fn init(&self, app: &Application, width: i32, height: i32) {
        self.window.set_title(Some("Simple Comics Viewer"));
        self.window.set_default_size(width, height);
        self.window.set_show_menubar(true);

        let _window = &self.window;
        let _image_container_list = &self.image_container_list;
        let _pages_info = &self.pages_info;
        let _settings = &self.settings;
        _settings.is_dual_mode.replace(true);

        let menu_ui_src = include_str!("menu.ui");
        let builder = gtk::Builder::new();
        let _ = builder.add_from_string(menu_ui_src);
        let _menubar: gio::MenuModel = builder.object("menu").unwrap();
        app.set_menubar(Some(&_menubar));


        let _drawing_area = gtk::DrawingArea::builder()
            .hexpand_set(true)
            .vexpand_set(true)
            .build();
        _drawing_area.set_draw_func(glib::clone!(@strong _image_container_list, @strong _pages_info, @strong _settings => move |area: &DrawingArea, ctx: &cairo::Context, width: i32, height: i32| {
            if _image_container_list.borrow().is_empty() {
                return;
            }

            if *_settings.is_dual_mode.borrow() {
                draw_dual_page(&_image_container_list.borrow(), &_pages_info, &_settings, area, ctx);
            } else {
                draw_single_page(&_image_container_list.borrow(), &_pages_info, area, ctx);
            }
        }));

        let _ = _drawing_area.connect_resize(glib::clone!(@strong _image_container_list, @strong _pages_info, @strong _settings => move|_drawing_area: &DrawingArea, width: i32, height: i32| {
            if _image_container_list.borrow().is_empty() { return; }
            let _index = _pages_info.current_page_index.as_ref().borrow().clone();
            // scale_page_for_single(&_image_container_list, _index, width, height);
            scale_page_for_dual(&_image_container_list, _index, width, height);
        }));

        let _event_controller_key = EventControllerKey::builder().build();
        let _ = _event_controller_key.connect_key_pressed(glib::clone!(@strong _image_container_list, @strong _pages_info, @strong _settings, @strong _drawing_area => move |event_controller_key: &EventControllerKey, keyval: gdk::Key, keycode: u32, state: gdk::ModifierType| {
            let is_dual_mode = _settings.is_dual_mode.borrow().clone();
            let tmp: i32 =
                match keyval {
                    gdk::Key::Left => {
                        if is_dual_mode {
                            2
                        } else {
                            1
                        }
                    },
                    gdk::Key::Right => {
                        if is_dual_mode {
                            -2
                        } else {
                            -1
                        }
                    },
                    _ => 0
                };
            if tmp == 0 {
                return gtk::Inhibit(true);
            }

            if _image_container_list.borrow().is_empty() {
                return gtk::Inhibit(true);
            }

            let size = _image_container_list.borrow().len();
            let _i = _pages_info.current_page_index.borrow().clone();
            if _i == 0 && tmp < 0 {
                return gtk::Inhibit(true);
            }

            let mut _result = if tmp > -1 { _i + (tmp as usize) } else { _i - (tmp.abs() as usize) };
            if size <= _result {
                return gtk::Inhibit(true);
            }

            if _result >= size {
                _result = size - 1;
            }
            
            _pages_info.current_page_index.replace(_result);
            let _height = _drawing_area.allocated_height();
            let _width = _drawing_area.allocated_width();
            // scale_page_for_single(&_image_container_list, _result, _width, _height);
            scale_page_for_dual(&_image_container_list, _result, _width, _height);
            
            _drawing_area.queue_draw();
            gtk::Inhibit(true)
        }));
        self.window.add_controller(_event_controller_key);


        let _scroll = gtk::ScrolledWindow::builder().child(&_drawing_area).build();
        _scroll.set_hexpand(true);
        _scroll.set_vexpand(true);
        self.v_box.append(&_scroll);

        let _drawing_area_ref = &_drawing_area;
        let _action_entry = create_action_entry_for_menu(_window, _image_container_list, _pages_info, _drawing_area_ref);
        app.add_action_entries(vec!(_action_entry));
        self.window.set_application(Some(app));

        self.window.set_child(Some(&self.v_box));
    }

    fn run(&self) {
        self.window.show();
    }

}

pub fn activate(app: &Application) {
    let main = MainWindow::new();
    main.init(app, 1024, 768);
    main.run();
}
