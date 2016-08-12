use std::error::Error;
use git;
use log::Output;
use remote::Remote;
use config::Config;

pub fn clean(config: Config) -> Result<i32, Box<Error>> {
    let output = Output::new();

    for host in config.hosts.values() {
        let remote = Remote::new(host, &output);
        try!(remote.init());
    }

    Ok(0)
}
