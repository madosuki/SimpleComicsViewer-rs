use std::fs::File;
use std::io::Read;
use std::cell::Cell;

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

fn read_bytes_from_file(path_str: &str) -> Option<Vec<u8>> {
    let path = Some(std::path::Path::new(path_str)).unwrap();
    let mut f = File::open(path).unwrap();
    let mut buf: Vec<u8> = vec!();
    match f.read_to_end(&mut buf) {
        Ok(_) => Some(buf),
        _ => None,
    }
}


fn create_pixbuf_from_file(path_str: String) -> Option<gdk_pixbuf::Pixbuf> {
    if let Some(buf) = read_bytes_from_file(&path_str) {
        let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
        if let Ok(v) = pixbuf_loader.write(&buf) {
            match pixbuf_loader.pixbuf() {
                None => None,
                Some(v) => {
                    let result_of_loader_close = pixbuf_loader.close();
                    if result_of_loader_close.is_err() {
                        return None
                    }
                    Some(v)
                }
            }
        }
    }
    
}

fn set_image_from_pixbuf(_image: &gtk::Image, _pixbuf_data: &gdk_pixbuf::Pixbuf) {
    _image.set_from_pixbuf(Some(_pixbuf_data));
    _image.set_vexpand(true);
}

#[derive(Default, Clone)]
struct ImageConatainer {
    gtk_image: gtk::Image,
    // src_pixbuf: Option<gdk_pixbuf::Pixbuf>,
    // dst_pixbuf: Option<gdk_pixbuf::Pixbuf>,
}

struct MainWindow {
    window: ApplicationWindow,
    v_box: gtk::Box,
    image: gtk::Image,
    image_container: ImageConatainer
    // menu_bar: gtk::MenuBar,
    // file_menu: gtk::MenuItem,
}


impl MainWindow {
    fn new(app: &Application) -> MainWindow {
        MainWindow {
            window: ApplicationWindow::new(app),
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            image: gtk::Image::new(),
            image_container: ImageConatainer::default(),
            // menu_bar: gtk::MenuBar::new(),
            // file_menu: gtk::MenuItem::with_label("File"),
        }
    }

    fn init(&self, width: i32, height: i32) {
        self.window.set_title("Simple Comics Viewer");
        self.window.set_default_size(width, height);

        let _image_container = &self.image_container;
        let _image = &self.image;
        let window = &self.window;
        let _self = &self;
        
        window.connect_size_allocate(|_win, _rec| {
            // println!("x: {}, y: {}\nwidth: {}, height: {}", _rec.x(), _rec.y(), _rec.width(), _rec.height());
        });

        let menu_bar = gtk::MenuBar::new();
        let file_menu = FileMenu {
            root: gtk::MenuItem::with_label("File"),
            body: gtk::Menu::new(),
            load: gtk::MenuItem::with_label("Load"),
            quit: gtk::MenuItem::with_label("Quit"),
            file_history: gtk::MenuItem::with_label("File History"),
        };
        file_menu.body.add(&file_menu.load);
        file_menu.load.connect_activate(glib::clone!(@weak window, @strong _image_container, @strong _image => move |_| {
            let dialog = gtk::FileChooserDialog::new(Some("File Select"), Some(&window), gtk::FileChooserAction::Open);

            dialog.add_button("Open", gtk::ResponseType::Ok);
            dialog.add_button("Cancel", gtk::ResponseType::Cancel);

            // let img_ptr = &_image_container.gtk_image;

            dialog.connect_response(glib::clone!(@strong _image, @strong _image_container => move|file_dialog, response| {
                if response == gtk::ResponseType::Ok {
                    println!("ok");
                    let filename = file_dialog.filename();
                    if filename.is_some() {
                        let filename_unwraped = filename.unwrap();
                        println!("{}", filename_unwraped.display());

                        let pixbuf_data = create_pixbuf_from_file(filename_unwraped.display().to_string());
                        if pixbuf_data.is_some() {
                            // let pixbuf_data_unwraped = pixbuf_data.unwrap();
                            // _image_container.src_pixbuf = Some(pixbuf_data_unwraped);
                            set_image_from_pixbuf(&_image_container.gtk_image, &pixbuf_data.unwrap());
                            // set_image_from_pixbuf(&_image, &pixbuf_data.unwrap());
                            // set_image_from_pixbuf(&_self.image, &pixbuf_data.unwrap());
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

        let _scroll = gtk::ScrolledWindow::builder().child(&self.image_container.gtk_image).build();
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
