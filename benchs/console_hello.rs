extern crate ctrlc;
extern crate terminus_spaceport;

use std::thread::sleep;
use std::time::Duration;
use std::process;
use std::io::Write;
use std::io::Read;
use terminus_spaceport::devices::{TERM, term_exit};

fn exit(code: i32) {
    println!("exit {}!", code);
    term_exit();
    let read_len = 10;
    let mut read_buffer = vec![0 as u8; read_len];
    TERM.stdin().lock().read(&mut read_buffer).unwrap();
    process::exit(code)
}


fn main() {
    ctrlc::set_handler(move || {
        exit(-1);
    }).expect("Error setting Ctrl-C handler");

    TERM.stdout().lock().write("Hello World!\n".as_bytes()).unwrap();
    TERM.stdout().lock().flush().unwrap();
    'outer: loop {
        loop {
            let read_len = 10;
            let mut read_buffer = vec![0 as u8; read_len];
            match TERM.stdin().lock().read(&mut read_buffer) {
                Ok(len) => {
                    if read_buffer.contains(&('q' as u8)) {
                        TERM.stdout().lock().write("quit!\n".as_bytes()).unwrap();
                        break 'outer;
                    }
                    TERM.stdout().lock().write(format!("got {}!\n", len).as_bytes()).unwrap();
                    TERM.stdout().lock().write(&read_buffer).unwrap();
                    TERM.stdout().lock().flush().unwrap();
                }
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
        }
        sleep(Duration::from_secs(5));
    }
    TERM.stdout().lock().write("done!\n".as_bytes()).unwrap();
    TERM.stdout().lock().flush().unwrap();
    exit(0);
}