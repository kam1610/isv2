use gtk::glib;
use gtk::Application;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;

const APP_ID: &str = "org.gtk.ImageScenarioViewer2";

// main ////////////////////////////////////////////////////
fn main() -> glib::ExitCode {
    println!("--------");
    ////////////////////////////////////////////////////////
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| isv2::load_css());
    app.connect_activate(isv2::build_ui);
    app.run()
}
