use crate::{Decode, Encode};

/// 编码方案
pub struct Table<'a> {
    /// 每个字符对应的位数
    pub bits: u32,
    /// 结尾字符集
    pub fini: &'a [char],
    /// 正文字符集
    pub init: &'a [char],
    decode: [&'a [(u16, u16)]; 17],
}

impl<'a> Table<'a> {
    pub(crate) fn decode(&self, ch: char) -> Option<(u32, bool)> {
        let plane = self.decode[ch as usize >> 16];
        let idx = plane.binary_search_by_key(&(ch as u16), |&x| x.0);
        let code = plane[idx.ok()?].1 as u32;
        Some((code & ((1 << 15) - 1), code >> 15 != 0))
    }

    /// 编码为迭代器
    pub fn encode_iter<I>(&'a self, iter: I) -> Encode<'a, I::IntoIter>
    where
        I: IntoIterator<Item = u8>,
    {
        Encode::new(self, iter.into_iter())
    }

    /// 解码为迭代器
    ///
    /// 若输入有错误字符，则输出该字符的错误。
    pub fn decode_iter<I>(&'a self, iter: I) -> Decode<'a, I::IntoIter>
    where
        I: IntoIterator<Item = char>,
    {
        Decode::new(self, iter.into_iter())
    }
}

#[cfg(any(feature = "std", test))]
impl Table<'_> {
    /// 编码为字符串
    pub fn encode_str<I>(&self, iter: I) -> String
    where
        I: IntoIterator<Item = u8>,
    {
        self.encode_iter(iter).collect()
    }

    /// 解码为数组
    ///
    /// 若输入有错误字符，则返回该字符的错误。
    pub fn decode_vec<I>(&self, iter: I) -> Result<Vec<u8>, char>
    where
        I: IntoIterator<Item = char>,
    {
        self.decode_iter(iter).collect()
    }
}

#[cfg(test)]
impl Table<'_> {
    pub(crate) fn with<T>(bits: u32, f: impl FnOnce(&Table) -> T) -> T {
        let fin_len = if bits > 8 { 1 << (bits - 8) } else { 0 };
        let chs: Vec<_> = ('\0'..).take(fin_len + (1 << bits)).collect();
        let mut decode = std::iter::zip(
            chs.iter().map(|&c| c as u16),
            (0..fin_len as u16).map(|x| x | (1 << 15)).chain(0..),
        );
        let decode = std::array::from_fn(|_| decode.by_ref().take(65536).collect());
        f(&Table {
            bits,
            fini: &chs[..fin_len],
            init: &chs[fin_len..],
            decode: decode.each_ref().map(|a: &Vec<_>| &**a),
        })
    }
}

/// 自动生成的编码方案
#[allow(warnings)]
pub mod tables {
    use super::*;
    include!(concat!(env!("OUT_DIR"), "/table.rs"));
}
