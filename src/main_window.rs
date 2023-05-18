use gtk4 as gtk;
use gdk4 as gdk;

use gtk::prelude::{ApplicationExt, ApplicationWindowExt, GtkApplicationExt, GtkWindowExt, WidgetExt, DialogExt, FileChooserExt, FileChooserExtManual,  MenuModelExt, BoxExt, DrawingAreaExt, DrawingAreaExtManual, SurfaceExt, GdkCairoContextExt, PopoverExt, ActionMapExtManual, FileExt};
use gtk::{Application, ApplicationWindow, Button, Allocation, DrawingArea, cairo, PopoverMenu, gio, glib};
use glib::clone;
use gdk_pixbuf;
use gdk_pixbuf::prelude::PixbufLoaderExt;

use crate::image_loader;
use image_loader::{ImageContainer, ImageContainerEx};

#[derive(Default)]
struct Page {
    x: i32,
    y: i32
}

struct MainWindow {
    window: ApplicationWindow,
    v_box: gtk::Box,
    image_container_list: std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>,
}

fn calc_margin(pixbuf_data: &gdk_pixbuf::Pixbuf, win_width: i32, win_height: i32) -> i32 {
    let pic_height = pixbuf_data.height();
    let pic_width = pixbuf_data.width();

    let diff = win_width - pic_width;


    if diff < 0 {
        return -1;
    }

    if diff == 0 {
        diff
    } else {
        diff / 2
    }
}

fn scale_page(image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, width: i32, height: i32) {

    if (width < 1) || (height < 1) {
        return;
    }
    
    if image_container_list.borrow().is_empty() {
        return;
    }
    
    image_container_list.borrow()[0].scale(width, height);

    if let Some(v) = image_container_list.borrow()[0].get_modified_pixbuf_data() {
        println!("modified width and height: {}, {}", v.width(), v.height());
    }
}


fn set_page_from_file(file: &gio::File, image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, width: i32, height: i32) {
    let _image_container = ImageContainer::default();
    image_container_list.borrow_mut().clear();
    image_container_list.borrow_mut().push(_image_container);

    image_container_list.borrow()[0].set_pixbuf_from_file(file, width, height);
    image_container_list.borrow()[0].scale(width, height);
}

fn create_action_entry_for_menu(_window: &gtk::ApplicationWindow, _image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, _drawing_area_ref: &DrawingArea) -> gio::ActionEntry<gtk::Application> {
        let _action_entry: gio::ActionEntry<gtk::Application> = gio::ActionEntry::builder("file_open")
            .activate(glib::clone!(@weak _window, @strong _image_container_list, @strong _drawing_area_ref => move |_app: &gtk::Application, _action: &gio::SimpleAction, _user_data: Option<&glib::Variant>| {
                println!("do action!");

                let dialog = gtk::FileChooserDialog::new(Some("File Select"),
                                                         Some(&_window),
                                                         gtk::FileChooserAction::Open,
                &[("Open", gtk::ResponseType::Ok), ("Cancel", gtk::ResponseType::Cancel)]);

                dialog.connect_response(glib::clone!(@strong _image_container_list, @strong _drawing_area_ref => move |file_dialog, response| {
                    if response == gtk::ResponseType::Ok {
                        println!("ok");
                        let Some(file) = file_dialog.file() else { return };
                        println!("{}", file.basename().unwrap().to_str().unwrap());

                        println!("drawing area allocated height: {}", _drawing_area_ref.allocated_height());
                        
                        set_page_from_file(&file, &_image_container_list, _drawing_area_ref.allocated_width(), _drawing_area_ref.allocated_height());
                        
                        _drawing_area_ref.queue_draw();
                    }
                    file_dialog.close();
                }));
            
                dialog.show();
            }))
        .build();
    _action_entry
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
        };

        _result
    }

    fn init(&self, app: &Application, width: i32, height: i32) {
        self.window.set_title(Some("Simple Comics Viewer"));
        self.window.set_default_size(width, height);
        self.window.set_show_menubar(true);

        let _window = &self.window;
        let _image_container_list = &self.image_container_list;

        let menu_ui_src = include_str!("menu.ui");
        let builder = gtk::Builder::new();
        let _ = builder.add_from_string(menu_ui_src);
        let _menubar: gio::MenuModel = builder.object("menu").unwrap();
        app.set_menubar(Some(&_menubar));

        let _drawing_area = gtk::DrawingArea::builder()
            .hexpand_set(true)
            .vexpand_set(true)
            .build();
        _drawing_area.set_draw_func(glib::clone!(@strong _image_container_list => move |area: &DrawingArea, ctx: &cairo::Context, width: i32, height: i32| {
            if _image_container_list.borrow().is_empty() {
                return;
            }

            let Some(modified) = _image_container_list.borrow()[0].get_modified_pixbuf_data() else { return; };
            let format = if modified.has_alpha() {
                cairo::Format::ARgb32
            } else {
                cairo::Format::Rgb24
            };
            let pix_w = modified.width();
            let pix_h = modified.height();
            let Ok(surface) = cairo::ImageSurface::create(format, pix_w, pix_h) else { return; };

            let margin = calc_margin(&modified, area.allocated_width(), area.allocated_height());
            let margin_f_for_surface = f64::from(margin.clone());
            let margin_f_for_pixbuf = f64::from(margin.clone());

            let _ = ctx.set_source_surface(&surface, margin_f_for_surface, 0.0);
            let _ = ctx.set_source_pixbuf(&modified, margin_f_for_pixbuf, 0.0);
            let _ = ctx.paint();
        }));

        let _ = _drawing_area.connect_resize(glib::clone!(@strong _image_container_list => move|_drawing_area: &DrawingArea, width: i32, height: i32| {
            if _image_container_list.borrow().is_empty() { return; }
            scale_page(&_image_container_list, width, height);
        }));

        let _scroll = gtk::ScrolledWindow::builder().child(&_drawing_area).build();
        _scroll.set_hexpand(true);
        _scroll.set_vexpand(true);
        self.v_box.append(&_scroll);

        let _drawing_area_ref = &_drawing_area;
        let _action_entry = create_action_entry_for_menu(_window, _image_container_list, _drawing_area_ref);
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
