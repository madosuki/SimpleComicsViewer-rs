use gdk4 as gdk;
use gtk4 as gtk;

use gtk::glib::{ControlFlow, Propagation};
use gtk::prelude::{
    ActionMapExtManual, ApplicationExt, ApplicationWindowExt, BoxExt, DialogExt, DrawingAreaExt,
    DrawingAreaExtManual, FileChooserExt, FileChooserExtManual, FileExt, GdkCairoContextExt,
    GridExt, GtkApplicationExt, GtkWindowExt, MenuLinkIterExt, MenuModelExt, PixbufLoaderExt,
    PopoverExt, SurfaceExt, WidgetExt,
};
use gtk::{
    cairo, gio, glib, Allocation, Application, ApplicationWindow, Button, DrawingArea,
    EventControllerKey, PopoverMenu,
};

use anyhow::Result;

use std::sync::Arc;
use std::sync::Mutex;

use crate::image_container;
use crate::image_loader;
use crate::utils;
use image_container::{ImageContainer, ImageContainerEx};

#[derive(Default)]
struct PagesInfo {
    current_page_index: Arc<Mutex<usize>>,
    loaded_filename: Arc<Mutex<Option<String>>>,
    loaded_dirname: Arc<Mutex<Option<String>>>,
}

#[derive(Default)]
struct Settings {
    is_dual_mode: Arc<Mutex<bool>>,
}

#[derive(Default)]
struct MarginData {
    left_margin: i32,
    top_margin: i32,
}

#[derive(Default)]
struct MarginDataForDual {
    left_margin: i32,
    top_margin_for_left: i32,
    top_margin_for_right: i32,
}

struct MainWindow {
    window: ApplicationWindow,
    v_box: gtk::Box,
    image_container_list: Arc<Mutex<Vec<ImageContainer>>>,
    pages_info: std::sync::Arc<PagesInfo>,
    settings: std::sync::Arc<Settings>,
    scroll_window: gtk::ScrolledWindow,
}

fn update_window_title(window: &gtk::ApplicationWindow, _msg: &str) {
    let Some(title) = window.title() else {
        return;
    };

    let new_title = format!("Simple Comics Viewer: {}", _msg);
    window.set_title(Some(&new_title));
}

fn calc_margin_for_single(
    pixbuf_data: &gtk::gdk_pixbuf::Pixbuf,
    target_width: i32,
    target_height: i32,
) -> MarginData {
    let pic_height = pixbuf_data.height();
    let pic_width = pixbuf_data.width();

    let width_diff = target_width - pic_width;
    let left_margin = if width_diff < 0 || width_diff == 0 {
        0
    } else {
        width_diff / 2
    };

    let height_diff = target_height - pic_height;
    let top_margin = if height_diff < 0 || height_diff == 0 {
        0
    } else {
        height_diff / 2
    };

    MarginData {
        left_margin,
        top_margin,
    }
}

fn calc_margin_for_dual(
    left: &gtk::gdk_pixbuf::Pixbuf,
    right: &gtk::gdk_pixbuf::Pixbuf,
    target_width: i32,
    target_height: i32,
) -> MarginDataForDual {
    let left_height = left.height();
    let left_width = left.width();
    let right_height = right.height();
    let right_width = right.width();

    let width_diff = target_width - (left_width + right_width);
    let left_margin = if width_diff <= 0 { 0 } else { width_diff / 2 };

    let left_height_diff = target_height - left_height;
    let top_margin_for_left = if left_height_diff <= 0 {
        0
    } else {
        left_height_diff / 2
    };

    let right_height_diff = target_height - right_height;
    let top_margin_for_right = if right_height_diff <= 0 {
        0
    } else {
        right_height_diff / 2
    };

    MarginDataForDual {
        left_margin,
        top_margin_for_left,
        top_margin_for_right,
    }
}

