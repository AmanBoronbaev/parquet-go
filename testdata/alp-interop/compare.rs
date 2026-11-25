use parquet::{
    basic::Encoding,
    file::reader::{FileReader, SerializedFileReader},
    record::RowAccessor,
};
use std::{env, error::Error, fs::File};
fn main() -> Result<(), Box<dyn Error>> {
    for path in env::args().skip(1) {
        let src = SerializedFileReader::new(File::open(&path)?)?;
        let out = SerializedFileReader::new(File::open(format!(
            "go-{}",
            std::path::Path::new(&path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        ))?)?;
        assert_eq!(
            src.metadata().file_metadata().num_rows(),
            out.metadata().file_metadata().num_rows()
        );
        for rg in out.metadata().row_groups() {
            for col in rg.columns() {
                assert!(col.encodings().any(|e| e == Encoding::ALP));
            }
        }
        let mut count = 0;
        for pair in src.get_row_iter(None)?.zip(out.get_row_iter(None)?) {
            let (a, b) = (pair.0?, pair.1?);
            assert_eq!(
                a.get_float(0)?.to_bits(),
                b.get_float(0)?.to_bits(),
                "FLOAT row {count}"
            );
            assert_eq!(
                a.get_double(2)?.to_bits(),
                b.get_double(1)?.to_bits(),
                "DOUBLE row {count}"
            );
            count += 1;
        }
        assert_eq!(count, 65537);
        println!("{path}: {} bitwise comparisons", count * 2);
    }
    Ok(())
}
