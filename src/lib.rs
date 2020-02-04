mod list;

#[cfg(test)]
mod test;

pub struct MemInfo {
    pub addr: u64,
    pub size: u64,
}