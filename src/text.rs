use crate::btree::*;

pub struct File {
    file_cursor: usize,
    data: BTree<BufferView>,
}

impl File {
    pub fn new() -> Self {
        return Self {
            file_cursor: 0,
            data: BTree::new(),
        };
    }

    pub fn insert(&mut self, text: &str) {}
}

pub struct BufferView {
    buffer: Box<[u8; 4096]>,
    content_size: u16,
    newline_count: u16,
}

impl BufferView {
    pub fn new() -> Self {
        let mut vec = vec![0u8; 4096];
        let ptr = vec.as_mut_ptr() as *mut [u8; 4096];
        vec.leak();

        return Self {
            buffer: unsafe { Box::from_raw(ptr) },
            content_size: 0,
            newline_count: 0,
        };
    }
}

#[derive(Default, Clone, Copy)]
pub struct BufferInfo {
    content_size: usize,
    newline_count: usize,
}

impl BufferInfo {
    fn content(self: BufferInfo) -> usize {
        return self.content_size;
    }

    fn newlines(self: BufferInfo) -> usize {
        return self.newline_count;
    }
}

impl BTreeInfo for BufferInfo {
    fn add(self, other: Self) -> Self {
        return BufferInfo {
            content_size: self.content_size + other.content_size,
            newline_count: self.newline_count + other.newline_count,
        };
    }
}

impl BTreeItem for BufferView {
    type Info = BufferInfo;
    fn get_info(&self) -> BufferInfo {
        return BufferInfo {
            content_size: self.content_size as usize,
            newline_count: self.newline_count as usize,
        };
    }
}
