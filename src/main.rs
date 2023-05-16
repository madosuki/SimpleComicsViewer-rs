mod main_window;
mod image_loader;

use gtk4 as gtk;
use gtk::Application;
use gio::prelude::{ApplicationExt, ApplicationExtManual};

fn main() -> glib::ExitCode {
    let app_id_str: &str = "com.simple_comics_viewer";
    let app = Application::builder().application_id(app_id_str).build();

    app.connect_activate(main_window::activate);
    app.run()
}

