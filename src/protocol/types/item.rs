use crate::utils::BinaryStream;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: i32,
    pub meta: i32,
    pub count: u32,
    pub nbt: Vec<u8>,
}

impl Default for Item {
    fn default() -> Self { Self::air() }
}

impl Item {
    pub fn new(id: i32, meta: i32, count: u32) -> Self {
        Self { id, meta, count, nbt: Vec::new() }
    }

    pub fn air() -> Self {
        Self { id: 0, meta: 0, count: 0, nbt: Vec::new() }
    }

    pub fn is_air(&self) -> bool {
        self.id <= 0
    }

    pub fn get(&mut self, stream: &mut BinaryStream) {
        let id = stream.get_var_int().unwrap_or(0);
        if id <= 0 {
            *self = Self::air();
            return;
        }
        let aux = stream.get_var_int().unwrap_or(0);
        let mut data = (aux >> 8) as i32;
        if data == 0x7fff {
            data = -1;
        }
        let count = (aux & 0xff) as u32;

        let nbt_len = stream.get_lshort().unwrap_or(0);
        let nbt = if nbt_len > 0 {
            stream.get(nbt_len as usize).unwrap_or_default()
        } else {
            Vec::new()
        };

        let can_place_on = stream.get_var_int().unwrap_or(0);
        for _ in 0..can_place_on {
            let _ = stream.get_string();
        }
        let can_destroy = stream.get_var_int().unwrap_or(0);
        for _ in 0..can_destroy {
            let _ = stream.get_string();
        }

        *self = Self { id, meta: data, count, nbt };
    }

    pub fn put(&self, stream: &mut BinaryStream) {
        if self.is_air() {
            stream.put_var_int(0);
            return;
        }
        stream.put_var_int(self.id);
        let aux = (((self.meta & 0x7fff) as u32) << 8) | (self.count & 0xff);
        stream.put_var_int(aux as i32);
        stream.put_lshort(self.nbt.len() as u16);
        stream.put(&self.nbt);
        stream.put_var_int(0);
        stream.put_var_int(0);
    }
}
