use gtk::prelude::{GtkWindowExt, WidgetExt};
use gtk::{Application, ApplicationWindow, Button};
// use glib;
// use gdk_pixbuf;


struct MainWindow {
    window: ApplicationWindow,
}

impl MainWindow {
    fn init(&self, width: i32, height: i32) {
        self.window.set_title("First GTK+ Program");
        self.window.set_default_size(width, height);
    }

    fn run(&self) {
        self.window.show();
    }
}

pub fn on_activate(app: &Application) {
    let main = MainWindow { window: ApplicationWindow::new(app) };
    main.init(1024, 768);
    main.run();
}
