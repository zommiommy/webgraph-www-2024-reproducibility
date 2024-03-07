#!/bin/bash -ex
# Reset the environment so that we can run the experiments in a clean environment
set -e

# Set the graph path
export GRAPH=graph
# Change the temporary directory to a non ramfs one to avoid OOM
export TMPDIR="/tmp"

# Set the number of nodes to use for the random access test
export NODES_NUM=10000000
# Enable cpu-specific optimizations
export RUSTFLAGS="-C target-cpu=native"
# Set GraalJVM settings
export JVMOPTS="-server -Xss256K -XX:PretenureSizeThreshold=512M -XX:MaxNewSize=4G -XX:+UseLargePages -XX:+UseTransparentHugePages -XX:+UseNUMA -XX:+UseTLAB -XX:+ResizeTLAB"
# Set the classpath for the java tests
export CLASSPATH=$(find -iname \*.jar | paste -d: -s -)

# Random access test rust
cargo build --release --manifest-path webgraph-rs/Cargo.toml --bin bench_bvgraph 
cat $GRAPH.graph >/dev/null
webgraph-rs/target/release/bench_bvgraph $GRAPH -r $NODES_NUM -s 2>&1 | tee swh-succ-rust.err

# Random access test java
cat $GRAPH.graph >/dev/null
graalvm-jdk-21.0.2+13.1/bin/java $JVMOPTS -Xmx600G \
    it.unimi.dsi.big.webgraph.test.SpeedTest $GRAPH -r $NODES_NUM -m 2>&1 | tee swh-succ-java.err

# BFS test rust
cargo build --manifest-path webgraph-rs/Cargo.toml --release --example bv_bf_visit
# Ensure it's loaded in memory (it task 40 seconds)
cat $GRAPH.graph >/dev/null
webgraph-rs/target/release/examples/bv_bf_visit $GRAPH --repeats 4 -s 2>&1 | tee swh-rust.err

# BFS test java
# Ensure it's loaded in memory (it task 40 seconds)
cat $GRAPH.graph >/dev/null
graalvm-jdk-21.0.2+13.1/bin/java $JVMOPTS -Xmx1024G \
    it.unimi.dsi.law.big.graph.BFS -m --repeats 3 $GRAPH - 2>&1 | tee swh-java.err

