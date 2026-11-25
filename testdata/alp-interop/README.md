# ALP writer interoperability experiment

Apache Arrow Rust writes three Parquet files. parquet-go reads each file, checks every FLOAT and DOUBLE against a PLAIN reference column by its IEEE bits, and prints the sizes and encodings from the footer. The experiment tests how data distribution affects the case for an opt-in ALP writer in Go. The files in this experiment were written by Rust.

## Reproduce

From the parquet-go repository root, with Rust, Cargo and Go installed:

```sh
repo=$(pwd)
out=$(mktemp -d)
(cd "$out" && cargo run --release --manifest-path "$repo/testdata/alp-interop/Cargo.toml")
go run ./testdata/alp-interop/read.go "$out"/*.parquet
```

Each file has one row group of 65,537 rows and paired PLAIN/ALP columns for each physical type. Both columns receive the same values and writer settings, with Zstandard compression and dictionaries disabled. The fixed seed and population are in `write.rs`. The extra row exercises a partial final ALP vector. The Go check covers 393,222 bitwise comparisons across the three files.

## Results

Compressed column chunk sizes in bytes, including page headers, read from the file footer:

| Input distribution | FLOAT PLAIN | FLOAT ALP | DOUBLE PLAIN | DOUBLE ALP |
| --- | ---: | ---: | ---: | ---: |
| Random multiples of 0.01 between −500 and 500 | 222,839 | 194,916 | 224,657 | 140,528 |
| Same distribution, every 31st value replaced by an IEEE edge case | 222,109 | 203,209 | 226,752 | 146,331 |
| Pseudorandom IEEE bit patterns | 262,285 | 288,722 | 524,442 | 549,881 |

On the hundredths sample, the ALP DOUBLE chunk used 37.4% fewer bytes. On random bit patterns it used 4.9% more. The exceptional values included negative zero, a NaN with a chosen payload, infinity and the smallest positive subnormal. All three files passed the bitwise comparison. The measurements describe these synthetic inputs, the pinned Rust writer and its Zstandard configuration.

The [upstream parquet-go decoder](https://github.com/parquet-go/parquet-go/pull/701) also has a separate conformance test using Apache's 9,032-row reference file, including nulls and three ALP vector sizes. The encoding layout is specified in [apache/parquet-format](https://github.com/apache/parquet-format/blob/5cc18cae643835b19f869d3cffb085cd3c04eb15/AlpEncoding.md).
