use std::result;
use std::rc::Rc;
use std::cell::RefCell;

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

struct IrqHandler(Option<Box<dyn FnMut() + 'static>>);

impl IrqHandler {
    fn new() -> IrqHandler {
        IrqHandler(None)
    }

    fn bind_handler<F: for<'r> FnMut() + 'static>(&mut self, handler: F) {
        self.0 = Some(Box::new(handler));
    }

    pub fn send_irq(&mut self) {
        if let Some(ref mut handler) = self.0 {
            (*handler)();
        }
    }
}

pub struct IrqCollection<T>(Vec<T>);

impl<T> IrqCollection<T> {
    fn new() -> IrqCollection<T> {
        IrqCollection(vec![])
    }
    fn check_irq_num(&self, irq_num: usize) -> Result<()> {
        if irq_num >= self.0.len() {
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
        Ok(self.0[irq_num].enable)
    }

    pub fn set_enable(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].enable = true)
    }

    pub fn clr_enable(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].enable = false)
    }

    pub fn pending(&self, irq_num: usize) -> Result<bool> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].pending)
    }

    pub fn set_pending(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].pending = true)
    }

    pub fn clr_pending(&mut self, irq_num: usize) -> Result<()> {
        self.check_irq_num(irq_num)?;
        Ok(self.0[irq_num].pending = false)
    }
}

type IrqHandlers = IrqCollection<IrqHandler>;

struct IrqVecInner {
    status: IrqStatus,
    handlers: IrqHandlers,
}


impl IrqVecInner {
    pub fn new(len: usize) -> IrqVecInner {
        let mut irq = IrqVecInner {
            status: IrqStatus::new(),
            handlers: IrqHandlers::new(),
        };
        for _ in 0..len {
            irq.status.0.push(IrqBit::new());
            irq.handlers.0.push(IrqHandler::new())
        }
        irq
    }
}

pub struct IrqVec {
    vec: Rc<RefCell<IrqVecInner>>
}

impl IrqVec {
    pub fn new(len: usize) -> IrqVec {
        IrqVec {
            vec: Rc::new(RefCell::new(IrqVecInner::new(len)))
        }
    }
    pub fn sender(&self, irq_num: usize) -> Result<IrqVecSender> {
        self.vec.borrow().handlers.check_irq_num(irq_num)?;
        Ok(IrqVecSender {
            irq_num,
            irq_vec: Rc::clone(&self.vec),
        })
    }

    pub fn binder(&self) -> IrqVecBinder {
        IrqVecBinder {
            irq_vec: Rc::clone(&self.vec),
        }
    }
    pub fn enable(&self, irq_num: usize) -> Result<bool> {
        self.vec.borrow().status.enable(irq_num)
    }

    pub fn set_enable(&self, irq_num: usize) -> Result<()> {
        self.vec.borrow_mut().status.set_enable(irq_num)
    }

    pub fn clr_enable(&self, irq_num: usize) -> Result<()> {
        self.vec.borrow_mut().status.clr_enable(irq_num)
    }

    pub fn pending(&self, irq_num: usize) -> Result<bool> {
        self.vec.borrow().status.pending(irq_num)
    }

    pub fn set_pending(&self, irq_num: usize) -> Result<()> {
        self.vec.borrow_mut().status.set_pending(irq_num)
    }

    pub fn clr_pending(&self, irq_num: usize) -> Result<()> {
        self.vec.borrow_mut().status.clr_pending(irq_num)
    }
}


pub struct IrqVecSender {
    irq_num: usize,
    irq_vec: Rc<RefCell<IrqVecInner>>,
}

impl IrqVecSender {
    pub fn send(&self) -> Result<()> {
        let mut irq_vec = self.irq_vec.borrow_mut();
        irq_vec.status.clr_pending(self.irq_num)?;
        if !irq_vec.status.enable(self.irq_num)? {
            return Ok(());
        }
        irq_vec.status.set_pending(self.irq_num)?;
        irq_vec.handlers.0[self.irq_num].send_irq();
        Ok(())
    }
}

impl Clone for IrqVecSender {
    fn clone(&self) -> Self {
        IrqVecSender {
            irq_num: self.irq_num,
            irq_vec: Rc::clone(&self.irq_vec),
        }
    }
}

pub struct IrqVecBinder {
    irq_vec: Rc<RefCell<IrqVecInner>>,
}

impl IrqVecBinder {
    pub fn bind<F: for<'r> FnMut() + 'static>(&self, irq_num: usize, handler: F) -> Result<()> {
        let mut irq_vec = self.irq_vec.borrow_mut();
        irq_vec.handlers.check_irq_num(irq_num)?;
        if irq_vec.handlers.0[irq_num].0.is_some() {
            Err(Error::ExistedHandler(irq_num))
        } else {
            Ok(irq_vec.handlers.0[irq_num].bind_handler(handler))
        }
    }
}
