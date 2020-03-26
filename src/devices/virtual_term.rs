extern crate termios;

use std::{io, fs};
use std::io::{Read, BufReader};
use std::io::Write;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use std::os::unix::io::AsRawFd;
use libc;
use self::termios::ECHONL;

#[test]
#[ignore]
fn console_hello() {
    let f_tty;
    let fd = unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 1 {
            f_tty = None;
            libc::STDIN_FILENO
        } else {
            let f = fs::File::open("/dev/tty").unwrap();
            let fd = f.as_raw_fd();
            f_tty = Some(BufReader::new(f));
            fd
        }
    };

    let old_fflag = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    //use async or mpsc is better
    unsafe { libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK) };
    let mut termios = termios::Termios::from_fd(fd).unwrap();
    let original = termios;
    termios.c_lflag &= !(ICANON | ECHO);
    termios::tcsetattr(fd, termios::TCSANOW, &termios).unwrap();
    // let read_rv = if let Some(mut f) = f_tty {
    //     f
    // } else {
    //     io::stdin().read_line(&mut rv)
    // };


    let mut reader = io::stdin();
    let stdout = io::stdout();
    stdout.lock().write("Hello World!\n".as_bytes()).unwrap();
    stdout.lock().flush().unwrap();

    loop {
        let read_len = 10;
        let mut read_buffer = vec![0 as u8; read_len];
        match reader.read(&mut read_buffer) {
            Ok(len) => {
                stdout.lock().write(format!("got {}!\n", len).as_bytes()).unwrap();
                stdout.lock().write(&read_buffer).unwrap();
                stdout.lock().flush().unwrap();
            }
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        }
    }


    stdout.lock().write("done!\n".as_bytes()).unwrap();
    stdout.lock().flush().unwrap();
    termios::tcsetattr(fd, termios::TCSANOW, &original).unwrap();
    unsafe { libc::fcntl(fd, libc::F_SETFL, old_fflag) };
}