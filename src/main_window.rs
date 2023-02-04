use std::fs::File;
use std::io::Read;

use gtk::prelude::{GtkWindowExt, WidgetExt, ContainerExt, MenuShellExt, GtkMenuItemExt, ImageExt, DialogExt, FileChooserExt, FileChooserExtManual};
use gtk::{Application, ApplicationWindow, Button, WindowPosition};
use glib;
use gdk_pixbuf;
use gdk_pixbuf::prelude::PixbufLoaderExt;


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
    image: gtk::Image,
    // menu_bar: gtk::MenuBar,
    // file_menu: gtk::MenuItem,
}

fn create_pixbuf_from_file(path_str: String) -> Option<gdk_pixbuf::Pixbuf> {
    let path = Some(std::path::Path::new(path_str.as_str())).unwrap();
    let mut f = File::open(path).unwrap();
    let mut buf: Vec<u8> = vec!();
    let result_of_read = f.read_to_end(&mut buf);
    if result_of_read.is_err() {
        return None
    }
    
    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    let result_of_write = pixbuf_loader.write(&buf);
    if result_of_write.is_err() {
        return None
    }
    
    let pixbuf_data = pixbuf_loader.pixbuf().unwrap();
    let result_of_close = pixbuf_loader.close();
    if result_of_close.is_err() {
        return None
    }

    Some(pixbuf_data)
}

fn set_image_from_pixbuf(_image: &gtk::Image, _pixbuf_data: &gdk_pixbuf::Pixbuf) {
    _image.set_from_pixbuf(Some(_pixbuf_data));
    _image.set_vexpand(true);
}

impl MainWindow {
    fn new(app: &Application) -> MainWindow {
        MainWindow {
            window: ApplicationWindow::new(app),
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            image: gtk::Image::new(),
            // menu_bar: gtk::MenuBar::new(),
            // file_menu: gtk::MenuItem::with_label("File"),
        }
    }

    fn init(&self, width: i32, height: i32) {
        self.window.set_title("Simple Comics Viewer");
        self.window.set_default_size(width, height);
        let window = &self.window;

        // self.menu_bar.append(&self.file_menu);
        // self.v_box.add(&self.menu_bar);

        let _image = &self.image;

        let menu_bar = gtk::MenuBar::new();
        let file_menu = FileMenu {
            root: gtk::MenuItem::with_label("File"),
            body: gtk::Menu::new(),
            load: gtk::MenuItem::with_label("Load"),
            quit: gtk::MenuItem::with_label("Quit"),
            file_history: gtk::MenuItem::with_label("File History"),
        };
        file_menu.body.add(&file_menu.load);
        file_menu.load.connect_activate(glib::clone!(@weak window, @strong _image => move |_| {
            let dialog = gtk::FileChooserDialog::new(Some("File Select"), Some(&window), gtk::FileChooserAction::Open);

            dialog.add_button("Open", gtk::ResponseType::Ok);
            dialog.add_button("Cancel", gtk::ResponseType::Cancel);

            dialog.connect_response(glib::clone!(@strong _image => move |file_dialog, response| {
                if response == gtk::ResponseType::Ok {
                    println!("ok");
                    let filename = file_dialog.filename();
                    if filename.is_some() {
                        let fname = filename.unwrap();
                        println!("{}", fname.display());

                        let pixbuf_data = create_pixbuf_from_file(fname.display().to_string());
                        if pixbuf_data.is_some() {
                            set_image_from_pixbuf(&_image, &pixbuf_data.unwrap());
                        }
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

        let _scroll = gtk::ScrolledWindow::builder().child(&self.image).build();
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
