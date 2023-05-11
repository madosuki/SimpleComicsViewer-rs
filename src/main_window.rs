use gtk::prelude::{GtkWindowExt, WidgetExt, ContainerExt, MenuShellExt, GtkMenuItemExt, ImageExt, DialogExt, FileChooserExt, FileChooserExtManual};
use gtk::{Application, ApplicationWindow, Button, WindowPosition};
use glib;
use gdk_pixbuf;
use gdk_pixbuf::prelude::PixbufLoaderExt;

use crate::image_loader;
use image_loader::{ImageContainer, ImageContainerEx};


struct FileMenu {
    root: gtk::MenuItem,
    body: gtk::Menu,
    load: gtk::MenuItem,
    quit: gtk::MenuItem,
    file_history: gtk::MenuItem,
}

struct MainWindow {
    window: ApplicationWindow,
    v_box: gtk::Box,
    current_page: gtk::Image,
    image_container_list: std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>,
    // menu_bar: gtk::MenuBar,
    // file_menu: gtk::MenuItem,
}

impl MainWindow {
    fn new(app: &Application) -> MainWindow {

        // let _image_container = ImageContainer::default();
        
        MainWindow {
            window: ApplicationWindow::new(app),
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            current_page: gtk::Image::new(),
            image_container_list: std::rc::Rc::new(std::cell::RefCell::new(vec!())),
            // image_container: std::rc::Rc::new(_image_container),
            // menu_bar: gtk::MenuBar::new(),
            // file_menu: gtk::MenuItem::with_label("File"),
        }
    }

    fn init(&self, width: i32, height: i32) {
        self.window.set_title("Simple Comics Viewer");
        self.window.set_default_size(width, height);

        let window = &self.window;
        let _image_container_list = &self.image_container_list;
        let _current_page = &self.current_page;


        // window.connect_size_allocate(glib::clone!(@strong _image_container => move |_win, _rec| {
        //     println!("x: {}, y: {}\nwidth: {}, height: {}", _rec.x(), _rec.y(), _rec.width(), _rec.height());
        //     println!("orig width: {}, orig height: {}", _image_container.get_orig_width(), _image_container.get_orig_height());
        // }));

        let menu_bar = gtk::MenuBar::new();
        let file_menu = FileMenu {
            root: gtk::MenuItem::with_label("File"),
            body: gtk::Menu::new(),
            load: gtk::MenuItem::with_label("Load"),
            quit: gtk::MenuItem::with_label("Quit"),
            file_history: gtk::MenuItem::with_label("File History"),
        };
        file_menu.body.add(&file_menu.load);
        file_menu.load.connect_activate(glib::clone!(@weak window, @strong _current_page, @strong _image_container_list => move |_| {
            let dialog = gtk::FileChooserDialog::new(Some("File Select"), Some(&window), gtk::FileChooserAction::Open);

            dialog.add_button("Open", gtk::ResponseType::Ok);
            dialog.add_button("Cancel", gtk::ResponseType::Cancel);

            dialog.connect_response(glib::clone!(@strong _current_page, @strong _image_container_list => move|file_dialog, response| {
                if response == gtk::ResponseType::Ok {
                    println!("ok");
                    let filename = file_dialog.filename();
                    if filename.is_some() {
                        let filename_unwraped = filename.unwrap();
                        println!("{}", filename_unwraped.display());

                        let _image_container = ImageContainer::default();
                        _image_container_list.borrow_mut().push(_image_container);

                        _image_container_list.borrow()[0].set_pixbuf_from_file(&filename_unwraped.display().to_string(), window.width_request(), window.height_request());
                        _image_container_list.borrow()[0].scale(1024, 768);
                        if let Some(v) = _image_container_list.borrow()[0].get_modified_pixbuf_data() {
                            _current_page.set_pixbuf(Some(&v));
                        }
                        
                        // println!("width: {}, height: {}", _image_container.get_orig_width(), _image_container.get_orig_height());

                    }
                }
                file_dialog.close();
            }));
            
            dialog.show_all();
        }));
        file_menu.body.add(&file_menu.file_history);
        file_menu.body.add(&file_menu.quit);
        file_menu.quit.connect_activate(glib::clone!(@weak window => move |_| {
            window.close();
        }));
        
        file_menu.root.set_submenu(Some(&file_menu.body));
        menu_bar.append(&file_menu.root);
        self.v_box.add(&menu_bar);

        let _scroll = gtk::ScrolledWindow::builder().child(_current_page).build();
        _scroll.set_expand(true);
        self.v_box.add(&_scroll);

        self.window.add(&self.v_box);
    }

    fn run(&self) {
        self.window.show_all();
    }

}

pub fn activate(app: &Application) {
    let main = MainWindow::new(app);
    main.init(1024, 768);
    main.run();

}
