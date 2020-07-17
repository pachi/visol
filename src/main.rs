use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::*; // Incluye macros como get_widget!(ui, gtktype, name)
use glib::clone;
use gtk::prelude::*;

mod appstate;
mod config;
// mod static_resource;
mod parsers;
mod utils;
mod window;

use appstate::AppState;
use config::Config;

// use gdk::{PixBuf};

// Ver https://github.com/gtk-rs/examples/blob/master/src/bin/gtktest.rs

const APP_ID: &str = "com.github.pachi.visol";
const APP_NAME: &str = "ViSOL";

fn main() {
    // Comprobación del directorio de ejecución
    utils::check_current_dir();

    // Inicialización de resources
    // static_resource::init().expect("Something went wrong with the resource file initilization.");

    let app = gtk::Application::new(Some(APP_ID), Default::default())
        .expect("Failed to initialize GTK application");
    app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);

    glib::set_application_name(APP_NAME);
    glib::set_prgname(Some(APP_NAME));

    // Resource location
    // let path = "/com/rvburke/visol".to_string();
    // app.set_property_resource_base_path(Some(&path));

    // let provider = gtk::CssProvider::new();
    // provider.load_from_file("res/app.css");
    // gtk::StyleContext::add_provider_for_screen(
    //     &gdk::Screen::get_default().unwrap(),
    //     &provider,
    //     600,
    // )

    // // Set up the textdomain for gettext
    // //use gettextrs::{setlocale, LocaleCategory, bindtextdomain, textdomain};
    // setlocale(LocaleCategory::LcAll, "");
    // bindtextdomain("visol", globals::LOCALEDIR.unwrap_or("./visol/po"));
    // textdomain("visol");

    let state = Rc::new(RefCell::new(AppState::new()));
    let config = Rc::new(RefCell::new(Config::default()));

    app.connect_activate(clone!(@strong state, @strong config => move |app| {
        window::build_ui(&app, &state, &config);
    }));

    app.run(&std::env::args().collect::<Vec<_>>());
}
