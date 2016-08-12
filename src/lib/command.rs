use std::io;
use std::process::{Command, Stdio};
use std::thread;
use log::{Log, Stream};

pub fn run(cmd: &str, args: &[&str], log: &Log) -> Result<(), io::Error> {
    let mut child = try!(command.stdin(Stdio::null())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn());

    let stderr = child.stderr.take().expect("missing child stderr");
    let stdout = child.stdout.take().expect("missing child stdout");

    let stderr_log = log.clone();
    let stderr_thread = thread::spawn(move || {
        stderr_log.stream(stderr, Stream::StdErr);
    });

    log.stream(stdout, Stream::StdOut);
    stderr_thread.join().expect("failed to join stderr_thread");

    match child.wait() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::Other, format!("Command failed with {}", status)))
            }
        }
        Err(err) => Err(err),
    }
}
