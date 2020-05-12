extern crate terminus_spaceport;
extern crate lazy_static;

use std::thread::sleep;
use std::time::Duration;
use std::io::Write;
use std::io::Read;
use terminus_spaceport::devices::{TERM, term_exit};
use terminus_spaceport::EXIT_CTRL;
use std::sync::{Once};

struct A();

impl Drop for A {
    fn drop(&mut self) {
        println!("A drop!")
    }
}

fn cleanup() {
    Once::new().call_once(|| {
        println!("cleanup!");
        term_exit();
        let read_len = 10;
        let mut read_buffer = vec![0 as u8; read_len];
        TERM.stdin().lock().read(&mut read_buffer).unwrap();
    })
}

fn main() {
    let _a = A();
    TERM.stdout().lock().write("Hello World!\n".as_bytes()).unwrap();
    TERM.stdout().lock().flush().unwrap();
    'outer: loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            println!("{}", msg);
            break;
        }
        loop {
            if let Ok(msg) = EXIT_CTRL.poll() {
                println!("{}", msg);
                break 'outer;
            }
            let read_len = 10;
            let mut read_buffer = vec![0 as u8; read_len];
            match TERM.stdin().lock().read(&mut read_buffer) {
                Ok(len) => {
                    if read_buffer.contains(&('q' as u8)) {
                        EXIT_CTRL.exit("quit!").unwrap();
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
        sleep(Duration::from_secs(1));
    }
    TERM.stdout().lock().write("done!\n".as_bytes()).unwrap();
    TERM.stdout().lock().flush().unwrap();
    cleanup();
}