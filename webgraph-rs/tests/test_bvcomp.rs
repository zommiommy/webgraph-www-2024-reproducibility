/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use lender::*;
use std::{fs::File, io::BufWriter};
use tempfile::NamedTempFile;

const NODES: usize = 325557;

use anyhow::Result;
use dsi_bitstream::prelude::*;
use dsi_progress_logger::*;
use webgraph::prelude::*;
use Code::{Delta, Gamma, Unary, Zeta};

fn logger_init() {
    env_logger::builder().is_test(true).try_init().unwrap();
}

#[cfg_attr(feature = "slow_tests", test)]
#[cfg_attr(not(feature = "slow_tests"), allow(dead_code))]
fn test_bvcomp_slow() -> Result<()> {
    _test_bvcomp_slow::<LE>().and(_test_bvcomp_slow::<BE>())
}

fn _test_bvcomp_slow<E: Endianness>() -> Result<()> {
    logger_init();

    let tmp_file = NamedTempFile::new()?;
    let tmp_path = tmp_file.path();
    for outdegrees in [Code::Unary, Gamma, Delta] {
        for references in [Unary, Gamma, Delta] {
            for blocks in [Unary, Gamma, Delta] {
                for intervals in [Unary, Gamma, Delta] {
                    for residuals in [
                        Unary,
                        Gamma,
                        Delta,
                        Zeta { k: 3 },
                        Zeta { k: 1 },
                        Zeta { k: 5 },
                    ] {
                        for compression_window in [0, 1, 2, 4, 7, 8, 10] {
                            for min_interval_length in [0, 2, 4, 7, 8, 10] {
                                for max_ref_count in [0, 1, 2, 3] {
                                    let compression_flags = CompFlags {
                                        outdegrees,
                                        references,
                                        blocks,
                                        intervals,
                                        residuals,
                                        min_interval_length,
                                        compression_window,
                                        max_ref_count,
                                    };

                                    let seq_graph =
                                        webgraph::graphs::bvgraph::sequential::BVGraphSeq::with_basename(
                                            "tests/data/cnr-2000",
                                        )
                                        .endianness::<BE>()
                                        .load()?;

                                    let writer = <DynCodesEncoder<BE, _>>::new(
                                        <BufBitWriter<BE, _>>::new(<WordAdapter<usize, _>>::new(
                                            BufWriter::new(File::create(tmp_path)?),
                                        )),
                                        &compression_flags,
                                    );
                                    let mut bvcomp = BVComp::new(
                                        writer,
                                        compression_window,
                                        min_interval_length,
                                        max_ref_count,
                                        0,
                                    );

                                    let mut pl = ProgressLogger::default();
                                    pl.display_memory(true)
                                        .item_name("node")
                                        .expected_updates(Some(NODES));

                                    pl.start("Compressing...");

                                    let mut iter_nodes = seq_graph.iter();
                                    while let Some((_, iter)) = iter_nodes.next() {
                                        bvcomp.push(iter)?;
                                        pl.light_update();
                                    }

                                    pl.done();
                                    bvcomp.flush()?;

                                    let code_reader = DynCodesDecoder::new(
                                        BufBitReader::<BE, _>::new(MemWordReader::<u32, _>::new(
                                            MmapBackend::load(
                                                tmp_path,
                                                mmap_rs::MmapFlags::empty(),
                                            )?,
                                        )),
                                        &compression_flags,
                                    )?;
                                    let mut seq_reader1 = sequential::Iter::new(
                                        code_reader,
                                        compression_flags.compression_window,
                                        compression_flags.min_interval_length,
                                        NODES,
                                    );

                                    pl.start("Checking equality...");
                                    let mut iter_nodes = seq_graph.iter();
                                    for _ in 0..seq_graph.num_nodes() {
                                        let (node0, iter0) = iter_nodes.next().unwrap();
                                        let (node1, iter1) = seq_reader1.next().unwrap();
                                        assert_eq!(node0, node1);
                                        assert_eq!(
                                            iter0.collect::<Vec<_>>(),
                                            iter1.collect::<Vec<_>>()
                                        );
                                        pl.light_update();
                                    }
                                    pl.done();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    std::fs::remove_file(tmp_path)?;
    Ok(())
}
