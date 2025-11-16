use std::io::{Result, Error};

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[allow(dead_code)]
impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader {
            data,
            pos: 0,
        }
    }

    pub fn seek(&mut self, pos: usize) -> Result<()> {
        if pos > self.data.len() {
            return Err(Error::other("seek past end of buffer"));
        }
        self.pos = pos;
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos+1 > self.data.len() {
            return Err(Error::other("read past end of buffer"));
        }
        let data = self.data[self.pos];
        self.pos += 1;
        Ok(data)
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16_be(&mut self) -> Result<u16> {
        let b1 = self.read_u8()? as u16;
        let b2 = self.read_u8()? as u16;
        Ok((b1 << 8) | b2)
    }

    pub fn read_u16_le(&mut self) -> Result<u16> {
        let b1 = self.read_u8()? as u16;
        let b2 = self.read_u8()? as u16;
        Ok((b2 << 8) | b1)
    }

    pub fn read_u32_be(&mut self) -> Result<u32> {
        let b1 = self.read_u8()? as u32;
        let b2 = self.read_u8()? as u32;
        let b3 = self.read_u8()? as u32;
        let b4 = self.read_u8()? as u32;
        Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
    }

    pub fn read_u32_le(&mut self) -> Result<u32> {
        let b1 = self.read_u8()? as u32;
        let b2 = self.read_u8()? as u32;
        let b3 = self.read_u8()? as u32;
        let b4 = self.read_u8()? as u32;
        Ok((b4 << 24) | (b3 << 16) | (b2 << 8) | b1)
    }

    pub fn read_bytes(&mut self, data: &mut [u8]) -> Result<()> {
        let len = data.len();
        if self.pos+len > self.data.len() {
            return Err(Error::other("read past end of buffer"));
        }
        data[..].clone_from_slice(&self.data[self.pos..self.pos+len]);
        self.pos += len;
        Ok(())
    }

    pub fn read_string(&mut self, s: &mut String, len: usize) -> Result<()> {
        if self.pos+len > self.data.len() {
            return Err(Error::other("read past end of buffer"));
        }

        for c in &self.data[self.pos..self.pos+len] {
            s.push(char::from_u32(*c as u32).unwrap_or(' '));
        }
        self.pos += len;
        Ok(())
    }
}
