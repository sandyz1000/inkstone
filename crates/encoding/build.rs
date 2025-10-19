use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("agl-aglfn/glyphlist.txt");
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:rerun-if-changed={}",
        path.to_str().expect("no-utf8 path")
    );

    let mut glyph_list =
        File::create(PathBuf::from(env::var("OUT_DIR").unwrap()).join("glyphlist.rs")).unwrap();

    writeln!(glyph_list, "[").unwrap();
    for line in BufReader::new(File::open(path).unwrap()).lines() {
        let line = line.unwrap();
        if line.starts_with("#") {
            continue;
        }
        let mut parts = line.split(";");
        let name = parts.next().unwrap();
        let unicode: String = parts
            .next()
            .unwrap()
            .split(" ")
            .map(|s| u32::from_str_radix(s, 16).unwrap())
            .map(|cp| std::char::from_u32(cp).unwrap())
            .collect();

        writeln!(glyph_list, "    ({:?}, {:?}),", name, unicode).unwrap();
    }
    writeln!(glyph_list, "]").unwrap();
}
