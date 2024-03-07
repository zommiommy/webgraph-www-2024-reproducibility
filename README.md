# webgraph-www-2024-replication ![GitHub CI](https://github.com/zommiommy/webgraph-www-2024-replication/actions/workflows/experiments.yml/badge.svg) ![license](https://img.shields.io/crates/l/webgraph)

This is a replication package for the paper

    Tommaso Fontana, Sebastiano Vigna, and Stefano Zacchiroli. 2024.
    WebGraph: The Next Generation (Is in Rust).
    In Companion Proceedings of the ACM Web Conference 2024 (WWW ’24 Companion), 
    May 13–17, 2024, Singapore, Singapore. ACM, New York, NY, USA, 4 pages.
    https://doi.org/10.1145/3589335.3651581

## Graphs

### [EU-2015](https://law.di.unimi.it/webdata/eu-2015/)

```bash
wget http://data.law.di.unimi.it/webdata/eu-2015/eu-2015.properties
wget http://data.law.di.unimi.it/webdata/eu-2015/eu-2015.graph
```

### [En-wiki 2023](https://law.di.unimi.it/webdata/enwiki-2023/)

```bash
wget http://data.law.di.unimi.it/webdata/enwiki-2023/enwiki-2023.properties
wget http://data.law.di.unimi.it/webdata/enwiki-2023/enwiki-2023.graph
```

### [Swh 2023-09-06](https://docs.softwareheritage.org/devel/swh-dataset/graph/dataset.html)

The software heritage 2023-09-06 snapshot can be downloaded from their S3 bucket
`s3://softwareheritage/graph/2023-09-06/compressed`.

## Machines

Both `EU-2015` and `En-wiki2023` were tested on the following machine:

```
Fedora Linux 38 (Workstation Edition)
Linux blew 6.5.12-200.fc38.x86_64 #1 SMP PREEMPT_DYNAMIC Mon Nov 20 22:12:09 UTC 2023 x86_64 GNU/Linux
Intel I7-12700kf CPU @ 3.60GHz
64GB RAM
```

while `Swh 2023-09-06` was tested on:

```
Debian GNU/Linux 11 (bullseye)
Linux maxxi 5.10.0-26-amd64 #1 SMP Debian 5.10.197-1 (2023-09-29) x86_64 GNU/Linux
2x Intel(R) Xeon(R) Gold 6130 CPU @ 2.10GHz
4TB RAM
```

## Java version

The JVM
[`graalvm-ce-java11-21.0.2+13.1-linux-amd64`](https://download.oracle.com/graalvm/21/archive/graalvm-jdk-21.0.2_linux-x64_bin.tar.gz)
was used on both machines:

```
$ sha256sum graalvm-jdk-21_linux-x64_bin.tar.gz
ee6286773c659afeefdf2f989a133e7a631c60897f2263ac183794ee1d6438f4  graalvm-jdk-21_linux-x64_bin.tar.gz
```

The same version for other OSs and CPUs can be downloaded
[here](https://www.oracle.com/java/technologies/javase/graalvm-jdk21-archive-downloads.html).

## Rust version

The following Rust tools were used on both machines:

```
rust toolchain stable-x86_64-unknown-linux-gnu
cargo 1.75.0 (1d8b05cdd 2023-11-20)
rustc 1.75.0 (82e1608df 2023-12-21)
rustup 1.26.0 (5af9b9484 2023-04-05)
```

This can be reproduced by [installing
rust](https://www.rust-lang.org/tools/install) and running:

```
rustup install 1.75.0
```

## Scripts

- `exp.sh` Expects the graph basename as argument e.g.:

```shell
./exp.sh webgraph-rs/tests/data/cnr-2000
```

The [CI file of this repo](https://github.com/zommiommy/webgraph-www-2024-replication/blob/main/.github/workflows/experiments.yml)are an example about how to run the
benchmarks on a Linux x86_64 machine.

`exp.sh` expects the Graal VM tar to be unzipped in the root of this
repository, it has to be executed from the root of this repository, and you have
to modify the `GRAPH` export inside the script to set the path to where the SWH
graph was downloaded. You probably also want to modify the `TMPDIR` export to
use a folder with enough space.

To avoid caching differences, before running every benchmark we run 
`cat $GRAPH >/dev/null` to bring force loading of the graph and get into a consistent state.

## WebGraph version

Note that this replication package uses the Rust version of WebGraph that was
current at the time of the experiments. The crate is in continuous development,
and in particular there is now a CLI interface accessing the binaries. To run
these experiments with a newer version, please update the scripts to use the
CLI, replacing `bf_visit` with `webgraph bench bf_visit`.

This version of `webgraph-rs` [is not compatible with newer Rust
versions](https://github.com/rust-lang/rust/issues/121604#event-11935096017),
use the [newest version from github](https://github.com/vigna/webgraph-rs) if
that's needed.
