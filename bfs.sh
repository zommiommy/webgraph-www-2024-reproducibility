#!/bin/bash -ex

# Reset the environment so that we can run the experiments in a clean environment
set -e

# Change the temporary directory to a non ramfs one to avoid OOM
export TMPDIR="/tmp"

# Set GraalJVM settings
export JVMOPTS="-server -Xss256K -XX:PretenureSizeThreshold=512M -XX:MaxNewSize=4G -XX:+UseLargePages -XX:+UseTransparentHugePages -XX:+UseNUMA -XX:+UseTLAB -XX:+ResizeTLAB"
# Set the classpath for the java tests
export CLASSPATH=$(find -iname \*.jar | paste -d: -s -)

for g in enwiki-2023 eu-2015; do
    cargo build --release --example bv_bf_visit
    cat $g.graph >/dev/null
    ./target/release/examples/bv_bf_visit --repeats 4 -s $g 2>$g-rust.err
    cat $g.graph >/dev/null
    java -Xmx60G $JVMOPTS \
        it.unimi.dsi.law.big.graph.BFS -m --repeats 4 $g - 2>$g-java.err
done