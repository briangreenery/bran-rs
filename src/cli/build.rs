use std::error::Error;
use std::thread;
use std::thread::JoinHandle;
use git;
use log::{Log, Output};
use remote::Remote;
use config::{Config, Host};

pub fn build(config: Config) -> Result<i32, Box<Error>> {
    let output = Output::new();
    let hash = try!(git::commit(&output));

    spawn()
}
