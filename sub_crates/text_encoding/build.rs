use std::env;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Generate all of the single byte encoding tables and wrapper code.
    {
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-ibm866.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("ibm866.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-2.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-2.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-3.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-3.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-4.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-4.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-5.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-5.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-6.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-6.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-7.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-7.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-8.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-8.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-10.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-10.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-13.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-13.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-14.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-14.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-15.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-15.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-iso-8859-16.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("iso-8859-16.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-koi8-r.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("koi8-r.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-koi8-u.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("koi8-u.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-macintosh.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("macintosh.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-874.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-874.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1250.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1250.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1251.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1251.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1252.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1252.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1253.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1253.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1254.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1254.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1255.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1255.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1256.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1256.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1257.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1257.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-windows-1258.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("windows-1258.rs")).unwrap(),
        ).unwrap();
        generate_single_byte_encoding_from_index(
            File::open("encoding_tables/index-x-mac-cyrillic.txt").unwrap(),
            File::create(&Path::new(&out_dir).join("x-mac-cyrillic.rs")).unwrap(),
        ).unwrap();
    }
}

fn generate_single_byte_encoding_from_index<R: Read, W: Write>(
    in_file: R,
    mut out_file: W,
) -> std::io::Result<()> {
    let in_file = std::io::BufReader::new(in_file);

    // Collect the table.
    let table = {
        let mut table = ['�'; 128];
        for line in in_file.lines() {
            let tmp = line.unwrap();
            let line = tmp.trim();
            if line.starts_with("#") || line == "" {
                continue;
            }

            let elements: Vec<_> = line.split_whitespace().collect();
            if elements.len() >= 2 {
                let index = elements[0].parse::<usize>().unwrap();
                assert!(index <= 127);
                let code = std::char::from_u32(u32::from_str_radix(&elements[1][2..], 16).unwrap())
                    .unwrap();
                table[index] = code;
            }
        }
        table
    };

    // Build the reverse table.
    let rev_table = {
        let mut rev_table = vec![];
        for (i, c) in table.iter().enumerate() {
            rev_table.push((c, 128 + i));
        }
        rev_table.sort_by_key(|x| x.0);
        rev_table
    };

    // Write shared code.
    out_file.write_all(
        format!(
            r#"
use {{DecodeResult, EncodeResult}};

pub fn encode_from_str<'a>(input: &str, output: &'a mut [u8]) -> EncodeResult<'a> {{
    super::single_byte_encode_from_str(&ENCODE_TABLE, input, output)
}}

pub fn decode_to_str<'a>(input: &[u8], output: &'a mut [u8]) -> DecodeResult<'a> {{
    super::single_byte_decode_to_str(&DECODE_TABLE, input, output)
}}
"#
        ).as_bytes(),
    )?;

    // Write encode table.
    out_file.write_all(
        format!(
            r#"
const ENCODE_TABLE: [(char, u8); {}] = [
"#,
            rev_table.len()
        ).as_bytes(),
    )?;

    for (c, i) in rev_table.iter() {
        out_file.write_all(format!("('\\u{{{:04X}}}', 0x{:02X}), ", **c as u32, i).as_bytes())?;
    }

    out_file.write_all(
        format!(
            r#"
];
"#
        ).as_bytes(),
    )?;

    // Write decode table.
    out_file.write_all(
        format!(
            r#"
const DECODE_TABLE: [char; 128] = [
"#
        ).as_bytes(),
    )?;

    for c in table.iter() {
        out_file.write_all(format!("'\\u{{{:04X}}}', ", *c as u32).as_bytes())?;
    }

    out_file.write_all(
        format!(
            r#"
];
"#
        ).as_bytes(),
    )?;

    Ok(())
}
