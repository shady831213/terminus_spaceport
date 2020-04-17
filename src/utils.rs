use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError, SendError};
use std::sync::Mutex;

struct ExitCtrlInner {
    sender: Sender<String>,
    receiver: Receiver<String>,
    received: Option<String>,
}

impl ExitCtrlInner {
    fn new() -> ExitCtrlInner {
        let (sender, receiver) = channel();
        let ctrc_sender = Sender::clone(&sender);
        ctrlc::set_handler(move || {
            ctrc_sender.send("Catch Ctrl-C!".to_string()).unwrap()
        }).expect("Error setting Ctrl-C handler");
        ExitCtrlInner {
            sender: sender,
            receiver,
            received: None,
        }
    }

    fn reset(&mut self) {
        self.received = None;
        loop {
            if self.receiver.try_recv().is_err() {
                break
            }
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

    fn exit(&self, msg: &str) -> Result<(), SendError<String>> {
        let sender = Sender::clone(&self.sender);
        sender.send(msg.to_string())
    }
}

pub struct ExitCtrl {
    inner: Mutex<ExitCtrlInner>,
}

impl ExitCtrl {
    fn new() -> ExitCtrl {
        ExitCtrl {
            inner: Mutex::new(ExitCtrlInner::new())
        }
    }
    pub fn poll(&self) -> Result<String, TryRecvError> {
        self.inner.lock().unwrap().poll()
    }
    pub fn reset(&self) {
        self.inner.lock().unwrap().reset()
    }
    pub fn exit(&self, msg: &str) -> Result<(), SendError<String>> {
        self.inner.lock().unwrap().exit(msg)
    }
}

lazy_static!(
    pub static ref EXIT_CTRL:ExitCtrl = ExitCtrl::new();
);