fn scale_page_for_single(
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    current_page_index: usize,
    target_width: i32,
    target_height: i32,
) {
    if (target_width < 1) || (target_height < 1) {
        return;
    }

    if (*image_container_list.lock().unwrap()).is_empty() {
        return;
    }

    if current_page_index >= (*image_container_list.lock().unwrap()).len() {
        return;
    }

    (*image_container_list.lock().unwrap())[current_page_index].scale(
        target_width,
        target_height,
        false,
    );
}

fn scale_page_for_dual(
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    current_page_index: usize,
    target_width: i32,
    target_height: i32,
) {
    if (target_width < 1) || (target_height < 1) {
        return;
    }

    if (*image_container_list.lock().unwrap()).is_empty() {
        return;
    }

    let next_index = current_page_index + 1;
    let _image_container_list_len = (*image_container_list.lock().unwrap()).len();

    let final_target_width = target_width / 2;
    (*image_container_list.lock().unwrap())[current_page_index].scale(
        final_target_width,
        target_height,
        true,
    );

    if next_index != _image_container_list_len {
        (*image_container_list.lock().unwrap())[next_index].scale(
            final_target_width,
            target_height,
            true,
        );
    }
}

fn set_page_from_file(
    file: &gio::File,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    page_index: usize,
    width: i32,
    height: i32,
    is_dual_mode: bool,
) {
    let image_container = ImageContainer::default();
    (*image_container_list.lock().unwrap()).push(image_container);

    (*image_container_list.lock().unwrap())[page_index].set_pixbuf_from_file(file, width, height);

    if is_dual_mode {
        let half_width = width / 2;
        (*image_container_list.lock().unwrap())[page_index].scale(half_width, height, is_dual_mode);
    } else {
        (*image_container_list.lock().unwrap())[page_index].scale(width, height, is_dual_mode);
    }
}

fn set_page_from_bytes(
    bytes: &[u8],
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    page_index: usize,
    width: i32,
    height: i32,
    is_dual_mode: bool,
) {
    let image_container = ImageContainer::default();

    (*image_container_list.lock().unwrap()).push(image_container);

    println!("{}, {}", width, height);
    (*image_container_list.lock().unwrap())[page_index].set_pixbuf_from_bytes(bytes, width, height);
    if is_dual_mode {
        let half_width = width / 2;
        (*image_container_list.lock().unwrap())[page_index].scale(half_width, height, true);
    } else {
        (*image_container_list.lock().unwrap())[page_index].scale(width, height, false);
    }
}

fn open_and_set_image_from_zip(
    file: &gio::File,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    drawing_area_ref: &DrawingArea,
    settings: &Arc<Settings>,
    pages_info: &Arc<PagesInfo>,
    window: &Arc<Mutex<&gtk::ApplicationWindow>>,
) {
    let Some(pathbuf) = file.path() else {
        return;
    };
    let Some(pathname) = pathbuf.as_path().to_str() else {
        return;
    };
    let Some(file_name_osstr) = pathbuf.file_name() else {
        return;
    };
    let Some(file_name) = file_name_osstr.to_str() else {
        return;
    };
    *pages_info.loaded_filename.lock().unwrap() = Some(file_name.to_owned());
    update_window_title(*window.lock().unwrap(), file_name);

    match image_loader::load_from_compressed_file_to_memory(pathname) {
        Ok(extracted) => {
            let mut count = 0;
            extracted.into_iter().for_each(|v| {
                let is_dual_model = settings.is_dual_mode.lock().unwrap();
                if *is_dual_model {
                    set_page_from_bytes(
                        &v.value,
                        &image_container_list,
                        count,
                        drawing_area_ref.allocated_width(),
                        drawing_area_ref.allocated_height(),
                        *is_dual_model,
                    );
                } else {
                    set_page_from_bytes(
                        &v.value,
                        &image_container_list,
                        count,
                        drawing_area_ref.allocated_width(),
                        drawing_area_ref.allocated_height(),
                        *is_dual_model,
                    );
                }
                
                count = count + 1;
            });     
        },
        Err(_) => {return}
    }
}

