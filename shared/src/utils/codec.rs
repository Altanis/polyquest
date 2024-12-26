/// A binary encoder/decoder.
#[derive(Default, Debug, Clone)]
pub struct BinaryCodec {
    data: Vec<u8>,
    index: usize
}

impl BinaryCodec {
    pub fn new() -> BinaryCodec {
        BinaryCodec::default()
    }

    pub fn from_bytes(data: Vec<u8>) -> BinaryCodec {
        BinaryCodec { data, index: 0 }
    }

    pub fn encode_bool(&mut self, v: bool) {
        self.data.push(if v { 1 } else { 0 });
        self.index += 1;
    }

    pub fn decode_bool(&mut self) -> Option<bool> {
        self.index += 1;

        match self.data.get(self.index - 1) {
            Some(0) => Some(false),
            Some(1) => Some(true),
            _ => None
        }
    }

    pub fn encode_varint(&mut self, mut v: i64) {
        loop {
            let mut byte = (v & 0b01111111) as u8;
            v >>= 7;

            if v != 0 {
                byte |= 0b10000000;
            }

            self.data.push(byte);
            self.index += 1;
            
            if v == 0 {
                break;
            }
        }
    }

    pub fn decode_varint(&mut self) -> Option<i64> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = self.data.get(self.index)?;
            self.index += 1;
            result |= ((byte & 0b01111111) as i64) << shift;
            shift += 7;
            if byte & 0b10000000 == 0 {
                break;
            }
        }

        Some(result)
    }

    pub fn encode_varuint(&mut self, mut v: u64) {
        loop {
            let mut byte = (v & 0b01111111) as u8;
            v >>= 7;

            if v != 0 {
                byte |= 0b10000000;
            }

            self.data.push(byte);
            self.index += 1;
            
            if v == 0 {
                break;
            }
        }
    }

    pub fn decode_varuint(&mut self) -> Option<u64> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = self.data.get(self.index)?;
            self.index += 1;
            result |= ((byte & 0b01111111) as u64) << shift;
            shift += 7;
            if byte & 0b10000000 == 0 {
                break;
            }
        }

        Some(result)
    }

    pub fn encode_f32(&mut self, v: f32) {
        self.data.extend_from_slice(&v.to_le_bytes());
        self.index += 4;
    }

    pub fn decode_f32(&mut self) -> Option<f32> {
        if self.index + 4 > self.data.len() {
            return None;
        }
    
        let value = f32::from_le_bytes(self.data[self.index..self.index + 4].try_into().ok()?);
        self.index += 4;
    
        Some(value)
    }

    pub fn encode_f64(&mut self, v: f64) {
        self.data.extend_from_slice(&v.to_le_bytes());
        self.index += 8;
    }

    pub fn decode_f64(&mut self) -> Option<f64> {
        if self.index + 8 > self.data.len() {
            return None;
        }
    
        let value = f64::from_le_bytes(self.data[self.index..self.index + 8].try_into().ok()?);
        self.index += 8;
    
        Some(value)
    }

    pub fn encode_string(&mut self, v: String) {
        self.encode_varuint(v.len() as u64);
        self.data.extend_from_slice(v.as_bytes());
        self.index += v.len();
    }

    pub fn decode_string(&mut self) -> Option<String> {
        let len = self.decode_varuint()? as usize;
        let result = String::from_utf8(self.data[self.index..self.index + len].to_vec()).ok()?;
        self.index += len;

        Some(result)
    }

    pub fn backspace(&mut self) {
        self.data.pop();
        self.index -= 1;
    }

    pub fn out(&self) -> Vec<u8> {
        self.data[0..self.index].to_vec()
    }

    pub fn dump_buffer(&self) -> Vec<u8> {
        self.data[self.index..self.data.len()].to_vec()
    }
}