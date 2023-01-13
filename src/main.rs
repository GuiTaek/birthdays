use std::{time::Duration, thread};


mod connection;




#[allow(unused)]
#[tokio::main]
async fn main() {
    // crate::connection::_get_ctconn("config.toml".to_string(), &mut std::io::stdin());
    // the linux syscall will essentially not block
    // see documentation
    thread::sleep(Duration::from_millis(10));
}
