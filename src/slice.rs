use std::{cmp, ops, ptr, u8};
use bytes::{BufMut, ByteOrder, Bytes, LittleEndian};

const U64_BYTE_SIZE: usize = 8;
const U32_BYTE_SIZE: usize = 4;
const U16_BYTE_SIZE: usize = 2;
const U8_BYTE_SIZE: usize = 1;

pub fn short_successor(v: &mut Slice) {
    let l = v.len();

    for i in 0..l {
        if let Some(v) = v.inner.get_mut(i) {
            if v != &u8::MAX {
                *v += 1;
                return;
            }
        }
    }
}

pub fn shortest_separator(key: &mut Slice, limit: &Slice) {
    let min_size = cmp::min(key.len(), limit.len());

    for i in 0..min_size {
        if key.inner[i] != limit.inner[i] {
            if let Some(val) = key.inner.get_mut(i) {
                if val != &u8::MAX {
                    *val += 1;
                    return;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Slice {
    inner: Vec<u8>,
}

impl Slice {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }

    pub fn from(inner: &[u8]) -> Self {
        Self {
            inner: Vec::from(inner),
        }
    }

    // TODO: delete
    pub fn from_bytes(bytes: &Bytes) -> Self {
        Self {
            inner: Vec::from(bytes.as_ref()),
        }
    }

    // TODO: delete
    pub fn to_bytes(self) -> Bytes {
        Bytes::from(self.inner)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn put_u8(&mut self, n: u8) -> usize {
        self.inner.put_u8(n);
        1
    }

    pub fn put_u16(&mut self, n: u16) -> usize {
        self.inner.put_u16::<LittleEndian>(n);
        U16_BYTE_SIZE
    }

    pub fn put_u32(&mut self, n: u32) -> usize {
        self.inner.put_u32::<LittleEndian>(n);
        U32_BYTE_SIZE
    }

    pub fn put_u64(&mut self, n: u64) -> usize {
        self.inner.put_u64::<LittleEndian>(n);
        U64_BYTE_SIZE
    }

    pub fn put_i64(&mut self, n: i64) -> usize {
        self.inner.put_i64::<LittleEndian>(n);
        U64_BYTE_SIZE + 1 // XXX
    }

    pub fn put_str(&mut self, n: &str) -> usize {
        let s = n.len();
        self.inner.put_slice(n.as_bytes());
        s
    }

    pub fn get_u8(&self, offset: usize) -> Option<u8> {
        self.inner.get(offset).map(|v| *v)
    }

    pub fn get_u16(&self, offset: usize) -> Option<u16> {
        let lim = offset + U16_BYTE_SIZE;
        if self.inner.len() > lim {
            let mut buf = [0; U16_BYTE_SIZE];
            buf.copy_from_slice(&self.inner[offset..lim]);
            Some(LittleEndian::read_u16(&buf))
        } else {
            None
        }
    }

    pub fn get_u32(&self, offset: usize) -> Option<u32> {
        let lim = offset + U32_BYTE_SIZE;
        if self.inner.len() > lim {
            let mut buf = [0; U32_BYTE_SIZE];
            buf.copy_from_slice(&self.inner[offset..lim]);
            Some(LittleEndian::read_u32(&buf))
        } else {
            None
        }
    }

    pub fn get_u64(&self, offset: usize) -> Option<u64> {
        let lim = offset + U64_BYTE_SIZE;
        if self.inner.len() > lim {
            let mut buf = [0; U64_BYTE_SIZE];
            buf.copy_from_slice(&self.inner[offset..lim]);
            Some(LittleEndian::read_u64(&buf))
        } else {
            None
        }
    }

    pub fn resize(&mut self, size: usize) {
        self.inner.resize(size, 0)
    }

    pub fn put(&mut self, n: &Self) -> usize {
        let s = n.len();
        self.inner.put(n.inner.clone());
        s
    }

    pub fn put_slice(&mut self, n: &[u8]) -> usize {
        let s = n.len();
        self.inner.put(n);
        s
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.inner.len() >= U8_BYTE_SIZE {
            let buf = self.split_off(U8_BYTE_SIZE);
            Some(buf[0])
        } else {
            None
        }
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        if self.inner.len() >= U16_BYTE_SIZE {
            let buf = self.split_off(U16_BYTE_SIZE);
            Some(LittleEndian::read_u16(&buf))
        } else {
            None
        }
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        let s = self.inner.len();
        if s >= U32_BYTE_SIZE {
            let buf = self.split_off(U32_BYTE_SIZE);
            Some(LittleEndian::read_u32(&buf))
        } else {
            None
        }
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        let s = self.inner.len();
        if s >= U64_BYTE_SIZE {
            let buf = self.split_off(U64_BYTE_SIZE);
            Some(LittleEndian::read_u64(&buf))
        } else {
            None
        }
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        let s = self.inner.len();
        if s >= U64_BYTE_SIZE {
            let buf = self.split_off(U64_BYTE_SIZE);
            Some(LittleEndian::read_i64(&buf))
        } else {
            None
        }
    }

    pub fn read(&mut self, i: usize) -> Option<Vec<u8>> {
        let s = self.inner.len();
        if s >= i {
            Some(self.split_off(i))
        } else {
            None
        }
    }

    pub fn split_off(&mut self, at: usize) -> Vec<u8> {
        assert!(at <= self.inner.len(), "`at` out of bounds");

        let other_len = self.inner.len() - at;
        let mut other = Vec::with_capacity(at);

        unsafe {
            other.set_len(at);

            let ptr = self.inner.as_ptr();
            ptr::copy_nonoverlapping(ptr, other.as_mut_ptr(), other.len());
            ptr::copy(ptr.offset(at as isize), self.inner.as_mut_ptr(), other_len);

            self.inner.set_len(other_len);
        }
        other
    }
}

impl AsRef<[u8]> for Slice {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl ops::Index<usize> for Slice {
    type Output = u8;

    fn index(&self, v: usize) -> &u8 {
        &self.inner[v]
    }
}

impl ops::Index<ops::Range<usize>> for Slice {
    type Output = [u8];

    fn index(&self, v: ops::Range<usize>) -> &[u8] {
        &self.inner[v]
    }
}

impl PartialEq<Slice> for Slice {
    fn eq(&self, other: &Slice) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd<Slice> for Slice {
    fn partial_cmp(&self, other: &Slice) -> Option<cmp::Ordering> {
        (self.inner).partial_cmp(&other.inner)
    }
}

impl PartialEq<[u8]> for Slice {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl PartialOrd<[u8]> for Slice {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        (self.as_ref()).partial_cmp(other)
    }
}

impl PartialEq<Slice> for [u8] {
    fn eq(&self, other: &Slice) -> bool {
        other == self
    }
}

impl PartialOrd<Slice> for [u8] {
    fn partial_cmp(&self, other: &Slice) -> Option<cmp::Ordering> {
        other.partial_cmp(self)
    }
}

impl PartialEq<str> for Slice {
    fn eq(&self, other: &str) -> bool {
        self == other.as_bytes()
    }
}

impl PartialOrd<str> for Slice {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.partial_cmp(other.as_bytes())
    }
}

impl PartialEq<Slice> for str {
    fn eq(&self, other: &Slice) -> bool {
        other == self
    }
}

impl PartialOrd<Slice> for str {
    fn partial_cmp(&self, other: &Slice) -> Option<cmp::Ordering> {
        other.partial_cmp(self)
    }
}

impl PartialEq<Vec<u8>> for Slice {
    fn eq(&self, other: &Vec<u8>) -> bool {
        &self.inner[..] == &other[..]
    }
}

impl PartialOrd<Vec<u8>> for Slice {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        (self.inner[..]).partial_cmp(&other[..])
    }
}

impl PartialEq<Slice> for Vec<u8> {
    fn eq(&self, other: &Slice) -> bool {
        other == self
    }
}

impl PartialOrd<Slice> for Vec<u8> {
    fn partial_cmp(&self, other: &Slice) -> Option<cmp::Ordering> {
        other.partial_cmp(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Slice;

    #[test]
    fn read_and_write_slice_test() {
        let mut slice = Slice::with_capacity(100);
        slice.put_u8(1);
        assert_eq!(slice.len(), 1);
        slice.put_u16(2);
        assert_eq!(slice.len(), 1 + 2);

        slice.put_u32(3);
        assert_eq!(slice.len(), 1 + 2 + 4);
        slice.put_u64(4);
        assert_eq!(slice.len(), 1 + 2 + 4 + 8);

        assert_eq!(slice.read_u8(), Some(1));
        assert_eq!(slice.len(), 2 + 4 + 8);
        assert_eq!(slice.read_u16(), Some(2));
        assert_eq!(slice.len(), 4 + 8);
        assert_eq!(slice.read_u32(), Some(3));
        assert_eq!(slice.len(), 8);
        assert_eq!(slice.read_u64(), Some(4));
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn slice_ord_test() {
        let slice = Slice::from(b"bbb");
        let slice2 = Slice::from(b"bba");
        let slice3 = Slice::from(b"baa");

        assert!(slice == slice);
        assert!(slice > slice2);
        assert!(slice3 < slice2);

        assert!(&slice > "bba");
        assert!(&slice > "baa");
        assert!(&slice > "baa");

        assert!(slice > b"bba".to_vec());
        assert!(slice > b"baa".to_vec());
        assert!(slice > b"baa".to_vec());
    }
}