fn open_and_set_image(
    file: &gio::File,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    drawing_area_ref: &DrawingArea,
    page_index: usize,
    settings: &Arc<Settings>,
    is_one: bool,
    window: &gtk::ApplicationWindow,
) {
    match utils::detect_file_type_from_file(&file) {
        utils::FileType::NONE => (),
        _ => {
            if is_one {
                let Some(path) = file.path() else {
                    return;
                };
                let Some(file_name) = path.file_name() else {
                    return;
                };
                let Some(file_name_str) = file_name.to_str() else {
                    return;
                };
                update_window_title(window, file_name_str);
            }

            set_page_from_file(
                &file,
                &image_container_list,
                page_index,
                drawing_area_ref.allocated_width(),
                drawing_area_ref.allocated_height(),
                *settings.is_dual_mode.lock().unwrap(),
            );
        }
    }
}

fn open_file_action(
    window: &gtk::ApplicationWindow,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    drawing_area_ref: &DrawingArea,
    settings: &Arc<Settings>,
    pages_info: &Arc<PagesInfo>,
) {
    let dialog = gtk::FileChooserDialog::new(
        Some("File Select"),
        Some(window),
        gtk::FileChooserAction::Open,
        &[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ],
    );

    dialog.connect_response(glib::clone!(@weak window, @strong image_container_list, @strong pages_info, @strong drawing_area_ref, @strong settings => move |file_dialog, response| {
        if response == gtk::ResponseType::Ok {
            let Some(file) = file_dialog.file() else { return };
            let Some(path) = file.path() else { return };
            if !path.is_file() { return; }

            let is_zip =
                match utils::detect_file_type_from_file(&file) {
                    utils::FileType::ZIP => true,
                    _ => false
                };

            (*image_container_list.lock().unwrap()).clear();
            *pages_info.current_page_index.lock().unwrap() = 0;
            if is_zip {
                let win = Arc::new(Mutex::new(&window));
                open_and_set_image_from_zip(&file, &image_container_list, &drawing_area_ref, &settings, &pages_info, &win);
            } else {
                let Some(dir_path) = path.parent() else {
                    open_and_set_image(&file, &image_container_list, &drawing_area_ref, 0, &settings, true, &window);
                    drawing_area_ref.queue_draw();
                    return;
                };

                let Some(dir_path_str) = dir_path.to_str() else { return; };
                update_window_title(&window, dir_path_str);
                *pages_info.loaded_dirname.lock().unwrap() = Some(dir_path_str.to_owned());

                let mut count: usize = 0;
                for entry in dir_path.read_dir().expect("read_dir call failed") {
                    if let Ok(entry) = entry {
                        if entry.file_type().unwrap().is_file() {
                            let tmp_path = entry.path();
                            let tmp_file = gio::File::for_path(&tmp_path);
                            open_and_set_image(&tmp_file, &image_container_list, &drawing_area_ref, count, &settings, false, &window);
                            count = count + 1;
                        }
                    }
                }
            }
            drawing_area_ref.queue_draw();
            
        }
        file_dialog.close();
    }));

    dialog.show();
}

fn create_action_entry_for_menu(
    window: &gtk::ApplicationWindow,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    pages_info: &Arc<PagesInfo>,
    drawing_area_ref: &DrawingArea,
    settings: &std::sync::Arc<Settings>,
) -> Vec<gio::ActionEntry<gtk::Application>> {
    let file_action_entry: gio::ActionEntry<gtk::Application> = gio::ActionEntry::builder("file_open")
        .activate(glib::clone!(@weak window, @strong image_container_list, @strong pages_info, @strong settings, @strong drawing_area_ref => move |_app: &gtk::Application, _action: &gio::SimpleAction, _user_data: Option<&glib::Variant>| {
            open_file_action(&window, &image_container_list, &drawing_area_ref, &settings, &pages_info);
        }))
        .build();

    let quit_action_entry: gio::ActionEntry<gtk::Application> = gio::ActionEntry::builder("quit")
        .activate(glib::clone!(@weak window => move |app: &gtk::Application, action: &gio::SimpleAction, user_data: Option<&glib::Variant>| {
            app.quit();
    })).build();

    let result: Vec<gio::ActionEntry<gtk::Application>> =
        vec![file_action_entry, quit_action_entry];
    result
}

