use std::io;
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard};
use libc;
use ansi_term::{Color, Style};

#[derive(Copy, Clone)]
pub enum Stream {
    StdOut,
    StdErr,
}

struct RawLog {
    stdout_is_tty: bool,
    stderr_is_tty: bool,
}

impl RawLog {
    fn new() -> Self {
        unsafe {
            RawLog {
                stdout_is_tty: libc::isatty(1) != 0,
                stderr_is_tty: libc::isatty(2) != 0,
            }
        }
    }

    fn use_color(&self, stream: Stream) -> bool {
        match stream {
            Stream::StdOut => self.stdout_is_tty,
            Stream::StdErr => self.stderr_is_tty,
        }
    }

    fn format_header(&self, stream: Stream, header: &str) -> String {
        let formatted = format!("[{}]", header);

        if self.use_color(stream) {
            Style::new().bold().paint(formatted).to_string()
        } else {
            formatted
        }
    }

    fn line(&self, stream: Stream, header: &str, msg: &str) {
        let formatted = format!("{} {}",
                                self.format_header(stream, header),
                                msg.trim_right());

        match stream {
            Stream::StdOut => {
                writeln!(io::stdout(), "{}", formatted).expect("failed to write stdout");
            }
            Stream::StdErr => {
                writeln!(io::stderr(), "{}", formatted).expect("failed to write stderr");
            }
        }
    }

    fn format_cmd(&self, stream: Stream, cmd: &str) -> String {
        let formatted = format!("$ {}", cmd);

        if self.use_color(stream) {
            Color::Green.bold().paint(formatted).to_string()
        } else {
            formatted
        }
    }

    fn cmd(&self, header: &str, cmd: &str) {
        let stream = Stream::StdOut;
        self.line(stream, header, &self.format_cmd(stream, cmd));
    }

    fn format_error(&self, stream: Stream, msg: &str) -> String {
        if self.use_color(stream) {
            Color::Red.bold().paint(msg).to_string()
        } else {
            msg.into()
        }
    }

    fn error(&self, header: &str, msg: &str) {
        let stream = Stream::StdErr;
        self.line(stream, header, &self.format_error(stream, msg));
    }

    fn format_success(&self, stream: Stream, msg: &str) -> String {
        if self.use_color(stream) {
            Color::Green.bold().paint(msg).to_string()
        } else {
            msg.into()
        }
    }

    fn success(&self, header: &str, msg: &str) {
        let stream = Stream::StdOut;
        self.line(stream, header, &self.format_success(stream, msg));
    }
}

#[derive(Clone)]
pub struct Output {
    log: Arc<Mutex<RawLog>>,
}

impl Output {
    pub fn new() -> Self {
        Output { log: Arc::new(Mutex::new(RawLog::new())) }
    }
}

#[derive(Clone)]
pub struct Log {
    header: String,
    log: Arc<Mutex<RawLog>>,
}

impl Log {
    pub fn new(header: &str, output: &Output) -> Self {
        Log {
            header: header.into(),
            log: output.log.clone(),
        }
    }

    fn lock(&self) -> MutexGuard<RawLog> {
        self.log.lock().expect("failed to acquire log mutex")
    }

    pub fn line(&self, stream: Stream, msg: &str) {
        self.lock().line(stream, &self.header, msg);
    }

    pub fn cmd(&self, cmd: &str) {
        self.lock().cmd(&self.header, cmd);
    }

    pub fn error(&self, msg: &str) {
        self.lock().error(&self.header, msg);
    }

    pub fn success(&self, msg: &str) {
        self.lock().success(&self.header, msg);
    }

    pub fn stream<R: Read>(&self, reader: R, stream: Stream) {
        let mut buf_reader = BufReader::new(reader);

        loop {
            let mut line = String::new();

            match buf_reader.read_line(&mut line) {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    self.line(stream, &line);
                }
                Err(err) => {
                    self.line(stream, err.description());
                    break;
                }
            }
        }
    }
}
