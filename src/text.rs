use crate::util::*;
use btree::*;

pub struct File {
    data: BTree<TextBuffer>,
}

impl File {
    pub fn new() -> Self {
        let mut data = BTree::new();
        data.add(TextBuffer::new());

        return Self { data };
    }

    pub fn push(&mut self, text: &str) {
        let last = unwrap(self.data.last_idx());
        let offset = unwrap(self.data.get(last)).get_info().content_size;

        self.insert_at(last, offset, text);
    }

    pub fn insert(&mut self, idx: usize, text: &str) {
        let (idx, offset) = unwrap(self.data.key_leq_idx(idx, BufferInfo::content));

        self.insert_at(idx, offset, text);
    }

    pub fn delete(&mut self, begin: usize, end: usize) {
        if begin >= end {
            return;
        }

        let mut len = end - begin;
        while len > 0 {
            let (idx, offset) = unwrap(self.data.key_idx(begin, BufferInfo::content));
            self.data.edit_or_remove(idx, |buf| {
                let char_count = buf.char_count as usize;
                if char_count > len {
                    let mut count = 0;
                    let mut byte_idx = None;
                    for (i, c) in buf.buffer.char_indices() {
                        if offset == count {
                            byte_idx = Some(i);
                            break;
                        }

                        count += 1;
                    }

                    let idx = match byte_idx {
                        Some(i) => i,
                        None => core::panic!("should this ever happen?"),
                    };

                    for _ in 0..len {
                        buf.buffer.remove(idx);
                    }

                    buf.char_count -= len as u16;

                    len = 0;
                    return false;
                }

                len -= char_count;
                return true;
            });
        }

        if self.data.len() == 0 {
            self.data.add(TextBuffer::new());
        }
    }

    pub fn last_line_begin(&self) -> usize {
        let lines = self.newlines();
        if lines == 0 {
            return 0;
        }

        return unwrap(self.cursor_for_line(lines - 1));
    }

    fn insert_at(&mut self, idx: ElemIdx, mut offset: usize, text: &str) {
        let result = self.data.get_mut(idx, |buf| {
            let above = buf.split_at(offset);
            let mut iter = text.chars();

            if buf.is_full() {
                return (iter, above);
            }

            for c in &mut iter {
                let is_full = buf.insert(offset, c);
                offset += 1;

                if is_full {
                    break;
                }
            }

            return (iter, above);
        });

        let (mut remaining_chars, buf) = unwrap(result);
        let mut buf = match buf.is_empty() {
            true => buf,
            false => {
                self.data.insert_after(idx, buf);
                TextBuffer::new()
            }
        };

        let mut idx = idx;
        for c in &mut remaining_chars {
            if buf.push(c) {
                idx = self.data.insert_after(idx, buf);
                buf = TextBuffer::new();
            }
        }

        if !buf.is_empty() {
            self.data.insert_after(idx, buf);
        }
    }

    pub fn newlines(&self) -> usize {
        return self.data.info().newline_count;
    }

    pub fn len(&self) -> usize {
        return self.data.info().content_size;
    }

    pub fn line_for_cursor(&self, idx: usize) -> Option<usize> {
        let (idx, remainder) = self.data.key_leq_idx(idx, BufferInfo::content)?;
        let lines_before = self.data.sum_until(idx, |_, info| info.newline_count)?;
        let bytes = self.data[idx].buffer.as_bytes().iter();
        let lines = lines_before + bytes.take(remainder).filter(|b| **b != b'\n').count();

        return Some(lines);
    }

    pub fn cursor_for_line(&self, line: usize) -> Option<usize> {
        let (idx, remainder) = self.data.key_leq_idx(line, BufferInfo::newlines)?;
        let cursor = self.data.sum_until(idx, |_, info| info.content_size)?;

        return Some(cursor + remainder);
    }

    pub fn end_cursor_for_line(&self, line: usize) -> usize {
        return match self.cursor_for_line(line + 1) {
            Some(cursor) => cursor,
            None => self.len(),
        };
    }

    pub fn text_for_line<'a>(&'a self, line: usize) -> Option<LineIter<'a>> {
        let (idx, remainder) = self.data.key_leq_idx(line, BufferInfo::newlines)?;
        let idx = self.data.count_until(idx)?;

        return Some(LineIter {
            file: self,
            idx,
            buffer_idx: remainder,
        });
    }

    pub fn text_after_cursor<'a>(&'a self, cursor: usize) -> Option<TextIter<'a>> {
        if cursor == 0 {
            return Some(TextIter {
                file: self,
                idx: 0,
                buffer_idx: 0,
            });
        }

        let (idx, remainder) = self.data.key_idx(cursor, BufferInfo::content)?;
        let idx = self.data.count_until(idx)?;

        return Some(TextIter {
            file: self,
            idx,
            buffer_idx: remainder,
        });
    }
}

