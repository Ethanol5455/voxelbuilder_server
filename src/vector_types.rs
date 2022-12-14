use byteorder::ByteOrder;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 {
            x,
            y,
        }
    }
}

impl Vec2<f32> {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let mut vec = Vec::<u8>::new();

        let mut buf: [u8; 4] = [0; 4];

        // x
        byteorder::LittleEndian::write_f32(&mut buf, self.x);
        vec.append(&mut buf.to_vec());

        // y
        byteorder::LittleEndian::write_f32(&mut buf, self.y);
        vec.append(&mut buf.to_vec());

        vec
    }

    pub fn from_u8_arr(data: &[u8]) -> Vec2<f32> {
        let x = byteorder::LittleEndian::read_f32(data);
        let y = byteorder::LittleEndian::read_f32(&data[4..8]);
        Vec2::new(x, y)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Vec3<T> {
        Vec3 {
            x,
            y,
            z,
        }
    }
}

impl Vec3<f32> {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let mut vec = Vec::<u8>::new();

        let mut buf: [u8; 4] = [0; 4];
        
        // x
        byteorder::LittleEndian::write_f32(&mut buf, self.x);
        vec.append(&mut buf.to_vec());

        // y
        byteorder::LittleEndian::write_f32(&mut buf, self.y);
        vec.append(&mut buf.to_vec());

        // z
        byteorder::LittleEndian::write_f32(&mut buf, self.z);
        vec.append(&mut buf.to_vec());

        vec
    }

    pub fn from_u8_arr(data: &[u8]) -> Vec3<f32> {
        let x = byteorder::LittleEndian::read_f32(data);
        let y = byteorder::LittleEndian::read_f32(&data[4..8]);
        let z = byteorder::LittleEndian::read_f32(&data[8..12]);
        Vec3::new(x, y, z)
    }
}

impl Vec3<i32> {
    pub fn to_u8_vec(&self) -> Vec<u8> {
        let mut vec = Vec::<u8>::new();

        let mut buf: [u8; 4] = [0; 4];
        
        // x
        byteorder::LittleEndian::write_i32(&mut buf, self.x);
        vec.append(&mut buf.to_vec());

        // y
        byteorder::LittleEndian::write_i32(&mut buf, self.y);
        vec.append(&mut buf.to_vec());

        // z
        byteorder::LittleEndian::write_i32(&mut buf, self.z);
        vec.append(&mut buf.to_vec());

        vec
    }

    pub fn from_u8_arr(data: &[u8]) -> Vec3<i32> {
        let x = byteorder::LittleEndian::read_i32(data);
        let y = byteorder::LittleEndian::read_i32(&data[4..8]);
        let z = byteorder::LittleEndian::read_i32(&data[8..12]);
        Vec3::new(x, y, z)
    }
}