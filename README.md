# webgraph-www-2024-replication

Replication package for the paper:

    Tommaso Fontana, Sebastiano Vigna, and Stefano Zacchiroli. 2024.
    WebGraph: The Next Generation (Is in Rust).
	In Companion Proceedings of the ACM Web Conference 2024 (WWW ’24 Companion), May 13–17, 2024, Singapore, Singapore. ACM, New York, NY, USA, 4 pages.
	https://doi.org/10.1145/3589335.3651581

## Graphs

### [EU-2015](https://law.di.unimi.it/webdata/eu-2015/)
```bash
$ wget http://data.law.di.unimi.it/webdata/eu-2015/eu-2015.properties
$ wget http://data.law.di.unimi.it/webdata/eu-2015/eu-2015.graph
```

### [En-wiki 2023](https://law.di.unimi.it/webdata/enwiki-2023/)
```bash
$ wget http://data.law.di.unimi.it/webdata/enwiki-2023/enwiki-2023.properties
$ wget http://data.law.di.unimi.it/webdata/enwiki-2023/enwiki-2023.graph
```

### [Swh 2023-09-06](https://docs.softwareheritage.org/devel/swh-dataset/graph/dataset.html)
The software heritage 2023-09-06 snapshot can be downloaded from their S3 bucket `s3://softwareheritage/graph/2023-09-06/compressed`. 

The experiments on the SWH graph were run on the following machine:
```
Linux maxxi 5.10.0-26-amd64 #1 SMP Debian 5.10.197-1 (2023-09-29) x86_64 GNU/Linux
2x Intel(R) Xeon(R) Gold 6130 CPU @ 2.10GHz
4TB RAM
```
with the following rust tools:
```
rust toolchain stable-x86_64-unknown-linux-gnu
cargo 1.75.0 (1d8b05cdd 2023-11-20)
rustc 1.75.0 (82e1608df 2023-12-21)
rustup 1.26.0 (5af9b9484 2023-04-05)
```
this can be reproduced by [installing rust](https://www.rust-lang.org/tools/install) and running:
```
$ rustup install 1.75.0
```
and as JVM [`graalvm-ce-java11-21.0.2+13.1-linux-amd64`](https://download.oracle.com/graalvm/21/archive/graalvm-jdk-21.0.2_linux-x64_bin.tar.gz):
```
$ sha256sum graalvm-jdk-21_linux-x64_bin.tar.gz
ee6286773c659afeefdf2f989a133e7a631c60897f2263ac183794ee1d6438f4  graalvm-jdk-21_linux-x64_bin.tar.gz
```
the same version for other os and cpus can be downloaded [here](https://www.oracle.com/java/technologies/javase/graalvm-jdk21-archive-downloads.html).
