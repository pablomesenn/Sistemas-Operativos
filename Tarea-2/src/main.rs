use std::thread;
use std::time::Duration;
mod factory;

fn main() {
    let factory = factory::Factory::new("Main Factory".to_string());
    factory.operate();
}