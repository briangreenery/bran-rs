use std::error::Error;
use std::thread;
use std::thread::JoinHandle;
use git;
use log::{Log, Output};
use remote::Remote;
use config::{Config, Host};

fn remote_build(host: Host,
                hash: String,
                cmds: Vec<String>,
                output: Output)
                -> Result<(), Box<Error>> {

    try!(git::push(&host, &output));

    let remote = Remote::new(&host, &output);
    try!(remote.run(&format!("git reset {} --hard", hash)));

    for cmd in &cmds {
        try!(remote.run(cmd));
    }

    Ok(())
}

fn run_remote_build(host: Host, hash: String, cmds: Vec<String>, output: Output) -> bool {
    let remote_log = Log::new(&host.name, &output);

    match remote_build(host, hash, cmds, output) {
        Ok(_) => {
            remote_log.success("Build succeeded");
            true
        }
        Err(_) => {
            remote_log.error("Build failed");
            false
        }
    }
}

fn spawn_build(host: &Host,
               hash: &str,
               config: &Config,
               output: &Output)
               -> (String, JoinHandle<bool>) {

    let thread_host = host.clone();
    let thread_hash = hash.to_string();
    let thread_cmds = config.build.clone();
    let thread_output = output.clone();

    let handle = thread::spawn(move || {
        run_remote_build(thread_host, thread_hash, thread_cmds, thread_output)
    });

    (host.name.to_string(), handle)
}

fn join_all(threads: Vec<(String, JoinHandle<bool>)>) -> Vec<(String, bool)> {
    threads.into_iter().map(|pair| (pair.0, pair.1.join().unwrap_or(false))).collect()
}

fn print_summary(results: &[(String, bool)], output: &Output) {
    println!("\n---------- Summary ----------\n");

    for result in results {
        let remote_log = Log::new(&result.0, output);

        if result.1 {
            remote_log.success("Build succeeded");
        } else {
            remote_log.error("Build failed");
        }
    }
}

fn all_succeeded(results: &[(String, bool)]) -> bool {
    results.iter().all(|pair| pair.1)
}

pub fn push(config: Config) -> Result<i32, Box<Error>> {
    let output = Output::new();
    let hash = try!(git::commit(&output));

    let results = join_all(config.hosts
                                 .values()
                                 .map(|host| spawn_build(host, &hash, &config, &output))
                                 .collect());

    if results.len() > 1 {
        print_summary(&results, &output);
    }

    if all_succeeded(&results) {
        Ok(0)
    } else {
        Ok(1)
    }
}
