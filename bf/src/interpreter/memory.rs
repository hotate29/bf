use log::trace;

pub trait Memory {
    fn get(&mut self, index: usize) -> u8 {
        *self.get_mut(index)
    }
    fn get_mut(&mut self, index: usize) -> &mut u8;
    fn inner(&self) -> &[u8];
}

impl Memory for Vec<u8> {
    fn get_mut(&mut self, index: usize) -> &mut u8 {
        &mut self[index]
    }

    fn inner(&self) -> &[u8] {
        self
    }
}

#[derive(Debug)]
pub struct AutoExtendMemory(Vec<u8>);

impl AutoExtendMemory {
    pub fn new(memory: Vec<u8>) -> Self {
        Self(memory)
    }
    #[inline]
    fn extend(&mut self, index: usize) {
        if self.0.len() <= index + 1 {
            let extend_len = self.0.len() * 2 + index + 1;

            trace!("extend! {} -> {}", self.0.len(), extend_len);
            self.0.resize(extend_len, 0);
        }
    }
}

impl Memory for AutoExtendMemory {
    #[inline]
    fn inner(&self) -> &[u8] {
        &self.0
    }
    #[inline]
    fn get_mut(&mut self, index: usize) -> &mut u8 {
        self.extend(index);
        &mut self.0[index]
    }
}
