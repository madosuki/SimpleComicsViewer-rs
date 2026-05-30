mod file_history;
mod image_container;
mod image_loader;
mod main_window;
mod natural_sort;
mod pdf_loader;
mod types;
mod utils;

use gtk::Application;
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4 as gtk;

fn main() -> gtk::glib::ExitCode {
    let app_id_str: &str = "com.simple_comics_viewer";
    let app = Application::builder().application_id(app_id_str).build();

    app.connect_activate(main_window::activate);
    app.run()
}
