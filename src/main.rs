#![windows_subsystem = "windows"]

use gtk::glib;
use gtk::Application;
use gtk::prelude::ApplicationExt;
use gtk::prelude::ApplicationExtManual;

const APP_ID: &str = "org.gtk.ImageScenarioViewer2";

// hide_console_window /////////////////////////////////////
// ref: https://stackoverflow.com/questions/29763647/how-to-make-a-program-that-does-not-display-the-console-window
#[cfg(target_os = "windows")]
fn hide_console_window() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe {GetConsoleWindow()};
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if window != ptr::null_mut() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

// main ////////////////////////////////////////////////////
fn main() -> glib::ExitCode {
    println!("--------");
    ////////////////////////////////////////////////////////
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| isv2::load_css());
    app.connect_activate(isv2::build_ui);
    let ret = app.run();

    #[cfg(target_os = "windows")]
    hide_console_window();

    ret
}
