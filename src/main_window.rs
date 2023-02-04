use gtk::prelude::{GtkWindowExt, WidgetExt, ContainerExt, MenuShellExt, GtkMenuItemExt};
use gtk::{Application, ApplicationWindow, Button, WindowPosition};
use glib;
// use gdk_pixbuf;

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
    // menu_bar: gtk::MenuBar,
    // file_menu: gtk::MenuItem,
}

impl MainWindow {
    fn new(app: &Application) -> MainWindow {
        MainWindow {
            window: ApplicationWindow::new(app),
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            // menu_bar: gtk::MenuBar::new(),
            // file_menu: gtk::MenuItem::with_label("File"),
        }
    }

    fn init(&self, width: i32, height: i32) {
        self.window.set_title("Simple Comics Viewer");
        self.window.set_default_size(width, height);

        // self.menu_bar.append(&self.file_menu);
        // self.v_box.add(&self.menu_bar);

        let menu_bar = gtk::MenuBar::new();
        let file_menu = FileMenu {
            root: gtk::MenuItem::with_label("File"),
            body: gtk::Menu::new(),
            load: gtk::MenuItem::with_label("Load"),
            quit: gtk::MenuItem::with_label("Quit"),
            file_history: gtk::MenuItem::with_label("File History"),
        };
        file_menu.body.add(&file_menu.load);
        file_menu.body.add(&file_menu.file_history);
        file_menu.body.add(&file_menu.quit);
        let window = &self.window;
        file_menu.quit.connect_activate(glib::clone!(@weak window => move |_| {
            window.close();
        }));
        
        file_menu.root.set_submenu(Some(&file_menu.body));
        menu_bar.append(&file_menu.root);
        
        self.v_box.add(&menu_bar);
        
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
