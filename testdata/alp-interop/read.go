package main

import (
	"fmt"
	"log"
	"math"
	"os"
	"path/filepath"
	"slices"
	"strings"

	"github.com/parquet-go/parquet-go"
	"github.com/parquet-go/parquet-go/format"
)

type row struct {
	FloatPlain  float32 `parquet:"f_plain"`
	FloatALP    float32 `parquet:"f_alp"`
	DoublePlain float64 `parquet:"d_plain"`
	DoubleALP   float64 `parquet:"d_alp"`
}

func check(path string) error {
	rows, err := parquet.ReadFile[row](path)
	if err != nil {
		return err
	}
	if len(rows) != 65537 {
		return fmt.Errorf("got %d rows", len(rows))
	}
	for i, r := range rows {
		if math.Float32bits(r.FloatPlain) != math.Float32bits(r.FloatALP) || math.Float64bits(r.DoublePlain) != math.Float64bits(r.DoubleALP) {
			return fmt.Errorf("%s row %d has different bits", path, i)
		}
	}
	f, err := os.Open(path)
	if err != nil {
		return err
	}
	defer f.Close()
	info, err := f.Stat()
	if err != nil {
		return err
	}
	p, err := parquet.OpenFile(f, info.Size())
	if err != nil {
		return err
	}
	for _, group := range p.Metadata().RowGroups {
		for _, chunk := range group.Columns {
			c := chunk.MetaData
			name := c.PathInSchema[0]
			expected := format.Plain
			if strings.HasSuffix(name, "_alp") {
				expected = format.ALP
			}
			if !slices.Contains(c.Encoding, expected) || c.Codec != format.Zstd {
				return fmt.Errorf("%s: unexpected encoding or codec: %v, %v", name, c.Encoding, c.Codec)
			}
			fmt.Printf("%s\t%s\t%d\t%d\t%v\n", filepath.Base(path), c.PathInSchema[0], c.TotalCompressedSize, c.TotalUncompressedSize, c.Encoding)
		}
	}
	return nil
}

func main() {
	if len(os.Args) < 2 {
		log.Fatal("pass the generated Parquet files as arguments")
	}
	for _, path := range os.Args[1:] {
		if err := check(path); err != nil {
			log.Fatal(err)
		}
	}
}
