use gtk::prelude::{GtkWindowExt, WidgetExt, ContainerExt, MenuShellExt, GtkMenuItemExt, ImageExt, DialogExt, FileChooserExt, FileChooserExtManual};
use gtk::{Application, ApplicationWindow, Button, WindowPosition, Allocation};
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

#[derive(Default)]
struct PageContainer {
    left: gtk::Image,
    right: gtk::Image
}

struct MainWindow {
    window: ApplicationWindow,
    v_box: gtk::Box,
    pages: std::rc::Rc<PageContainer>,
    image_container_list: std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>,
    menubar_size: std::rc::Rc<std::cell::Cell<gtk::Rectangle>>,
    // menu_bar: gtk::MenuBar,
    // file_menu: gtk::MenuItem,
}

fn scale_page(image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, pages: &std::rc::Rc<PageContainer>, window: &ApplicationWindow, menubar_size: &std::rc::Rc<std::cell::Cell<gtk::Rectangle>>) {

    if image_container_list.borrow().is_empty() {
        return;
    }
    
    let (width, height) = window.size();
    println!("window size: {}, {}", &width, &height);

    let final_height = std::cmp::max(height, menubar_size.get().height()) - std::cmp::min(height, menubar_size.get().height());
    println!("final height: {}", final_height);
    println!("menubar height: {}", menubar_size.get().height());

    println!("image_container_list len on scale_page: {}", image_container_list.borrow().len());

    image_container_list.borrow()[0].scale(width, final_height);

    if let Some(v) = image_container_list.borrow()[0].get_modified_pixbuf_data() {
        pages.left.clear();
        println!("modified width and height: {}, {}", v.width(), v.height());
        pages.left.set_from_pixbuf(Some(&v));
    }
}


fn set_page_from_file(file_path: String, image_container_list: &std::rc::Rc<std::cell::RefCell<Vec<ImageContainer>>>, pages: &std::rc::Rc<PageContainer>, window: &ApplicationWindow, menubar_size: &std::rc::Rc<std::cell::Cell<gtk::Rectangle>>) {
    let _image_container = ImageContainer::default();
    image_container_list.borrow_mut().clear();
    image_container_list.borrow_mut().push(_image_container);

    image_container_list.borrow()[0].set_pixbuf_from_file(&file_path, window.width_request(), window.height_request());
    let (width, height) = window.size();
    println!("window size: {}, {}", &width, &height);

    let final_height = std::cmp::max(height, menubar_size.get().height()) - std::cmp::min(height, menubar_size.get().height());
    println!("final height: {}", final_height);
    println!("menubar height: {}", menubar_size.get().height());
    
    image_container_list.borrow()[0].scale(width, final_height);

    if let Some(v) = image_container_list.borrow()[0].get_modified_pixbuf_data() {
        pages.left.set_pixbuf(Some(&v));
    }
}


impl MainWindow {
    fn new(app: &Application) -> MainWindow {

        // let _image_container = ImageContainer::default();
        
        MainWindow {
            window: ApplicationWindow::new(app),
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            pages: std::rc::Rc::new(PageContainer::default()),
            image_container_list: std::rc::Rc::new(std::cell::RefCell::new(vec!())),
            menubar_size: std::rc::Rc::new(std::cell::Cell::new(gtk::Rectangle::new(0, 0, 0, 0))),
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
        let _pages = &self.pages;
        let _menubar_size = &self.menubar_size;


        window.connect_size_allocate(glib::clone!(@weak window, @strong _image_container_list, @strong _menubar_size, @strong _pages  => move |_win, _rec| {
            // println!("x: {}, y: {}\nwidth: {}, height: {}", _rec.x(), _rec.y(), _rec.width(), _rec.height());
            // println!("orig width: {}, orig height: {}", _image_container.get_orig_width(), _image_container.get_orig_height());
            scale_page(&_image_container_list, &_pages, &window, &_menubar_size);
        }));

        let menu_bar = gtk::MenuBar::new();
        menu_bar.connect_size_allocate(glib::clone!(@strong _menubar_size => move |_menubar, _rec| {
            _menubar_size.replace(*_rec);
        }));
        let file_menu = FileMenu {
            root: gtk::MenuItem::with_label("File"),
            body: gtk::Menu::new(),
            load: gtk::MenuItem::with_label("Load"),
            quit: gtk::MenuItem::with_label("Quit"),
            file_history: gtk::MenuItem::with_label("File History"),
        };
        file_menu.body.add(&file_menu.load);
        file_menu.load.connect_activate(glib::clone!(@weak window, @strong _pages, @strong _image_container_list, @strong _menubar_size => move |_| {
            let dialog = gtk::FileChooserDialog::new(Some("File Select"), Some(&window), gtk::FileChooserAction::Open);

            dialog.add_button("Open", gtk::ResponseType::Ok);
            dialog.add_button("Cancel", gtk::ResponseType::Cancel);

            dialog.connect_response(glib::clone!(@strong _pages, @strong _image_container_list, @strong _menubar_size => move|file_dialog, response| {
                if response == gtk::ResponseType::Ok {
                    println!("ok");
                    let filename = file_dialog.filename();
                    if filename.is_some() {
                        let filename_unwraped = filename.unwrap();
                        println!("{}", filename_unwraped.display());

                        set_page_from_file(filename_unwraped.display().to_string(), &_image_container_list, &_pages, &window, &_menubar_size);
    
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

        let _scroll = gtk::ScrolledWindow::builder().child(&_pages.left).build();
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
