use std::{time::Duration, thread};

use fltk::app::App;
use fltk::{prelude::*};
mod connection;
use fltk::*;
use fltk_grid::Grid;
use fltk::input::Input;

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
    //wind.show();
    button.set_callback(move |_| frame.set_label("Hello World!"));
    app.run().unwrap();
    // crate::connection::_get_ctconn("config.toml".to_string(), &mut std::io::stdin());
    // the linux syscall will essentially not block
    // see documentation
    thread::sleep(Duration::from_millis(10));
    println!("here");
    let app = app::App::default();
    let mut wind = Window::default().with_size(320, 480).with_label("Counter");
    let mut grid = Grid::default_fill();

    grid.debug(true);
    grid.set_layout(5, 5);
    let mut button = button::Button::default();
    button.set_label("send");
    // grid.insert(&mut button, 0..2, 0..2);
    let mut fqdn = Input::default().with_size(30, 1);
    let mut username = Input::default();
    let mut password = input::SecretInput::default();
    grid.insert(&mut fqdn, 1, 2..5);
    fqdn.set_size(250, 30);
    fqdn.set_maximum_size(128);
    fqdn.set_label_size(128);
    grid.insert(&mut username, 2, 2..5);
    grid.insert(&mut password, 3, 2..5);
    fqdn.set_maximum_size(30);
    username.set_maximum_size(30);
    password.set_maximum_size(30);
    let mut fqdn_label = frame::Frame::default();
    let mut username_label = frame::Frame::default();
    let mut password_label = frame::Frame::default();
    fqdn_label.set_label("FQDN von Churchtools: (z.B. xxx.church.tools)");
    username_label.set_label("username:");
    password_label.set_label("password:");
    grid.insert(&mut fqdn_label, 1, 0..2);
    grid.insert(&mut username_label, 2, 0..2);
    grid.insert(&mut password_label, 3, 0..2);
    wind.end();
    button.set_callback(move |_| app.quit());
    //wind.show();

    app.run().unwrap();

    thread::sleep(Duration::from_millis(100));

    let mut app = App::default();
    let mut window = Window::new(100, 100, 550, 135, "birthdays app");
    let mut fqdn = Input::new(150, 5, 300, 30, "fqdn of churchtools: ");
    let mut username = Input::new(150, 35, 300, 30, "username:");
    let mut password = fltk::input::SecretInput::new(150, 65, 300, 30, "password:");
    let mut button = Button::new(300, 100, 60, 30, "send");
    fqdn.set_tooltip("A \"fully qualified domain name\" (fqdn) \
        is a part of an url/link. It defines uniquely an IP adress and \
        therefore a server. The fqdn that is  needed here is the fqdn \
        from churchtools, without a trailing point \".\". An example \
        would be \"xxx.church.tools\" where \"xxx\" stand for the acronym your church has.");
    window.end();
    window.show();
    app.run().unwrap();
    
//    println!("{}", fqdn.value());
}