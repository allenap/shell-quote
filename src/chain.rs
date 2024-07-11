#[derive(Default)]
pub enum Chain {
    Link(Vec<u8>, Box<Chain>),
    #[default]
    End,
}

const CAP_MIN: usize = 2usize.pow(22);

impl Chain {
    pub fn extend(mut self, bytes: &[u8]) -> Self {
        match self {
            Self::Link(ref mut buffer, _) if buffer.capacity() >= buffer.len() + bytes.len() => {
                buffer.extend(bytes);
                self
            }
            Self::Link(_, _) | Self::End => {
                let capacity = bytes.len().min(CAP_MIN);
                let mut buffer = Vec::<u8>::with_capacity(capacity);
                buffer.extend(bytes);
                Self::Link(buffer, self.into())
            }
        }
    }

    pub fn push(mut self, byte: u8) -> Self {
        match self {
            Self::Link(ref mut buffer, _) if buffer.capacity() > buffer.len() => {
                buffer.push(byte);
                self
            }
            Self::Link(_, _) | Self::End => {
                let mut buffer = Vec::<u8>::with_capacity(CAP_MIN);
                buffer.push(byte);
                Self::Link(buffer, self.into())
            }
        }
    }

    pub fn _capacity(&self) -> usize {
        match self {
            Self::Link(buffer, prev) => buffer.capacity() + prev._capacity(),
            Self::End => 0,
        }
    }

    pub fn _len(&self) -> usize {
        match self {
            Self::Link(buffer, prev) => buffer.len() + prev._len(),
            Self::End => 0,
        }
    }

    pub fn write_to(&self, out: &mut Vec<u8>) {
        if let Self::Link(buffer, prev) = self {
            prev.write_to(out);
            out.extend(buffer);
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.to_vec_help(0)
    }

    fn to_vec_help(&self, len: usize) -> Vec<u8> {
        match self {
            Self::Link(buffer, prev) => {
                let len = len + buffer.len();
                let mut out = prev.to_vec_help(len);
                out.extend(buffer);
                out
            }
            Self::End => Vec::<u8>::with_capacity(len),
        }
    }
}