fn draw_single_page(
    image_container_list: &Vec<ImageContainer>,
    pages_info: &PagesInfo,
    area: &DrawingArea,
    ctx: &cairo::Context,
) {
    // let _index = _pages_info.current_page_index.as_ref().borrow().clone();
    let index = pages_info.current_page_index.lock().unwrap().clone();

    let Some(modified) = image_container_list[index].get_modified_pixbuf_data() else {
        return;
    };
    let format = if modified.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };
    let pix_w = modified.width();
    let pix_h = modified.height();
    let Ok(surface) = cairo::ImageSurface::create(format, pix_w, pix_h) else {
        return;
    };

    let margin = calc_margin_for_single(&modified, area.allocated_width(), area.allocated_height());
    let left_margin = f64::from(margin.left_margin);
    let top_margin = f64::from(margin.top_margin);

    let _ = ctx.set_source_surface(&surface, left_margin, top_margin);
    let _ = ctx.set_source_pixbuf(&modified, left_margin, top_margin);
    let _ = ctx.paint();
}

fn draw_dual_page(
    image_container_list: &Vec<ImageContainer>,
    pages_info: &PagesInfo,
    settings: &Settings,
    area: &DrawingArea,
    ctx: &cairo::Context,
) {
    // let _index = _pages_info.current_page_index.as_ref().borrow().clone();
    let index = pages_info.current_page_index.lock().unwrap().clone();
    let right_index = index;
    let left_index = index + 1;
    let half_area_width = area.allocated_width() / 2;

    let Some(right) = image_container_list[right_index].get_modified_pixbuf_data() else {
        return;
    };
    let right_format = if right.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };
    let pix_w = right.width();
    let pix_h = right.height();
    let Ok(surface_for_right) = cairo::ImageSurface::create(right_format, pix_w, pix_h) else {
        return;
    };

    let right_pos = if pix_w <= half_area_width {
        f64::from((half_area_width - pix_w) + pix_w)
    } else {
        0.0
    };

    if left_index >= image_container_list.len() {
        // FIXME: refelect page dirction. current is only support right to left.
        let margin = calc_margin_for_single(&right, area.allocated_width(), area.allocated_height());
        let top_margin = f64::from(margin.top_margin);

        let _ = ctx.set_source_surface(&surface_for_right, right_pos, top_margin);
        let _ = ctx.set_source_pixbuf(&right, right_pos, top_margin);
        let _ = ctx.paint();
        return;
    }

    let Some(left) = image_container_list[left_index].get_modified_pixbuf_data() else {
        let _ = ctx.set_source_surface(&surface_for_right, right_pos, 0.0);
        let _ = ctx.set_source_pixbuf(&right, right_pos, 0.0);
        let _ = ctx.paint();
        return;
    };
    let left_format = if left.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };

    let margin = calc_margin_for_dual(
        &left,
        &right,
        area.allocated_width(),
        area.allocated_height(),
    );
    let left_margin = f64::from(margin.left_margin);
    let top_margin_for_left = f64::from(margin.top_margin_for_left);
    let top_margin_for_right = f64::from(margin.top_margin_for_right);

    let left_pic_width = left.width();
    let final_left_margin = if left_pic_width > half_area_width || left_pic_width == half_area_width
    {
        0.0
    } else {
        left_margin
    };

    let right_margin = if left_pic_width > half_area_width {
        final_left_margin + f64::from(left_pic_width)
    } else {
        left_margin + f64::from(left_pic_width)
    };

    let _ = ctx.set_source_surface(&surface_for_right, right_margin, top_margin_for_right);
    let _ = ctx.set_source_pixbuf(&right, right_margin, top_margin_for_right);
    let _ = ctx.paint();

    let Ok(surface_for_left) =
        cairo::ImageSurface::create(left_format, left.width(), left.height())
    else {
        return;
    };
    let _ = ctx.set_source_surface(&surface_for_left, final_left_margin, top_margin_for_left);
    let _ = ctx.set_source_pixbuf(&left, final_left_margin, top_margin_for_left);
    let _ = ctx.paint();
}

