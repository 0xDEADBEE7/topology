//! Names impl methods after their implemented types.

trait Size {
    fn size(&self) -> usize;
}

impl Size for [u8; 4] {
    fn size(&self) -> usize { 4 }
}

impl<A, B> Size for (A, B) {
    fn size(&self) -> usize { 2 }
}

impl dyn Size + Send {
    fn twice(&self) -> usize { self.size() * 2 }
}

impl Size for *const u8 {
    fn size(&self) -> usize { 8 }
}

impl<'a> Size for &'a mut std::vec::Vec<u8> {
    fn size(&self) -> usize {

        self.len()
    }
}
