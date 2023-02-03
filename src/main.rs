mod main_window;

use gtk::Application;
use gio::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    let app_id_str: &str = "simple_comics_viewer";
    let app = Application::builder().application_id(app_id_str).build();

    app.connect_activate(main_window::on_activate);
    app.run();
}
