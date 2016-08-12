extern crate ansi_term;
extern crate libc;
extern crate yaml_rust;
extern crate time;

#[macro_use]
extern crate clap;

mod cli;
mod lib;

// use std::error::Error;
// use std::io;
// use std::io::Write;
// use std::thread;
// use std::thread::JoinHandle;

// use log::{Log, Output};
// use clap::{App, AppSettings, Arg, SubCommand};
// use remote::Remote;
// use config::{Config, Host};

fn run() -> Result<i32, Box<Error>> {
    let app = App::new("bran")
                  .version(crate_version!())
                  .about("A command line remote builder")
                  .subcommand(SubCommand::with_name("init").about("Initialize bran"))
                  .subcommand(SubCommand::with_name("push").about("Push files to all hosts"))
                  .subcommand(SubCommand::with_name("clean")
                                  .about("Clean the build directory on all hosts"))
                  .subcommand(SubCommand::with_name("build")
                                  .about("Push files to all hosts and run the build command"))
                  .subcommand(SubCommand::with_name("run")
                                  .about("Run an ad-hoc command on all hosts")
                                  .setting(AppSettings::TrailingVarArg)
                                  .arg(Arg::from_usage("<cmd>... 'command to run'")));

    let matches = app.clone().get_matches();

    if matches.subcommand_matches("init").is_some() {
        init()
    } else if matches.subcommand_matches("build").is_some() {
        build()
    } else if let Some(cmd) = matches.subcommand_matches("run") {
        run(cmd.values_of("cmd").unwrap().collect())
    } else {
        try!(app.print_help());
        println!("");
        Ok(2)
    }
}

fn main() {
    match run() {
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
        Err(err) => {
            writeln!(io::stderr(), "Error: {}", err.description()).ok();
            std::process::exit(1);
        }
    }
}
