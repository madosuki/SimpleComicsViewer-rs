mod image_container;
mod image_loader;
mod main_window;
mod utils;

use gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::Application;
use gtk4 as gtk;

fn main() -> glib::ExitCode {
    let app_id_str: &str = "com.simple_comics_viewer";
    let app = Application::builder().application_id(app_id_str).build();

    app.connect_activate(main_window::activate);
    app.run()
}
