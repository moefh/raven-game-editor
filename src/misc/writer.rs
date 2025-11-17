pub struct Writer {
    pub data: Vec<u8>,
}

#[allow(dead_code)]
impl Writer {
    pub fn new() -> Self {
        Writer {
            data: Vec::new(),
        }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.data.push(v);
    }

    pub fn write_u16_le(&mut self, v: u16) {
        let b1 = (v & 0xff) as u8;
        let b2 = (v >> 8) as u8;
        self.data.push(b1);
        self.data.push(b2);
    }

    pub fn write_u16_be(&mut self, v: u16) {
        let b2 = (v & 0xff) as u8;
        let b1 = (v >> 8) as u8;
        self.data.push(b1);
        self.data.push(b2);
    }

    pub fn write_u32_le(&mut self, v: u32) {
        let b1 = (v & 0xff) as u8;
        let b2 = ((v >>  8) & 0xff) as u8;
        let b3 = ((v >> 16) & 0xff) as u8;
        let b4 = ((v >> 24) & 0xff) as u8;
        self.data.push(b1);
        self.data.push(b2);
        self.data.push(b3);
        self.data.push(b4);
    }

    pub fn write_u32_be(&mut self, v: u32) {
        let b4 = (v & 0xff) as u8;
        let b3 = ((v >>  8) & 0xff) as u8;
        let b2 = ((v >> 16) & 0xff) as u8;
        let b1 = ((v >> 24) & 0xff) as u8;
        self.data.push(b1);
        self.data.push(b2);
        self.data.push(b3);
        self.data.push(b4);
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_n_bytes(&mut self, data: &[u8], pad: u8, len: usize) {
        if data.len() >= len {
            self.data.extend_from_slice(&data[0..len]);
        } else {
            self.data.extend_from_slice(&data);
            for _ in data.len()..len {
                self.data.push(pad);
            }
        }
    }
}
