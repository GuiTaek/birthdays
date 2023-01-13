use std::{time::Duration, thread};

use fltk::{prelude::*};
mod connection;
use fltk::*;

use fltk::{app, button::Button, frame::Frame, group::Flex, prelude::*, window::Window};

#[allow(unused)]
fn main() {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    let mut flex = group::Flex::default().center_of_parent().column();
    let mut but_inc = button::Button::default().with_label("+");
    let mut button = fltk::button::Button::new(160, 210, 80, 40, "Click");
    let mut frame = fltk::frame::Frame::new(0, 0, 400, 200, "");
    wind.end();
    wind.show();
    button.set_callback(move |_| frame.set_label("Hello World!"));
    app.run().unwrap();
    // crate::connection::_get_ctconn("config.toml".to_string(), &mut std::io::stdin());
    // the linux syscall will essentially not block
    // see documentation
    thread::sleep(Duration::from_millis(10));
    println!("here");
    let app = app::App::default();
    let mut wind = Window::default().with_size(160, 200).with_label("Counter");
    let mut flex = Flex::default().with_size(120, 140).center_of_parent().column();
    let mut but_inc = Button::default().with_label("+");
    let mut frame = Frame::default().with_label("0");
    let mut but_dec = Button::default().with_label("-");
    flex.end();
    wind.end();
    wind.show();
    app.run().unwrap();
}
