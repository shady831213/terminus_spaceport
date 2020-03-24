use std::cell::RefCell;
use std::collections::HashMap;
use std::result;

#[derive(Debug)]
pub enum Error {
    ExistedHandler(usize),
    UnknownIRQ(usize),
    HandlerError(String)
}

pub type Result<T> = result::Result<T, Error>;

pub trait IrqHandler {
    fn handle(&mut self) -> Result<()>;
}

pub struct IrqBit {
    pub enable: bool,
    pub pending: bool,
    handler: Option<RefCell<Box<dyn IrqHandler>>>,
}

impl IrqBit {
    fn new() -> IrqBit {
        IrqBit {
            enable: false,
            pending: false,
            handler: None,
        }
    }
    fn bind_handler<T: IrqHandler+'static>(&mut self, handler: T) {
        self.handler = Some(RefCell::new(Box::new(handler)))
    }
    pub fn send_irq(&self) -> Option<Result<()>> {
        if let Some(ref handler) = self.handler {
            if self.enable && self.pending {
                Some(handler.borrow_mut().handle())
            } else {
                Some(Ok(()))
            }
        } else {
            None
        }
    }
}

pub struct IrqSignal(Vec<IrqBit>);


impl IrqSignal {
    fn new(len: usize) -> IrqSignal {
        let mut irq = IrqSignal(vec![]);
        for i in 0..len {
            irq.0.push(IrqBit::new())
        }
        irq
    }

    fn check_irq_num(&self, irq_num: usize) -> Result<()> {
        if irq_num >= self.0.len() {
            Err(Error::UnknownIRQ(irq_num))
        } else {
            Ok(())
        }
    }

    pub fn enable(&self, irq_num: usize) -> Result<bool> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].enable)
    }

    pub fn pending(&self, irq_num: usize) -> Result<bool> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].pending)
    }

    pub fn set_enable(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].enable = true)
    }

    pub fn set_pending(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].pending = true)
    }

    pub fn clr_enable(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].enable = false)
    }

    pub fn clr_pending(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].pending = false)
    }

    pub fn bind_handler<T: IrqHandler+'static>(&mut self, irq_num: usize, handler: T) -> Result<()> {
        self.check_irq_num(irq_num)?;
        if self.0[irq_num].handler.is_some() {
            Err(Error::ExistedHandler(irq_num))
        } else {
            Ok(self.0[irq_num].bind_handler(handler))
        }
    }

    pub fn send_irq(&self, irq_num:usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        if let Some(res) = self.0[irq_num].send_irq() {
            res
        } else {
            Err(Error::UnknownIRQ(irq_num))
        }
    }
}
