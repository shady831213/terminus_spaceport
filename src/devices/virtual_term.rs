extern crate termios;
extern crate libc;
extern crate ctrlc;

use std::{io, fs};
use std::io::{Stdout, Stdin, Stderr};
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel, TryRecvError};

pub struct Term(Termios, libc::c_int, RawFd);

impl Term {
    fn new() -> io::Result<Term> {
        let stdin_fd = unsafe {
            if libc::isatty(libc::STDIN_FILENO) == 1 {
                libc::STDIN_FILENO
            } else {
                let f = fs::File::open("/dev/tty")?;
                f.as_raw_fd()
            }
        };
        let origin_fflag = unsafe { libc::fcntl(stdin_fd, libc::F_GETFL) };
        unsafe { libc::fcntl(stdin_fd, libc::F_SETFL, libc::O_NONBLOCK) };
        let mut termios = Termios::from_fd(stdin_fd)?;
        let origin_termios = termios;
        termios.c_lflag &= !(ICANON | ECHO);
        tcsetattr(stdin_fd, TCSANOW, &termios)?;
        println!("create term!");
        Ok(Term(
            origin_termios,
            origin_fflag,
            stdin_fd,
        ))
    }
    pub fn stdin(&self) -> Stdin {
        io::stdin()
    }
    pub fn stdout(&self) -> Stdout {
        io::stdout()
    }
    pub fn stderr(&self) -> Stderr {
        io::stderr()
    }
}

lazy_static!(
    pub static ref TERM: Term = Term::new().unwrap();
);

pub fn term_exit() {
    tcsetattr(TERM.2, TCSANOW, &TERM.0).unwrap();
    unsafe { libc::fcntl(TERM.2, libc::F_SETFL, TERM.1) };
}


struct CtrlCInner {
    receiver: Receiver<String>,
    received: Option<String>,
}

impl CtrlCInner {
    fn new() -> CtrlCInner {
        let (sender, receiver) = channel();
        ctrlc::set_handler(move || {
            sender.send("Catch Ctrl-C!".to_string()).unwrap()
        }).expect("Error setting Ctrl-C handler");
        CtrlCInner {
            receiver,
            received: None,
        }
    }

    fn poll(&mut self) -> Result<String, TryRecvError> {
        if let Some(ref res) = self.received {
            Ok(res.clone())
        } else {
            let res = self.receiver.try_recv()?;
            self.received = Some(res.clone());
            Ok(res)
        }
    }
}

pub struct CtrlC {
    inner: Mutex<CtrlCInner>,
}

impl CtrlC {
    fn new() -> CtrlC {
        CtrlC{
            inner:Mutex::new(CtrlCInner::new())
        }
    }
    pub fn poll(&self) -> Result<String, TryRecvError> {
        self.inner.lock().unwrap().poll()
    }
}

lazy_static!(
    pub static ref CTRL_C:CtrlC = CtrlC::new();
);
