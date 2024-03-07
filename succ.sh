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
  cargo build --release --bin bench_bvgraph
  ./target/release/bench_bvgraph -r 10000000 -s $g 2>$g-succ-rust.err 1>&2
  java -Xmx60G $JVMOPTS \
    it.unimi.dsi.big.webgraph.test.SpeedTest -r 10000000 -m $g 2>$g-succ-java.err 1>&2
done