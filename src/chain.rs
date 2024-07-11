#[derive(Default)]
pub struct Chain(Vec<Vec<u8>>);

const CAP_MIN: usize = 2usize.pow(20);

impl Chain {
    pub fn extend(mut self, bytes: &[u8]) -> Self {
        match self.0.last_mut() {
            Some(buffer) if buffer.capacity() >= buffer.len() + bytes.len() => {
                buffer.extend(bytes);
                self
            }
            Some(_) | None => {
                let capacity = bytes.len().min(CAP_MIN);
                let mut buffer = Vec::<u8>::with_capacity(capacity);
                buffer.extend(bytes);
                self.0.push(buffer);
                self
            }
        }
    }

    pub fn push(mut self, byte: u8) -> Self {
        match self.0.last_mut() {
            Some(buffer) if buffer.capacity() > buffer.len() => {
                buffer.push(byte);
                self
            }
            Some(_) | None => {
                let mut buffer = Vec::<u8>::with_capacity(CAP_MIN);
                buffer.push(byte);
                self.0.push(buffer);
                self
            }
        }
    }

    pub fn _capacity(&self) -> usize {
        self.0.iter().map(Vec::capacity).sum()
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(Vec::len).sum()
    }

    pub fn write_to(&self, out: &mut Vec<u8>) {
        for buffer in self.0.iter() {
            out.extend(buffer);
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len());
        self.write_to(&mut out);
        out
    }
}
