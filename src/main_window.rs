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

fn calc_margin_for_single(pixbuf_data: &gdk_pixbuf::Pixbuf, _target_width: i32, _target_height: i32) -> i32 {
    let _pic_height = pixbuf_data.height();
    let _pic_width = pixbuf_data.width();

    let diff = _target_width - _pic_width;


    if diff < 0 {
        return -1;
    }

    if diff == 0 {
        diff
    } else {
        diff / 2
    }
}

fn calc_margin_for_dual(_left: &gdk_pixbuf::Pixbuf, _right: &gdk_pixbuf::Pixbuf, _target_width: i32, _target_height: i32) -> i32 {
    let _left_height = _left.height();
    let _left_width = _left.width();
    let _right_height = _right.height();
    let _right_width = _right.width();

    let diff = _target_width - (_left_width + _right_width);

    if diff < 0 {
        return -1;
    }

    if diff == 0 {
        diff
    } else {
        diff / 2
    }
}


fn scale_page_for_single(image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, current_page_index: usize, _target_width: i32, _target_height: i32) {

    if (_target_width < 1) || (_target_height < 1) {
        return;
    }
    
    if image_container_list.borrow().is_empty() {
        return;
    }

    if current_page_index >= image_container_list.borrow().len() {
        return;
    }

    
    image_container_list.borrow()[current_page_index].scale(_target_width, _target_height, false);
}

fn scale_page_for_dual(image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, current_page_index: usize, _target_width: i32, _target_height: i32) {

    if (_target_width < 1) || (_target_height < 1) {
        return;
    }
    
    if image_container_list.borrow().is_empty() {
        return;
    }

    let next_index = current_page_index + 1;
    let _image_container_list_len = image_container_list.borrow().len();

    let final_target_width = _target_width / 2;
    image_container_list.borrow()[current_page_index].scale(final_target_width, _target_height, true);

    if next_index != _image_container_list_len {
        image_container_list.borrow()[next_index].scale(final_target_width, _target_height, true);
    }
}