fn fullscreen(
    window: &gtk::ApplicationWindow,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    pages_info: &PagesInfo,
    drawing_area_ref: &DrawingArea,
) {
    if window.is_fullscreen() {
        window.unfullscreen();
        window.set_show_menubar(true);
    } else {
        window.fullscreen();
        window.set_show_menubar(false);
    }
}

fn move_page(
    n: i32,
    settings: &Settings,
    drawing_area: &DrawingArea,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    pages_info: &Arc<PagesInfo>,
) {
    if n == 0 || (*image_container_list.lock().unwrap()).is_empty() {
        return;
    }

    let size = (*image_container_list.lock().unwrap()).len();
    let i = pages_info.current_page_index.lock().unwrap().clone();
    if i == 0 && n < 0 {
        return;
    }

    let mut result = if n > -1 {
        i + (n as usize)
    } else {
        i - (n.abs() as usize)
    };
    if size <= result {
        return;
    }

    if result >= size {
        result = size - 1;
    }

    // _pages_info.current_page_index.replace(_result);
    *pages_info.current_page_index.lock().unwrap() = result;
    let height = drawing_area.allocated_height();
    let width = drawing_area.allocated_width();
    if *settings.is_dual_mode.lock().unwrap() {
        scale_page_for_dual(&image_container_list, result, width, height);
    } else {
        scale_page_for_single(&image_container_list, result, width, height);
    }

    drawing_area.queue_draw();
}

impl MainWindow {
    fn new() -> Self {
        let window_ui_src = include_str!("window.ui");

        let builder = gtk::Builder::new();
        let _ = builder.add_from_string(window_ui_src);

        let win = builder.object("window").unwrap();

        let result = MainWindow {
            window: win,
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            image_container_list: Arc::new(Mutex::new(vec![])),
            pages_info: std::sync::Arc::new(PagesInfo::default()),
            settings: std::sync::Arc::new(Settings::default()),
            scroll_window: gtk::ScrolledWindow::new(),
        };

        result
    }

