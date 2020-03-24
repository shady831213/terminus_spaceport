use std::cell::RefCell;
use std::collections::HashMap;
use std::result;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Error {
    ExistedHandler(usize),
    UnknownIRQ(usize),
}

pub type Result<T> = result::Result<T, Error>;

pub struct IrqBit<'a> {
    pub enable: bool,
    pub pending: bool,
    handler: Option<Box<dyn FnMut() + 'a>>,
}

impl<'a> IrqBit<'a> {
    fn new() -> IrqBit<'a> {
        IrqBit {
            enable: false,
            pending: false,
            handler: None,
        }
    }
    fn bind_handler<F: FnMut() + 'a>(&mut self, handler: F) {
        self.handler = Some(Box::new(handler))
    }

    pub fn send_irq(&mut self) -> Option<Result<()>> {
        if let Some(ref mut handler) = self.handler {
            if self.enable && self.pending {
                Some(Ok((*handler)()))
            } else {
                Some(Ok(()))
            }
        } else {
            None
        }
    }
}

pub struct IrqSignal<'a>(RefCell<Vec<IrqBit<'a>>>);


impl<'a> IrqSignal<'a> {
    pub fn new(len: usize) -> IrqSignal<'a> {
        let mut irq = IrqSignal(RefCell::new(vec![]));
        for i in 0..len {
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

    pub fn enable(&self, irq_num: usize) -> Result<bool> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.borrow()[irq_num].enable)
    }

    pub fn pending(&self, irq_num: usize) -> Result<bool> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.borrow()[irq_num].pending)
    }

    pub fn set_enable(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.borrow_mut()[irq_num].enable = true)
    }

    pub fn set_pending(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.borrow_mut()[irq_num].pending = true)
    }

    pub fn clr_enable(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.borrow_mut()[irq_num].enable = false)
    }

    pub fn clr_pending(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0.borrow_mut()[irq_num].pending = false)
    }

    pub fn bind_handler<F: FnMut() + 'a>(&self, irq_num: usize, handler: F) -> Result<()> {
        self.check_irq_num(irq_num)?;
        if self.0.borrow()[irq_num].handler.is_some() {
            Err(Error::ExistedHandler(irq_num))
        } else {
            Ok(self.0.borrow_mut()[irq_num].bind_handler(handler))
        }
    }

    pub fn send_irq(&self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        if let Some(res) = self.0.borrow_mut()[irq_num].send_irq() {
            res
        } else {
            Err(Error::UnknownIRQ(irq_num))
        }
    }
}
