use std::result;
use std::sync::{Arc, Mutex};
use std::ops::Deref;

#[derive(Debug)]
pub enum Error {
    ExistedHandler(usize),
    UnknownIRQ(usize),
}

pub type Result<T> = result::Result<T, Error>;

pub struct IrqBit {
    pub enable: bool,
    pub pending: bool,
}

impl IrqBit {
    fn new() -> IrqBit {
        IrqBit {
            enable: false,
            pending: false,
        }
    }
}

struct IrqHandler(Option<Box<dyn FnMut(&IrqStatus) + 'static>>);

impl IrqHandler {
    fn new() -> IrqHandler {
        IrqHandler(None)
    }

    fn bind_handler<F: for<'r> FnMut(&'r IrqStatus) + 'static>(&mut self, handler: F) {
        self.0 = Some(Box::new(handler))
    }

    pub fn send_irq(&mut self, irq_status: &IrqStatus) -> bool {
        if let Some(ref mut handler) = self.0 {
            (*handler)(irq_status);
            true
        } else {
            false
        }
    }
}

pub struct IrqCollection<T>(Mutex<Vec<T>>);

impl<T> IrqCollection<T> {
    fn new() -> IrqCollection<T> {
        IrqCollection(Mutex::new(vec![]))
    }
    fn check_irq_num(&self, irq_num: usize) -> Result<()> {
        if irq_num >= self.0.lock().unwrap().len() {
            Err(Error::UnknownIRQ(irq_num))
        } else {
            Ok(())
        }
    }
}

pub type IrqStatus = IrqCollection<IrqBit>;

impl IrqStatus {
    pub fn enable(&self, irq_num: usize) -> Result<bool> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.lock().unwrap()[irq_num].enable)
    }

    pub fn set_enable(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.lock().unwrap()[irq_num].enable = true)
    }

    pub fn clr_enable(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.lock().unwrap()[irq_num].enable = false)
    }

    pub fn pending(&self, irq_num: usize) -> Result<bool> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.lock().unwrap()[irq_num].pending)
    }

    pub fn set_pending(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.lock().unwrap()[irq_num].pending = true)
    }

    pub fn clr_pending(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.lock().unwrap()[irq_num].pending = false)
    }
}

type IrqHandlers = IrqCollection<IrqHandler>;

struct IrqVecInner {
    status: IrqStatus,
    handlers: IrqHandlers,
}


impl IrqVecInner {
    pub fn new(len: usize) -> IrqVecInner {
        let irq = IrqVecInner {
            status: IrqStatus::new(),
            handlers: IrqHandlers::new(),
        };
        for _ in 0..len {
            irq.status.0.lock().unwrap().push(IrqBit::new());
            irq.handlers.0.lock().unwrap().push(IrqHandler::new())
        }
        irq
    }
}

pub struct IrqVec {
    vec: Arc<IrqVecInner>
}

impl IrqVec {
    pub fn new(len: usize) -> IrqVec {
        IrqVec {
            vec: Arc::new(IrqVecInner::new(len))
        }
    }
    pub fn sender(&self, irq_num: usize) -> Result<IrqVecSender> {
        self.vec.handlers.check_irq_num(irq_num)?;
        Ok(IrqVecSender {
            irq_num,
            irq_vec: Arc::clone(&self.vec),
        })
    }

    pub fn binder(&self) -> IrqVecBinder {
        IrqVecBinder {
            irq_vec: Arc::clone(&self.vec),
        }
    }
}

impl Deref for IrqVec {
    type Target = IrqStatus;
    fn deref(&self) -> &Self::Target {
        &self.vec.status
    }
}


pub struct IrqVecSender {
    irq_num: usize,
    irq_vec: Arc<IrqVecInner>,
}

impl IrqVecSender {
    pub fn send(&self) -> Result<()> {
        if !self.irq_vec.status.enable(self.irq_num)? {
            return Ok(());
        }
        if self.irq_vec.handlers.0.lock().unwrap()[self.irq_num].send_irq(&self.irq_vec.status) {
            Ok(())
        } else {
            Err(Error::UnknownIRQ(self.irq_num))
        }
    }
}

impl Clone for IrqVecSender {
    fn clone(&self) -> Self {
        IrqVecSender {
            irq_num: self.irq_num,
            irq_vec: Arc::clone(&self.irq_vec),
        }
    }
}

pub struct IrqVecBinder {
    irq_vec: Arc<IrqVecInner>,
}

impl IrqVecBinder {
    pub fn bind<F: for<'r> FnMut(&'r IrqStatus) + 'static>(&self, irq_num: usize, handler: F) -> Result<()> {
        self.irq_vec.handlers.check_irq_num(irq_num)?;
        if self.irq_vec.handlers.0.lock().unwrap()[irq_num].0.is_some() {
            Err(Error::ExistedHandler(irq_num))
        } else {
            Ok(self.irq_vec.handlers.0.lock().unwrap()[irq_num].bind_handler(handler))
        }
    }
}