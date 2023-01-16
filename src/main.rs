use std::{time::Duration, thread};

use fltk::app::App;
use fltk::{prelude::*};
mod connection;
use fltk::*;
//use fltk_grid::Grid;
use fltk::input::Input;

use fltk::{app, button::Button, frame::Frame, group::Flex, prelude::*, window::Window};

#[allow(unused)]
fn main() {
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
