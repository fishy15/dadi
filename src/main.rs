mod config;

fn main() {
    println!("{:?}", config::read_config().unwrap());
}