impl<'a> IntoIterator for &'a File {
    type Item = &'a str;
    type IntoIter = FileIter<'a>;

    fn into_iter(self) -> FileIter<'a> {
        return FileIter { file: self, idx: 0 };
    }
}

pub struct TextIter<'a> {
    file: &'a File,
    idx: usize,
    buffer_idx: usize,
}

impl<'a> Iterator for TextIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.file.data.get(self.idx)?;

        self.idx += 1;
        let bytes = &result.buffer.as_bytes()[self.buffer_idx..];
        self.buffer_idx = 0;

        return Some(unsafe { core::str::from_utf8_unchecked(bytes) });
    }
}

pub struct LineIter<'a> {
    file: &'a File,
    idx: usize,
    buffer_idx: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.file.data.get(self.idx)?;

        if result.newline_count == 0 {
            self.idx += 1;
            let bytes = &result.buffer.as_bytes()[self.buffer_idx..];
            self.buffer_idx = 0;

            return Some(unsafe { core::str::from_utf8_unchecked(bytes) });
        }

        self.idx = usize::MAX;
        let newline = result.buffer.as_bytes().iter().position(|&c| c == b'\n')?;
        let bytes = &result.buffer.as_bytes()[self.buffer_idx..newline];
        self.buffer_idx = 0;

        return Some(unsafe { core::str::from_utf8_unchecked(bytes) });
    }
}

pub struct FileIter<'a> {
    file: &'a File,
    idx: usize,
}

impl<'a> Iterator for FileIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.file.data.get(self.idx);
        self.idx += 1;
        return result.map(|buf_view| &*buf_view.buffer);
    }
}

struct TextBuffer {
    buffer: String,
    char_count: u16,
    newline_count: u16,
}

impl TextBuffer {
    #[cfg(debug_assertions)]
    const MAX_LEN: usize = 64;

    #[cfg(not(debug_assertions))]
    const MAX_LEN: usize = 1024;

    pub fn new() -> Self {
        return Self {
            buffer: String::new(),
            char_count: 0,
            newline_count: 0,
        };
    }

    pub fn is_empty(&self) -> bool {
        return self.char_count == 0;
    }

    pub fn split_at(&mut self, offset: usize) -> Self {
        let buffer = self.buffer.clone();

        let mut other = Self::new();
        self.buffer.clear();
        self.char_count = 0;
        self.newline_count = 0;

        let mut idx = 0;
        let mut iter = buffer.chars();
        while idx < offset {
            let c = unwrap(iter.next());
            self.push(c);
            idx += 1;
        }

        while let Some(c) = iter.next() {
            other.push(c);
        }

        return other;
    }

    pub fn is_full(&self) -> bool {
        return self.buffer.len() >= TextBuffer::MAX_LEN - 4;
    }

    pub fn insert(&mut self, idx: usize, c: char) -> bool {
        if self.buffer.len() == 0 {
            self.buffer.reserve_exact(TextBuffer::MAX_LEN);
        }

        let mut count = 0;
        let mut byte_idx = None;
        for (i, c) in self.buffer.char_indices() {
            if idx == count {
                byte_idx = Some(i);
                break;
            }

            count += 1;
        }

        // TODO this is sad. I guess all the methods are for byte-positions?
        // Which, great I guess. Super glad about that. Thanks Rust.
        //                                  - Albert Liu, Dec 20, 2021 Mon 03:39 EST
        match byte_idx {
            Some(i) => self.buffer.insert(i, c),
            None => {
                if idx > count {
                    core::panic!(
                        "TextBuffer index was {} for count = {} (this is an editor error)",
                        idx,
                        count
                    );
                }

                self.buffer.push(c)
            }
        }

        self.char_count += 1;
        if c == '\n' {
            self.newline_count += 1;
        }

        return self.is_full();
    }

    pub fn push(&mut self, c: char) -> bool {
        if self.buffer.len() == 0 {
            self.buffer.reserve_exact(TextBuffer::MAX_LEN);
        }

        self.buffer.push(c);
        self.char_count += 1;
        if c == '\n' {
            self.newline_count += 1;
        }

        return self.is_full();
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

impl BTreeItem for TextBuffer {
    type Info = BufferInfo;

    fn get_info(&self) -> BufferInfo {
        return BufferInfo {
            content_size: self.char_count as usize,
            newline_count: self.newline_count as usize,
        };
    }
}
