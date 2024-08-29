use crate::table::Table;
use core::iter::FusedIterator;

/// 参阅 [`Table::encode_iter`]
#[derive(Clone)]
pub struct Encode<'a, I> {
    buf: u32,
    len: u32,
    table: &'a Table<'a>,
    inner: I,
    mask: u32,
}

impl<'a, I> Encode<'a, I>
where
    I: Iterator<Item = u8>,
{
    pub(crate) fn new(table: &'a Table, iter: I) -> Self {
        Self {
            buf: 0,
            len: 0,
            table,
            inner: iter,
            mask: (1 << table.bits) - 1,
        }
    }
}

impl<I> Iterator for Encode<'_, I>
where
    I: Iterator<Item = u8>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let bits = self.table.bits;
        while self.len < bits {
            if let Some(byte) = self.inner.next() {
                self.len += 8;
                self.buf <<= 8;
                self.buf |= byte as u32;
            } else if self.len > 0 {
                let fin_bits = bits.saturating_sub(8);
                let (mask, bits, table) = if self.len > fin_bits {
                    (self.mask, bits, self.table.init)
                } else {
                    let mask = (1 << fin_bits) - 1;
                    (mask, fin_bits, self.table.fini)
                };
                let code = (self.buf << bits >> self.len) & mask;
                self.len = 0;
                return Some(table[code as usize]);
            } else {
                return None;
            }
        }
        self.len -= bits;
        let code = (self.buf >> self.len) & self.mask;
        Some(self.table.init[code as usize])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let map = |x: usize| {
            ((x as u128) * 8 + (self.len as u128))
                .div_ceil(self.table.bits as u128)
                .try_into()
                .ok()
        };
        let (lo, hi) = self.inner.size_hint();
        (map(lo).unwrap_or(usize::MAX), hi.and_then(map))
    }
}

impl<I> FusedIterator for Encode<'_, I> where I: Iterator<Item = u8> + FusedIterator {}
