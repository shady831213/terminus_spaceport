extern crate ctrlc;
extern crate libc;
extern crate termios;

use std::io::{Stderr, Stdin, Stdout};
use std::os::unix::io::{AsRawFd, RawFd};
use std::{fs, io};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

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
        Ok(Term(origin_termios, origin_fflag, stdin_fd))
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

lazy_static! {
    pub static ref TERM: Term = Term::new().unwrap();
}

pub fn term_exit() {
    tcsetattr(TERM.2, TCSANOW, &TERM.0).unwrap();
    unsafe { libc::fcntl(TERM.2, libc::F_SETFL, TERM.1) };
}
