#!/bin/bash -ex
# Reset the environment so that we can run the experiments in a clean environment
set -e

# Change the temporary directory to a non ramfs one to avoid OOM
export TMPDIR="/tmp"
# Enable cpu-specific optimizations
export RUSTFLAGS="-C target-cpu=native"
# Set GraalJVM settings
export JVMOPTS="-server -Xss256K -XX:PretenureSizeThreshold=512M -XX:MaxNewSize=4G -XX:+UseLargePages -XX:+UseTransparentHugePages -XX:+UseNUMA -XX:+UseTLAB -XX:+ResizeTLAB"
# Set the classpath for the java tests
export CLASSPATH=$(find -iname \*.jar | paste -d: -s -)
# Set the number of nodes to use for the random access test
export NODES_NUM=10000000

# Check if the number of arguments is less than 1
if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <path/to/graph-basename>"
    exit 1
fi

BASENAME="$1"

if ! [ -e "$BASENAME.properties" ]; then
  echo "The properties file $BASENAME.properties does not exists"
fi

if ! [ -e "$BASENAME.graph" ]; then
  echo "The graph file $BASENAME.graph does not exists"
fi

if ! [ -e "$BASENAME.offsets" ]; then
    echo "The offsets don't exists, building them"
    cargo run --release --manifest-path webgraph-rs/Cargo.toml --bin build_offsets $BASENAME 
fi

if ! [ -e "$BASENAME.ef" ]; then
    echo "The elias-fano don't exists, building it"
    cargo run --release --manifest-path webgraph-rs/Cargo.toml --bin build_ef $BASENAME 
fi

# Random access test rust
cargo build --release --manifest-path webgraph-rs/Cargo.toml --bin bench_bvgraph 
cat $BASENAME.graph >/dev/null
webgraph-rs/target/release/bench_bvgraph $BASENAME -r $NODES_NUM -s 2>&1 | tee "$BASENAME-succ-rust.err"

# Random access test java
cat $BASENAME.graph >/dev/null
graalvm-jdk-21.0.2+13.1/bin/java $JVMOPTS -Xmx600G \
    it.unimi.dsi.big.webgraph.test.SpeedTest $BASENAME -r $NODES_NUM -m 2>&1 | tee "$BASENAME-succ-java.err"

# BFS test rust
cargo build --manifest-path webgraph-rs/Cargo.toml --release --example bv_bf_visit
# Ensure it's loaded in memory (it task 40 seconds)
cat $BASENAME.graph >/dev/null
webgraph-rs/target/release/examples/bv_bf_visit $BASENAME --repeats 4 -s 2>&1 | tee "$BASENAME-rust.err"

# BFS test java
# Ensure it's loaded in memory (it task 40 seconds)
cat $BASENAME.graph >/dev/null
graalvm-jdk-21.0.2+13.1/bin/java $JVMOPTS -Xmx1024G \
    it.unimi.dsi.law.big.graph.BFS -m --repeats 3 $BASENAME - 2>&1 | tee "$BASENAME-java.err"
