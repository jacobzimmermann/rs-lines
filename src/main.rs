use std::env;

#[macro_use]
extern crate glib;
extern crate gio;
extern crate gtk;

use gio::prelude::*;

mod app;
mod lines_area;
mod window;

fn main() {
    gtk::init().expect("Failed to initialise GTK");

    let app = app::LinesApp::new();
    let args: Vec<String> = env::args().collect();
    app.run(&args);
}
