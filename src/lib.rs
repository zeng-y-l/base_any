#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

mod decode;
mod encode;
mod table;

pub use decode::Decode;
pub use encode::Encode;
pub use table::*;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prelude::TestCaseError, prop_assert, prop_assert_eq, prop_assume, proptest};
    use std::{
        collections::HashSet,
        ops::{Bound, RangeBounds},
    };
    use tables::*;
    use unicode_ident::{is_xid_continue, is_xid_start};

    #[test]
    fn code() {
        for (_, table) in ALL {
            let init_max = 1 << table.bits;
            for code in 0..init_max {
                let ch = table.init[code];
                assert_eq!(table.decode(ch), Some((code as u32, false)));
            }
            let fin_max = if table.bits > 8 {
                1 << (table.bits - 8)
            } else {
                0
            };
            for code in 0..fin_max {
                let ch = table.fini[code];
                assert_eq!(table.decode(ch), Some((code as u32, true)));
            }

            proptest!(|(code in init_max..)| {
                prop_assert!(table.init.get(code).is_none());
            });
            proptest!(|(code in fin_max..)| {
                prop_assert!(table.fini.get(code).is_none());
            });

            let chs: HashSet<_> = table.init.iter().chain(table.fini).collect();
            proptest!(|(ch: char)| {
                prop_assume!(!chs.contains(&ch));
                prop_assert!(table.decode(ch).is_none());
            });
        }
    }

    #[test]
    fn base8192() {
        for c in BASE8192.init.iter().chain(BASE8192.fini) {
            assert!(c.is_alphanumeric());
            assert!(is_xid_continue(*c));
        }
    }

    #[test]
    fn base1024() {
        for c in BASE1024.init.iter().chain(BASE1024.fini) {
            assert!(matches!(c, '\u{4e00}'..='\u{9FFF}' | '\u{3100}'..='\u{312F}'),);
            assert!(is_xid_start(*c));
        }
    }

    #[test]
    fn bits() {
        for bits in 1..15 {
            Table::with(bits, |table| {
                proptest!(|(data: Vec<u8>)| {
                    check_all(table, &data)?;
                });
            });
        }
    }

    type Res = Result<(), TestCaseError>;

    fn check_size(mut iter: impl Iterator) -> Res {
        let mut size = Vec::new();
        while {
            let (lo, hi) = iter.size_hint();
            size.push((
                Bound::Included(lo),
                hi.map_or(Bound::Unbounded, Bound::Included),
            ));
            iter.next().is_some()
        } {}

        for (size, range) in size.iter().rev().enumerate() {
            prop_assert!(range.contains(&size));
        }
        Ok(())
    }

    fn check_fuse(mut iter: impl Iterator) -> Res {
        iter.by_ref().count();
        prop_assert!(iter.next().is_none());
        Ok(())
    }

    fn check_all(table: &Table, data: &[u8]) -> Res {
        let encode_iter = Encode::new(table, data.iter().copied());
        let decode_iter = Decode::new(table, encode_iter.clone());

        let data2: Result<Vec<_>, _> = decode_iter.clone().collect();
        prop_assert_eq!(Ok(data), data2.as_deref());

        check_size(encode_iter.clone())?;
        check_size(decode_iter.clone())?;

        check_fuse(encode_iter.clone())?;
        check_fuse(decode_iter.clone())?;

        Ok(())
    }

    proptest!(
        #[test]
        fn test(data: Vec<u8>) {
            for (_, table) in ALL {
                check_all(table, &data)?;
            }
        }

        #[test]
        fn decode_err(mut code: String) {
            code.push(char::MAX);
            for (_, table) in ALL {
                let iter = Decode::new(table, code.chars());
                prop_assert!(iter.clone().any(|r| r.is_err()));
                check_fuse(iter.clone())?;
            }
        }

        #[test]
        fn base64(data: Vec<u8>) {
            use base64::prelude::*;
            for (a, b) in [
                (BASE64, BASE64_STANDARD_NO_PAD),
                (BASE64URL, BASE64_URL_SAFE_NO_PAD),
            ] {
                prop_assert_eq!(a.encode_str(data.iter().copied()), b.encode(&data));
            }
        }

        #[test]
        fn base32(data: Vec<u8>) {
            use base32::Alphabet::*;
            for (a, b) in [
                (BASE32, Rfc4648 { padding: false }),
                (BASE32HEX, Rfc4648Hex { padding: false }),
            ] {
                prop_assert_eq!(a.encode_str(data.iter().copied()), base32::encode(b, &data));
            }
        }

        #[test]
        fn base16(data: Vec<u8>) {
            prop_assert_eq!(
                HEX.encode_str(data.iter().copied()),
                base16ct::HexDisplay(&data).to_string()
            );
        }
    );
}
