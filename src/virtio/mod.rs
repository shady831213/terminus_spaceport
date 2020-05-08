const MAX_QUEUE: u32 = 8;
const MAX_QUEUE_NUM: u16 = 16;
const VIRTIO_MMIO_CONFIG: u64 = 0x100;
//for queue
pub const DESC_F_NEXT: u16 = 0x1;
pub const DESC_F_WRITE: u16 = 0x2;

mod device;
mod queue;
#[cfg(test)]
mod test;