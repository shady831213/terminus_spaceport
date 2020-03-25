use std::cell::RefCell;
use std::result;
use std::marker::PhantomData;
use std::sync::Arc;
use std::ops::Deref;

#[derive(Debug)]
pub enum Error {
    ExistedHandler(usize),
    UnknownIRQ(usize),
}

pub type Result<T> = result::Result<T, Error>;

pub struct IrqBit {
    pub enable: bool,
    handler: Option<Box<dyn FnMut() + 'static>>,
}

impl IrqBit {
    fn new() -> IrqBit {
        IrqBit {
            enable: false,
            handler: None,
        }
    }
    fn bind_handler<F: FnMut() + 'static>(&mut self, handler: F) {
        self.handler = Some(Box::new(handler))
    }

    pub fn send_irq(&mut self) -> Option<Result<()>> {
        if let Some(ref mut handler) = self.handler {
            if self.enable {
                Some(Ok((*handler)()))
            } else {
                Some(Ok(()))
            }
        } else {
            None
        }
    }
}

struct IrqVecInner(RefCell<Vec<IrqBit>>);


impl IrqVecInner {
    pub fn new(len: usize) -> IrqVecInner {
        let irq = IrqVecInner(RefCell::new(vec![]));
        for _ in 0..len {
            irq.0.borrow_mut().push(IrqBit::new())
        }
        irq
    }

    fn check_irq_num(&self, irq_num: usize) -> Result<()> {
        if irq_num >= self.0.borrow().len() {
            Err(Error::UnknownIRQ(irq_num))
        } else {
            Ok(())
        }
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
    pub fn enable(&self, irq_num: usize) -> Result<bool> {
        self.vec.check_irq_num(irq_num)?;
        Ok(self.vec.0.borrow()[irq_num].enable)
    }

    pub fn set_enable(&self, irq_num: usize) -> Result<()> {
        self.vec.check_irq_num(irq_num)?;
        Ok(self.vec.0.borrow_mut()[irq_num].enable = true)
    }

    pub fn clr_enable(&self, irq_num: usize) -> Result<()> {
        self.vec.check_irq_num(irq_num)?;
        Ok(self.vec.0.borrow_mut()[irq_num].enable = false)
    }

    pub fn sender(&self) -> IrqVecSender {
        IrqVecSender {
            irq_vec: Arc::clone(&self.vec),
        }
    }

    pub fn binder(&self) -> IrqVecBinder {
        IrqVecBinder {
            irq_vec: Arc::clone(&self.vec),
        }
    }
}


pub struct IrqVecSender {
    irq_vec: Arc<IrqVecInner>,
}

impl IrqVecSender {
    pub fn send(&self, irq_num: usize) -> Result<()> {
        self.irq_vec.check_irq_num(irq_num)?;
        if let Some(res) = self.irq_vec.0.borrow_mut()[irq_num].send_irq() {
            res
        } else {
            Err(Error::UnknownIRQ(irq_num))
        }
    }
}

pub struct IrqVecBinder {
    irq_vec: Arc<IrqVecInner>,
}

impl IrqVecBinder {
    pub fn bind<F: FnMut() + 'static>(&self, irq_num: usize, handler: F) -> Result<()> {
        self.irq_vec.check_irq_num(irq_num)?;
        if self.irq_vec.0.borrow()[irq_num].handler.is_some() {
            Err(Error::ExistedHandler(irq_num))
        } else {
            Ok(self.irq_vec.0.borrow_mut()[irq_num].bind_handler(handler))
        }
    }
}