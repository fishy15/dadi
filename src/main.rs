mod config;
mod datefs;

fn main() {
    println!("{:?}", config::read_config().unwrap());
}
