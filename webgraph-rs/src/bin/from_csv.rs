use clap::Parser;
use dsi_bitstream::prelude::{Endianness, BE};
use dsi_progress_logger::*;
use itertools::Itertools;
use rayon::slice::ParallelSliceMut;
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use webgraph::graphs::arc_list_graph::ArcListGraph;
use webgraph::labels::Left;
use webgraph::prelude::*;

#[derive(Parser, Debug)]
#[command(about = "Compress a CSV graph from stdin into webgraph. This does not support any form of escaping.", long_about = None)]
struct Args {
    /// The basename of the dst.
    basename: String,

    #[arg(long)]
    /// The number of nodes in the graph
    num_nodes: usize,

    #[arg(long)]
    /// The number of arcs in the graph
    num_arcs: Option<usize>,

    #[clap(flatten)]
    csv_args: CSVArgs,

    #[clap(flatten)]
    num_cpus: NumCpusArg,

    #[clap(flatten)]
    pa: PermutationArgs,

    #[clap(flatten)]
    ca: CompressArgs,
}

fn main() {
    stderrlog::new()
        .verbosity(2)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let args = Args::parse();

    let mut group_by = SortPairs::new(args.pa.batch_size, temp_dir(&args.pa.temp_dir)).unwrap();
    let mut nodes = BTreeMap::new();

    // read the csv and put it inside the sort pairs
    let stdin = std::io::stdin();
    let mut pl = ProgressLogger::default();
    pl.display_memory(true)
        .item_name("lines")
        .expected_updates(args.csv_args.max_lines.or(args.num_arcs));
    pl.start("Reading arcs CSV");

    let mut iter = stdin.lock().lines();
    // skip the first few lines
    for _ in 0..args.csv_args.lines_to_skip {
        iter.next().unwrap().unwrap();
    }
    let mut line_id = 0;
    for line in iter {
        // break if we reached the end
        if let Some(max_lines) = args.csv_args.max_lines {
            if line_id > max_lines {
                break;
            }
        }
        let line = line.unwrap();
        // skip comment
        if line.trim().starts_with(args.csv_args.line_comment_simbol) {
            continue;
        }

        // split the csv line into the args
        let vals = line.split(args.csv_args.csv_separator).collect::<Vec<_>>();
        let src = vals[0];
        let dst = vals[1];

        // parse if numeric, or build a node list
        let src_id = if args.csv_args.numeric {
            src.parse::<usize>().unwrap()
        } else {
            let node_id = nodes.len();
            *nodes.entry(src.to_string()).or_insert(node_id)
        };
        let dst_id = if args.csv_args.numeric {
            dst.parse::<usize>().unwrap()
        } else {
            let node_id = nodes.len();
            *nodes.entry(dst.to_string()).or_insert(node_id)
        };

        group_by.push(src_id, dst_id).unwrap();
        pl.light_update();
        line_id += 1;
    }
    pl.done();
    log::info!("Arcs read: {}", line_id);

    // conver the iter to a graph
    let g = Left(ArcListGraph::new(
        args.num_nodes,
        group_by
            .iter()
            .unwrap()
            .map(|(src, dst, _)| (src, dst))
            .dedup(),
    ));
    // compress it
    let target_endianness = args.ca.endianess.clone();
    BVComp::parallel_endianness(
        &args.basename,
        &g,
        args.num_nodes,
        args.ca.into(),
        args.num_cpus.num_cpus,
        temp_dir(args.pa.temp_dir),
        &target_endianness.unwrap_or_else(|| BE::NAME.into()),
    )
    .unwrap();

    // save the nodes
    if !args.csv_args.numeric {
        let mut file = std::fs::File::create(format!("{}.nodes", args.basename)).unwrap();
        let mut buf = std::io::BufWriter::new(&mut file);
        let mut nodes = nodes.into_iter().collect::<Vec<_>>();
        // sort based on the idx
        nodes.par_sort_by(|(_, a), (_, b)| a.cmp(b));
        for (node, _) in nodes {
            buf.write_all(node.as_bytes()).unwrap();
            buf.write_all(b"\n").unwrap();
        }
    }
}
