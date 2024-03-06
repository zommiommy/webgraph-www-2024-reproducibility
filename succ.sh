#!/bin/bash -ex
for g in enwiki-2023 eu-2015; do
  cargo build --release --bin bench_bvgraph
  ./target/release/bench_bvgraph -r 10000000 -s $g 2>$g-succ-rust.err 1>&2
  java -Xmx60G $JVMOPTS \
    it.unimi.dsi.big.webgraph.test.SpeedTest -r 10000000 -m $g 2>$g-succ-java.err 1>&2
done