use std::{env, fmt::Write, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=table/");

    let mut out = String::new();
    let mut all = Vec::new();

    for entry in fs::read_dir("table").unwrap() {
        let path = entry.unwrap().path();
        let file = fs::read_to_string(&path).unwrap();
        let mut lines = file.lines().filter(|l| !l.starts_with("-- "));

        for cmt in lines.by_ref().take_while(|&l| l != "----") {
            writeln!(out, "/// {cmt}").unwrap();
        }

        let name = path.file_stem().unwrap().to_str().unwrap();
        all.push(name.to_owned());
        write!(out, "pub const {name}: &Table = &Table {{ ").unwrap();

        let size: usize = lines.next().unwrap().parse().unwrap();
        assert!(size.is_power_of_two());
        let bits = size.ilog2();
        assert!(0 < bits && bits < 16);
        write!(out, "bits: {bits}, ").unwrap();
        let fin_size = if bits > 8 { 1 << (bits - 8) } else { 0 };
        assert_eq!(lines.next().unwrap(), "----");

        let mut data = Vec::new();
        data.extend(
            lines
                .by_ref()
                .take_while(|&l| l != "----")
                .flat_map(str::chars)
                .enumerate()
                .map(|(i, c)| (c, false, i)),
        );
        data.extend(
            lines
                .flat_map(str::chars)
                .enumerate()
                .map(|(i, c)| (c, true, i)),
        );

        data.sort_unstable();
        data.dedup_by_key(|x| x.0);
        assert_eq!(size + fin_size, data.len());

        data.sort_unstable_by_key(|x| (!x.1, x.2));
        let mid = data.partition_point(|x| x.1);
        let (fini, init) = data.split_at_mut(mid);
        assert_eq!(fin_size, fini.len());
        assert_eq!(size, init.len());

        for (i, c) in fini.iter_mut().enumerate() {
            c.2 = i;
        }
        for (i, c) in init.iter_mut().enumerate() {
            c.2 = i;
        }

        write!(out, "fini: &[").unwrap();
        for x in fini {
            write!(out, "'{}',", x.0.escape_unicode()).unwrap();
        }
        write!(out, "], ").unwrap();

        write!(out, "init: &[").unwrap();
        for x in init {
            write!(out, "'{}',", x.0.escape_unicode()).unwrap();
        }
        write!(out, "], ").unwrap();

        data.sort_unstable();
        write!(out, "decode: [").unwrap();
        (0..17).fold(&*data, |remains, plane| {
            let end = remains.partition_point(|x| (x.0 as u32) >> 16 == plane);
            let (plane, remains) = remains.split_at(end);
            write!(out, "&[").unwrap();
            for &(ch, fin, id) in plane {
                let ch = ch as u16;
                let code = (id as u32) | ((fin as u32) << 15);
                write!(out, "({ch},{code}),").unwrap();
            }
            write!(out, "],").unwrap();
            remains
        });
        write!(out, "], ").unwrap();

        writeln!(out, "}};").unwrap();
    }

    write!(out, "pub const ALL: &[(&str, &Table)] = &[").unwrap();
    for name in &all {
        write!(out, "({name:?}, {name}), ").unwrap();
    }
    writeln!(out, "];").unwrap();

    let mut out_file = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_file.push("table.rs");
    fs::write(out_file, out).unwrap();
}
