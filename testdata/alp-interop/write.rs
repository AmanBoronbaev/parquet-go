use parquet::basic::{Compression, Encoding};
use parquet::data_type::{DoubleType, FloatType};
use parquet::file::{properties::WriterProperties, writer::SerializedFileWriter};
use parquet::schema::{parser::parse_message_type, types::ColumnPath};
use std::{error::Error, fs::File, sync::Arc};

fn main() -> Result<(), Box<dyn Error>> {
    let schema = Arc::new(parse_message_type(
        "message samples { REQUIRED FLOAT f_plain; REQUIRED FLOAT f_alp; REQUIRED DOUBLE d_plain; REQUIRED DOUBLE d_alp; }",
    )?);
    for pattern in ["hundredths", "raw-bits", "exceptions"] {
        let mut seed = 0x243f6a8885a308d3u64;
        let mut floats = Vec::new();
        let mut doubles = Vec::new();
        for i in 0..65_537 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let cents = ((seed % 100_001) as i64 - 50_000) as f64 / 100.;
            let (f, d) = match pattern {
                "raw-bits" => (f32::from_bits(seed as u32), f64::from_bits(seed)),
                "exceptions" if i % 31 == 0 => match (i / 31) % 4 {
                    0 => (-0.0, -0.0),
                    1 => (
                        f32::from_bits(0x7fc00042),
                        f64::from_bits(0x7ff8000000000042),
                    ),
                    2 => (f32::INFINITY, f64::INFINITY),
                    _ => (f32::from_bits(1), f64::from_bits(1)),
                },
                _ => (cents as f32, cents),
            };
            floats.push(f);
            doubles.push(d);
        }
        let props = WriterProperties::builder()
            .set_dictionary_enabled(false)
            .set_compression(Compression::ZSTD(Default::default()))
            .set_encoding(Encoding::PLAIN)
            .set_column_encoding(ColumnPath::from("f_alp"), Encoding::ALP)
            .set_column_encoding(ColumnPath::from("d_alp"), Encoding::ALP)
            .build();
        let mut file = SerializedFileWriter::new(
            File::create(format!("{pattern}.parquet"))?,
            schema.clone(),
            Arc::new(props),
        )?;
        let mut group = file.next_row_group()?;
        for idx in 0..4 {
            let mut col = group.next_column()?.unwrap();
            if idx < 2 {
                col.typed::<FloatType>().write_batch(&floats, None, None)?;
            } else {
                col.typed::<DoubleType>()
                    .write_batch(&doubles, None, None)?;
            }
            col.close()?;
        }
        group.close()?;
        file.close()?;
    }
    Ok(())
}
