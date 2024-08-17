use std::io::Write;
use std::{error::Error, fs::File, io::BufWriter};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");

    let map = map_compiler::compile_map("map.tmx")?;

    let output_file = File::create(format!("{out_dir}/map.rs"))?;
    let mut writer = BufWriter::new(output_file);
    writeln!(writer, "{}", map)?;

    Ok(())
}
