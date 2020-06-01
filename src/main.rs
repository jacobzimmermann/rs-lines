#[macro_use]
extern crate glib;

mod lines_area;
mod window;
mod app;

fn main() -> Result<(), glib::BoolError> {
    use gio::prelude::*;
    use std::env;

    app::Lines::initialise().map(|app| {
        let args: Vec<String> = env::args().collect();
        app.run(&args);
    })
}
