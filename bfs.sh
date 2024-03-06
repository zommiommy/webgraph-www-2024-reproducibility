#!/bin/bash -ex
for g in enwiki-2023 eu-2015; do
    cargo build --release --example bv_bf_visit
    cat $g.graph >/dev/null
    ./target/release/examples/bv_bf_visit --repeats 4 -s $g 2>$g-rust.err
    cat $g.graph >/dev/null
    java -Xmx60G $JVMOPTS \
        it.unimi.dsi.law.big.graph.BFS -m --repeats 4 $g - 2>$g-java.err
done