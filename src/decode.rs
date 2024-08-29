use crate::table::Table;
use core::iter::FusedIterator;

/// 参阅 [`Table::decode_iter`]
#[derive(Clone)]
pub struct Decode<'a, I> {
    buf: u32,
    len: u32,
    table: &'a Table<'a>,
    inner: I,
}

impl<'a, I> Decode<'a, I>
where
    I: Iterator<Item = char>,
{
    pub(crate) fn new(table: &'a Table, iter: I) -> Self {
        Self {
            buf: 0,
            len: 0,
            table,
            inner: iter,
        }
    }
}

impl<I> Iterator for Decode<'_, I>
where
    I: Iterator<Item = char>,
{
    type Item = Result<u8, char>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.len < 8 {
            let Some(ch) = self.inner.next() else {
                self.len = 0;
                return None;
            };
            let Some((data, finish)) = self.table.decode(ch) else {
                return Some(Err(ch));
            };
            let bits = self.table.bits;
            let len = if finish { bits.saturating_sub(8) } else { bits };
            self.len += len;
            self.buf <<= len;
            self.buf |= data;
        }
        self.len -= 8;
        Some(Ok((self.buf >> self.len) as u8))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let map = |x: usize, diff| {
            let remains = (x as u128) * (self.table.bits as u128);
            ((remains.saturating_sub(diff) + (self.len as u128)) / 8).try_into()
        };
        let diff = self.table.bits / 8 * 8;
        let (lo, hi) = self.inner.size_hint();
        (
            map(lo, diff as u128).unwrap_or(usize::MAX),
            hi.and_then(|hi| map(hi, 0).ok()),
        )
    }
}

impl<I> FusedIterator for Decode<'_, I> where I: Iterator<Item = char> + FusedIterator {}
