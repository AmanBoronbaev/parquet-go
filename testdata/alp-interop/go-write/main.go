package main

import (
	p "github.com/parquet-go/parquet-go"
	"log"
	"math"
	"os"
	"path/filepath"
)

type Row struct {
	F float32 `parquet:"f_plain"`
	D float64 `parquet:"d_plain"`
}

func main() {
	for _, path := range os.Args[1:] {
		rows := make([]Row, 65537)
		seed := uint64(0x243f6a8885a308d3)
		for i := range rows {
			seed ^= seed << 13
			seed ^= seed >> 7
			seed ^= seed << 17
			d := float64(int64(seed%100001)-50000) / 100
			f := float32(d)
			switch filepath.Base(path) {
			case "raw-bits.parquet":
				f, d = math.Float32frombits(uint32(seed)), math.Float64frombits(seed)
			case "exceptions.parquet":
				if i%31 == 0 {
					switch (i / 31) % 4 {
					case 0:
						d = math.Copysign(0, -1)
						f = float32(d)
					case 1:
						f, d = math.Float32frombits(0x7fc00042), math.Float64frombits(0x7ff8000000000042)
					case 2:
						d = math.Inf(1)
						f = float32(d)
					case 3:
						f, d = math.SmallestNonzeroFloat32, math.SmallestNonzeroFloat64
					}
				}
			}
			rows[i] = Row{f, d}
		}
		if err := p.WriteFile("go-"+filepath.Base(path), rows, p.DefaultEncodingFor(p.Float, &p.ALP), p.DefaultEncodingFor(p.Double, &p.ALP)); err != nil {
			log.Fatal(err)
		}
	}
}
