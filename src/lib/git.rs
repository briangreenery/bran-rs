use std::io;
use std::io::{Read, Write};
use std::fs::File;
use std::path::Path;
use std::process::Command;
use time;

use log::{Log, Output};
use cmd::run;
use config::Host;

fn git(log: &Log, args: &[&str], git_ssh_command: Option<String>) -> Result<(), io::Error> {
    let mut args_with_tree = Vec::new();
    args_with_tree.push("--git-dir=.bran");
    args_with_tree.push("--work-tree=.");
    args_with_tree.extend_from_slice(args);

    let mut command = Command::new("git");
    command.args(&args_with_tree);

    if let Some(git_ssh_command) = git_ssh_command {
        command.env("GIT_SSH_COMMAND", git_ssh_command);
    }

    log.cmd(&format!("git {}", args_with_tree.join(" ")));
    run(command, &log)
}

fn write_file(path: &Path, content: &str, log: &Log) -> Result<(), io::Error> {
    log.cmd(&format!("echo \"{}\" > {}", content, path.to_str().unwrap()));
    let mut file = try!(File::create(path));
    try!(file.write(content.as_bytes()));
    Ok(())
}

pub fn init(output: &Output) -> Result<(), io::Error> {
    let log = Log::new("local", output);

    try!(git(&log, &["init"], None));

    try!(write_file(&Path::new(".bran").join("info").join("exclude"),
                    ".bran",
                    &log));

    try!(write_file(&Path::new(".bran").join("info").join("attributes"),
                    "* -filter -diff -merge -text",
                    &log));

    try!(git(&log, &["config", "user.name", "Brandon Stark"], None));
    try!(git(&log, &["config", "user.email", "bran.stark@example.com"], None));
    try!(git(&log, &["config", "core.autocrlf", "false"], None));
    try!(git(&log, &["config", "core.ignorecase", "false"], None));
    try!(git(&log, &["config", "commit.gpgsign", "false"], None));
    Ok(())
}

fn head() -> Result<String, io::Error> {
    let mut file = try!(File::open(Path::new(".bran").join("refs").join("heads").join("master")));
    let mut contents = String::new();
    try!(file.read_to_string(&mut contents));
    Ok(contents.trim().into())
}

pub fn commit(output: &Output) -> Result<String, io::Error> {
    let log = Log::new("local", output);
    let msg = format!("{}", time::now().rfc822z());

    git(&log, &["add", "-A", "."], None).ok();
    git(&log, &["commit", "-m", &msg], None).ok();
    head()
}

pub fn push(host: &Host, output: &Output) -> Result<(), io::Error> {
    let log = Log::new("local", output);
    let url = host.git_ssh_url();
    let env = host.git_ssh_command();
    try!(git(&log, &["push", "-f", &url, "master:bran"], env));
    Ok(())
}
