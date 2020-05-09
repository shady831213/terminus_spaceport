const MAX_QUEUE: u32 = 8;
const MAX_QUEUE_NUM: u16 = 16;

const MMIO_MAGIC_VALUE: u64 = 0x000;
const MMIO_VERSION: u64 = 0x004;
const MMIO_DEVICE_ID: u64 = 0x008;
const MMIO_VENDOR_ID: u64 = 0x00c;
const MMIO_DEVICE_FEATURES: u64 = 0x010;
// const MMIO_DEVICE_FEATURES_SEL:u64 = 0x014;
const MMIO_DRIVER_FEATURES: u64 = 0x020;
// const MMIO_DRIVER_FEATURES_SEL:u64 = 0x024;
// const MMIO_GUEST_PAGE_SIZE:u64 = 0x028; /* version 1 only */
const MMIO_QUEUE_SEL: u64 = 0x030;
const MMIO_QUEUE_NUM_MAX: u64 = 0x034;
const MMIO_QUEUE_NUM: u64 = 0x038;
// const MMIO_QUEUE_ALIGN:u64 = 0x03c; /* version 1 only */
// const MMIO_QUEUE_PFN:u64 = 0x040; /* version 1 only */
const MMIO_QUEUE_READY: u64 = 0x044;
const MMIO_QUEUE_NOTIFY: u64 = 0x050;
const MMIO_INTERRUPT_STATUS: u64 = 0x060;
const MMIO_INTERRUPT_ACK: u64 = 0x064;
const MMIO_STATUS: u64 = 0x070;
const MMIO_QUEUE_DESC_LOW: u64 = 0x080;
const MMIO_QUEUE_DESC_HIGH: u64 = 0x084;
const MMIO_QUEUE_AVAIL_LOW: u64 = 0x090;
const MMIO_QUEUE_AVAIL_HIGH: u64 = 0x094;
const MMIO_QUEUE_USED_LOW: u64 = 0x0a0;
const MMIO_QUEUE_USED_HIGH: u64 = 0x0a4;
// const MMIO_CONFIG_GENERATION:u64 = 0x0fc;
const MMIO_CONFIG: u64 = 0x100;

//for queue
pub const DESC_F_NEXT: u16 = 0x1;
pub const DESC_F_WRITE: u16 = 0x2;

mod device;

pub use device::*;

mod queue;

pub use queue::*;

pub struct VirtIOInfo {
    pub base: u64,
    pub size: u64,
    pub irq_id: u32,
    pub ty: String,
}

#[cfg(test)]
mod test;