fn set_page_from_file(file: &gio::File, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, page_index: usize, width: i32, height: i32, is_dual_mode: bool) {
    let _image_container = ImageContainer::default();
    _image_container_list.borrow_mut().push(_image_container);

    _image_container_list.borrow()[page_index].set_pixbuf_from_file(file, width, height);

    if is_dual_mode {
        let half_width = width / 2;
        _image_container_list.borrow()[page_index].scale(half_width, height, is_dual_mode);
    } else {
        _image_container_list.borrow()[page_index].scale(width, height, is_dual_mode);
    }

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


fn open_and_set_image_from_zip(file: &gio::File, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, _drawing_area_ref: &DrawingArea, _settings: &Settings) {
    let Some(_pathbuf) = file.path() else { return; };
    let Some(_pathname) = _pathbuf.as_path().to_str() else {
        return;
    };

    let _extracted = image_loader::load_from_compressed_file_to_memory(_pathname).unwrap();
    let mut count = 0;
    _extracted.into_iter().for_each(|v| {
        if *_settings.is_dual_mode.borrow() {
            set_page_from_bytes_for_dual(&v.value, &_image_container_list, count, _drawing_area_ref.allocated_width(), _drawing_area_ref.allocated_height());
        } else {
            set_page_from_bytes_for_single(&v.value, &_image_container_list, count, _drawing_area_ref.allocated_width(), _drawing_area_ref.allocated_height());
        }

        count = count + 1;
    });
}

fn open_and_set_image(file: &gio::File, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, _drawing_area_ref: &DrawingArea, page_index: usize, _settings: &Settings) {
    match utils::detect_file_type_from_file(&file) {
        utils::FileType::NONE => { return; },
        _ => {
            set_page_from_file(&file, &_image_container_list, page_index, _drawing_area_ref.allocated_width(), _drawing_area_ref.allocated_height(), *_settings.is_dual_mode.borrow());
        }
    };
}


fn create_action_entry_for_menu(_window: &gtk::ApplicationWindow,
                                _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>,
                                _pages_info: &std::rc::Rc<PagesInfo>,
                                _drawing_area_ref: &DrawingArea,
                                _settings: &std::rc::Rc<Settings>) -> gio::ActionEntry<gtk::Application> {
        let _action_entry: gio::ActionEntry<gtk::Application> = gio::ActionEntry::builder("file_open")
            .activate(glib::clone!(@weak _window, @strong _image_container_list, @strong _pages_info, @strong _settings, @strong _drawing_area_ref => move |_app: &gtk::Application, _action: &gio::SimpleAction, _user_data: Option<&glib::Variant>| {
                let dialog = gtk::FileChooserDialog::new(Some("File Select"),
                                                         Some(&_window),
                                                         gtk::FileChooserAction::Open,
                &[("Open", gtk::ResponseType::Ok), ("Cancel", gtk::ResponseType::Cancel)]);

                dialog.connect_response(glib::clone!(@strong _image_container_list, @strong _pages_info, @strong _drawing_area_ref, @strong _settings => move |file_dialog, response| {
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
                            open_and_set_image_from_zip(&file, &_image_container_list, &_drawing_area_ref, &_settings);
                        } else {
                            let Some(_dir) = _path.parent() else {
                                open_and_set_image(&file, &_image_container_list, &_drawing_area_ref, 0, &_settings);
                                _drawing_area_ref.queue_draw();
                                return;
                            };

                            let mut count: usize = 0;
                            for entry in _dir.read_dir().expect("read_dir call failed") {
                                if let Ok(entry) = entry {
                                    if entry.file_type().unwrap().is_file() {
                                        let tmp_path = entry.path();
                                        let tmp_file = gio::File::for_path(&tmp_path);
                                        open_and_set_image(&tmp_file, &_image_container_list, &_drawing_area_ref, count, &_settings);
                                        count = count + 1;
                                    }
                                }
                            }
                        }
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

    if _left_index >= _image_container_list.len() {
        let _ = ctx.set_source_surface(&surface_for_right, _right_pos, 0.0);
        let _ = ctx.set_source_pixbuf(&_right, _right_pos, 0.0);
        let _ = ctx.paint();
        return;
    }
    
    let Some(_left) = _image_container_list[_left_index].get_modified_pixbuf_data() else {
        let _ = ctx.set_source_surface(&surface_for_right, _right_pos, 0.0);
        let _ = ctx.set_source_pixbuf(&_right, _right_pos, 0.0);
        let _ = ctx.paint();
        return;
    };
    let _left_format = if _left.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };

    let margin = calc_margin_for_dual(&_right, &_left, area.allocated_width(), area.allocated_height()) as f64;

    let _margin_for_right = _right_pos + margin;
    
    let _ = ctx.set_source_surface(&surface_for_right, _margin_for_right, 0.0);
    let _ = ctx.set_source_pixbuf(&_right, _margin_for_right, 0.0);
    let _ = ctx.paint();
    
    let Ok(surface_for_left) = cairo::ImageSurface::create(_left_format, _left.width(), _left.height()) else { return; };
    let _ = ctx.set_source_surface(&surface_for_left, margin, 0.0);
    let _ = ctx.set_source_pixbuf(&_left, margin, 0.0);
    let _ = ctx.paint();
}

fn fullscreen(_window: &gtk::ApplicationWindow, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, _pages_info: &PagesInfo, _drawing_area_ref: &DrawingArea) {
    if _window.is_fullscreen() {
        _window.unfullscreen();
        _window.set_show_menubar(true);
    } else {
        _window.fullscreen();
        _window.set_show_menubar(false);
    }
}

fn move_page(n: i32,
             _settings: &Settings,
             _drawing_area: &DrawingArea,
             _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>,
             _pages_info: &std::rc::Rc<PagesInfo>) {

    if n == 0 || _image_container_list.borrow().is_empty() {
        return;
    }

    let size = _image_container_list.borrow().len();
    let _i = _pages_info.current_page_index.borrow().clone();
    if _i == 0 && n < 0 {
        return;
    }

    let mut _result = if n > -1 { _i + (n as usize) } else { _i - (n.abs() as usize) };
    if size <= _result {
        return;
    }

    if _result >= size {
        _result = size - 1;
    }
            
    _pages_info.current_page_index.replace(_result);
    let _height = _drawing_area.allocated_height();
    let _width = _drawing_area.allocated_width();
    if *_settings.is_dual_mode.borrow() {
        scale_page_for_dual(&_image_container_list, _result, _width, _height);
    } else {
        scale_page_for_single(&_image_container_list, _result, _width, _height);
    }
            
    _drawing_area.queue_draw();
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
        _drawing_area.set_draw_func(glib::clone!(@strong _image_container_list, @strong _pages_info, @strong _settings => move |area: &DrawingArea, ctx: &cairo::Context, _width: i32, _height: i32| {
            if _image_container_list.borrow().is_empty() {
                return;
            }

            if *_settings.is_dual_mode.borrow() {
                draw_dual_page(&_image_container_list.borrow(), &_pages_info, &_settings, area, ctx);
            } else {
                draw_single_page(&_image_container_list.borrow(), &_pages_info, area, ctx);
            }
        }));

        let _ = _drawing_area.connect_resize(glib::clone!(@strong _image_container_list, @strong _pages_info, @strong _settings => move|_drawing_area: &DrawingArea, _width: i32, _height: i32| {
            if _image_container_list.borrow().is_empty() { return; }
            // println!("resized! {}, {}", _drawing_area.allocated_width(), _drawing_area.allocated_height());
            let _index = _pages_info.current_page_index.as_ref().borrow().clone();
            if *_settings.is_dual_mode.borrow() {
                scale_page_for_dual(&_image_container_list, _index, _width, _height);
            } else {
                scale_page_for_single(&_image_container_list, _index, _width, _height);                
            }
        }));



        let _event_controller_key = EventControllerKey::builder().build();
        let _ = _event_controller_key.connect_key_pressed(glib::clone!(@strong _window, @strong _image_container_list, @strong _pages_info, @strong _settings, @strong _drawing_area => move |_event_controller_key: &EventControllerKey, keyval: gdk::Key, _keycode: u32, state: gdk::ModifierType| {
            
            if state == gdk::ModifierType::ALT_MASK && keyval == gdk::Key::Return {
                fullscreen(&_window, &_image_container_list, &_pages_info, &_drawing_area);
                return gtk::Inhibit(true);
            }

            match keyval {
                gdk::Key::Left => {
                    if *_settings.is_dual_mode.borrow() {
                        move_page(2, &_settings, &_drawing_area, &_image_container_list, &_pages_info);
                    } else {
                        move_page(1, &_settings, &_drawing_area, &_image_container_list, &_pages_info);
                    }
                    gtk::Inhibit(true)
                },
                gdk::Key::Right => {
                    if *_settings.is_dual_mode.borrow() {
                        move_page(-2, &_settings, &_drawing_area, &_image_container_list, &_pages_info);
                    } else {
                        move_page(-1, &_settings, &_drawing_area, &_image_container_list, &_pages_info);
                    }
                    gtk::Inhibit(true)
                },
                _ => gtk::Inhibit(true)
            }
        }));
        self.window.add_controller(_event_controller_key);


        let _scroll = gtk::ScrolledWindow::builder().child(&_drawing_area).build();
        _scroll.set_hexpand(true);
        _scroll.set_vexpand(true);
        self.v_box.append(&_scroll);

        let _drawing_area_ref = &_drawing_area;
        let _action_entry = create_action_entry_for_menu(_window, _image_container_list, _pages_info, _drawing_area_ref, _settings);
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