    fn init(&self, app: &Application, width: i32, height: i32) -> Result<()> {
        // let _header_bar = gtk::HeaderBar::builder().build();
        // self.window.set_titlebar(Some(&_header_bar));
        self.window.set_title(Some("Simple Comics Viewer"));
        self.window.set_default_size(width, height);
        self.window.set_show_menubar(true);

        let window = &self.window;
        let image_container_list = &self.image_container_list;
        let pages_info = &self.pages_info;
        let settings = &self.settings;
        *settings.is_dual_mode.lock().unwrap() = true;

        let menu_ui_src = include_str!("menu.ui");
        let builder = gtk::Builder::new();
        builder.add_from_string(menu_ui_src)?;
        let menu_model: gio::MenuModel = builder.object("menu").unwrap();
        // let _menu_button = gtk::MenuButton::builder()
        //     .menu_model(&_menu_model)
        //     .focus_on_click(true)
        //     .build();
        // _header_bar.pack_end(&_menu_button);

        let popover_menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        app.set_menubar(Some(&popover_menu.menu_model().unwrap()));
        // let _history = _menubar.;
        // println!("{:?}", _history);
        // let _tmp_section = _history.unwrap().n_children();
        // println!("{}", _tmp_section);

        // let _popover_menu_bar = gtk::PopoverMenuBar::from_model(Some(&_menu_model));
        // let _menu_button = gtk::MenuButton::builder().label("M").build();

        let drawing_area = gtk::DrawingArea::builder()
            .hexpand_set(true)
            .vexpand_set(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build();
        drawing_area.set_draw_func(glib::clone!(@strong image_container_list, @strong pages_info, @strong settings => move |area: &DrawingArea, ctx: &cairo::Context, width: i32, height: i32| {
            if (*image_container_list.lock().unwrap()).is_empty() {
                return;
            }

            if *settings.is_dual_mode.lock().unwrap() {
                draw_dual_page(&*image_container_list.lock().unwrap(), &pages_info, &settings, area, ctx);
            } else {
                draw_single_page(&*image_container_list.lock().unwrap(), &pages_info, area, ctx);
            }
        }));

        let _ = drawing_area.connect_resize(glib::clone!(@strong image_container_list, @strong pages_info, @strong settings => move|drawing_area: &DrawingArea, width: i32, height: i32| {
            if (*image_container_list.lock().unwrap()).is_empty() { return; }
            
            let index = pages_info.current_page_index.lock().unwrap().clone();
            if *settings.is_dual_mode.lock().unwrap() {
                scale_page_for_dual(&image_container_list, index, width, height);
            } else {
                scale_page_for_single(&image_container_list, index, width, height);                
            }
        }));

        let event_controller_key = EventControllerKey::builder().build();
        let _ = event_controller_key.connect_key_pressed(glib::clone!(@strong window, @strong image_container_list, @strong pages_info, @strong settings, @strong drawing_area, @strong pages_info => move |event_controller_key: &EventControllerKey, keyval: gdk::Key, keycode: u32, state: gdk::ModifierType| {
            
            if state == gdk::ModifierType::ALT_MASK && keyval == gdk::Key::Return {
                fullscreen(&window, &image_container_list, &pages_info, &drawing_area);
                return Propagation::Stop;
            }

            if state == gdk::ModifierType::CONTROL_MASK && keyval == gdk::Key::o {
                open_file_action(&window, &image_container_list, &drawing_area, &settings, &pages_info);
                return Propagation::Stop;
            }

            match keyval {
                gdk::Key::Left => {
                    if *settings.is_dual_mode.lock().unwrap() {
                        move_page(2, &settings, &drawing_area, &image_container_list, &pages_info);
                    } else {
                        move_page(1, &settings, &drawing_area, &image_container_list, &pages_info);
                    }
                    Propagation::Stop
                },
                gdk::Key::Right => {
                    if *settings.is_dual_mode.lock().unwrap() {
                        move_page(-2, &settings, &drawing_area, &image_container_list, &pages_info);
                    } else {
                        move_page(-1, &settings, &drawing_area, &image_container_list, &pages_info);
                    }
                    Propagation::Stop
                },
                _ => Propagation::Stop
            }
        }));
        self.window.add_controller(event_controller_key);

        // let _scroll = gtk::ScrolledWindow::builder().child(&self.v_box).build();
        // _scroll.set_hexpand(true);
        // _scroll.set_vexpand(true);
        self.scroll_window.set_hexpand(true);
        self.scroll_window.set_vexpand(true);
        self.scroll_window.set_halign(gtk::Align::Fill);
        self.scroll_window.set_valign(gtk::Align::Fill);

        let drawing_area_ref = &drawing_area;
        let action_entry = create_action_entry_for_menu(
            window,
            image_container_list,
            pages_info,
            drawing_area_ref,
            settings,
        );
        app.add_action_entries(action_entry);
        self.scroll_window.set_child(Some(drawing_area_ref));
        // self.v_box.set_halign(gtk::Align::Fill);
        // self.v_box.set_valign(gtk::Align::Fill);
        // self.v_box.set_hexpand(true);
        // self.v_box.set_vexpand(true);
        // self.v_box.append(&_drawing_area);

        self.window.set_application(Some(app));
        self.window.set_child(Some(&self.scroll_window));
        Ok(())
    }

    fn run(&self) {
        self.window.show();
    }
}

pub fn activate(app: &Application) {
    let main = MainWindow::new();
    match main.init(app, 1024, 768) {
        Ok(_) => {
            main.run();
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
