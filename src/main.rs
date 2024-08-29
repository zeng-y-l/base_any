use base_any::tables::ALL;
use std::io::{self, Read, Write};

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let mode = args.next().expect("too few arguments");

    if mode == "stat" {
        stat();
        return;
    }

    let table = args.next().expect("too few arguments");
    assert!(args.next().is_none(), "too many arguments");

    let table = ALL
        .iter()
        .find(|(n, _)| *n == table)
        .expect("unknown table")
        .1;

    match &*mode {
        "en" => {
            let mut input = Vec::new();
            io::stdin().read_to_end(&mut input).unwrap();
            println!("{}", table.encode_str(input));
        }
        "de" => {
            let input = io::read_to_string(io::stdin()).unwrap();
            let decode = table.decode_vec(input.chars()).expect("unknown character");
            io::stdout().write_all(&decode).unwrap()
        }
        _ => panic!("unknown mode"),
    };
}

fn stat() {
    println!(
        "|{:^12}|{:^6}|{:^9}|{:^12}|{:^13}|",
        "Name", "Bits", "Range", "UTF-8 Effi", "UTF-16 Effi"
    );
    println!(
        "|:{s:-^10}:|:{s:-^4}:|:{s:-^7}:|:{s:-^10}:|:{s:-^11}:|",
        s = ""
    );
    for (name, table) in ALL {
        let range = match table.init.iter().chain(table.fini).max().unwrap() {
            ..='\x7F' => "ASCII",
            ..'\u{10000}' => "BMP",
            _ => "Unicode",
        };
        let bits = table.bits;
        let effi = |s: usize| bits as f64 / 8.0 * table.init.len() as f64 / s as f64 * 100.0;
        let sum8 = table.init.iter().map(|c| c.len_utf8()).sum();
        let sum16 = table.init.iter().map(|c| c.len_utf16() * 2).sum();

        println!(
            "|{name:^12}|{bits:^6}|{range:^9}|{eff8:^12}|{eff16:^13}|",
            eff8 = format!("{:.2}%", effi(sum8)),
            eff16 = format!("{:.2}%", effi(sum16)),
        );
    }
}
