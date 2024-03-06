/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use super::codecs::MemoryFlags;
use super::*;
use crate::graphs::bvgraph::EmptyDict;
use crate::prelude::*;
use anyhow::{Context, Result};
use dsi_bitstream::prelude::*;
use epserde::prelude::*;
use java_properties;
use sealed::sealed;
use std::io::*;
use std::path::{Path, PathBuf};
use sux::traits::IndexedDict;

/// Sequential or random access.
#[doc(hidden)]
#[sealed]
pub trait Access: 'static {}

#[derive(Debug, Clone)]
pub struct Sequential {}
#[sealed]
impl Access for Sequential {}

#[derive(Debug, Clone)]
pub struct Random {}
#[sealed]
impl Access for Random {}

/// [`Static`] or [`Dynamic`] dispatch.
#[sealed]
pub trait Dispatch: 'static {}

/// Static dispatch.
///
/// You have to specify all codes used of the graph. The defaults
/// are the same as the default parameters of the Java version.
#[derive(Debug, Clone)]
pub struct Static<
    const OUTDEGREES: usize = { const_codes::GAMMA },
    const REFERENCES: usize = { const_codes::UNARY },
    const BLOCKS: usize = { const_codes::GAMMA },
    const INTERVALS: usize = { const_codes::GAMMA },
    const RESIDUALS: usize = { const_codes::ZETA },
    const K: usize = 3,
> {}

#[sealed]
impl<
        const OUTDEGREES: usize,
        const REFERENCES: usize,
        const BLOCKS: usize,
        const INTERVALS: usize,
        const RESIDUALS: usize,
        const K: usize,
    > Dispatch for Static<OUTDEGREES, REFERENCES, BLOCKS, INTERVALS, RESIDUALS, K>
{
}

/// Dynamic dispatch.
///
/// Parameters are retrieved from the graph properties.
#[derive(Debug, Clone)]
pub struct Dynamic {}

#[sealed]
impl Dispatch for Dynamic {}

/// Load mode.
///
/// The load mode is the way the graph data is accessed. Each load mode has
/// a corresponding strategy to access the graph and the offsets.
///
/// You can set both modes with [`Load::mode`], or set them separately with
/// [`Load::graph_mode`] and [`Load::offsets_mode`].
#[sealed]
pub trait LoadMode: 'static {
    type Factory<E: Endianness>: BitReaderFactory<E>;

    fn new_factory<E: Endianness>(
        graph: &PathBuf,
        flags: codecs::MemoryFlags,
    ) -> Result<Self::Factory<E>>;

    type Offsets: IndexedDict<Input = usize, Output = usize>;

    fn load_offsets(offsets: &PathBuf, flags: MemoryFlags) -> Result<MemCase<Self::Offsets>>;
}

/// The graph is read from a file; offsets are fully deserialized in memory.
///
/// Note that you must guarantee that the graph file is padded with enough
/// zeroes so that it can be read one `u32` at a time.
#[derive(Debug, Clone)]
pub struct File {}
#[sealed]
impl LoadMode for File {
    type Factory<E: Endianness> = FileFactory<E>;
    type Offsets = EF;

    fn new_factory<E: Endianness>(
        graph: &PathBuf,
        _flags: MemoryFlags,
    ) -> Result<Self::Factory<E>> {
        Ok(FileFactory::<E>::new(graph)?)
    }

    fn load_offsets(offsets: &PathBuf, _flags: MemoryFlags) -> Result<MemCase<Self::Offsets>> {
        Ok(EF::load_full(offsets)?.into())
    }
}

/// The graph and offsets are memory mapped.
///
/// This is the default mode. You can [set memory-mapping flags](Load::flags).
#[derive(Debug, Clone)]
pub struct Mmap {}
#[sealed]
impl LoadMode for Mmap {
    type Factory<E: Endianness> = MmapBackend<u32>;
    type Offsets = <EF as DeserializeInner>::DeserType<'static>;

    fn new_factory<E: Endianness>(graph: &PathBuf, flags: MemoryFlags) -> Result<Self::Factory<E>> {
        Ok(MmapBackend::load(graph, flags.into())?)
    }

    fn load_offsets(offsets: &PathBuf, flags: MemoryFlags) -> Result<MemCase<Self::Offsets>> {
        EF::mmap(offsets, flags.into())
    }
}

/// The graph and offsets are loaded into allocated memory.
#[derive(Debug, Clone)]
pub struct LoadMem {}
#[sealed]
impl LoadMode for LoadMem {
    type Factory<E: Endianness> = MemoryFactory<E, Box<[u32]>>;
    type Offsets = <EF as DeserializeInner>::DeserType<'static>;

    fn new_factory<E: Endianness>(
        graph: &PathBuf,
        _flags: MemoryFlags,
    ) -> Result<Self::Factory<E>> {
        Ok(MemoryFactory::<E, _>::new_mem(graph)?)
    }

    fn load_offsets(offsets: &PathBuf, _flags: MemoryFlags) -> Result<MemCase<Self::Offsets>> {
        Ok(EF::load_mem(offsets)?)
    }
}

/// The graph and offsets are loaded into memory obtained via `mmap()`.
///
/// You can [set memory-mapping flags](Load::flags).
#[derive(Debug, Clone)]
pub struct LoadMmap {}
#[sealed]
impl LoadMode for LoadMmap {
    type Factory<E: Endianness> = MemoryFactory<E, MmapBackend<u32>>;
    type Offsets = <EF as DeserializeInner>::DeserType<'static>;

    fn new_factory<E: Endianness>(graph: &PathBuf, flags: MemoryFlags) -> Result<Self::Factory<E>> {
        Ok(MemoryFactory::<E, _>::new_mmap(graph, flags)?)
    }

    fn load_offsets(offsets: &PathBuf, flags: MemoryFlags) -> Result<MemCase<Self::Offsets>> {
        EF::load_mmap(offsets, flags.into())
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct LoadConfig<E: Endianness, A: Access, D: Dispatch, GLM: LoadMode, OLM: LoadMode> {
    pub(crate) basename: PathBuf,
    pub(crate) graph_load_flags: MemoryFlags,
    pub(crate) offsets_load_flags: MemoryFlags,
    pub(crate) _marker: std::marker::PhantomData<(E, A, D, GLM, OLM)>,
}

impl<E: Endianness, A: Access, D: Dispatch, GLM: LoadMode, OLM: LoadMode>
    LoadConfig<E, A, D, GLM, OLM>
{
    /// Set the endianness of the graph and offsets file.
    pub fn endianness<E2: Endianness>(self) -> LoadConfig<E2, A, D, GLM, OLM> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: self.graph_load_flags,
            offsets_load_flags: self.offsets_load_flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, A: Access, D: Dispatch, GLM: LoadMode, OLM: LoadMode>
    LoadConfig<E, A, D, GLM, OLM>
{
    /// Choose between [`Static`] and [`Dynamic`] dispatch.
    pub fn dispatch<D2: Dispatch>(self) -> LoadConfig<E, A, D2, GLM, OLM> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: self.graph_load_flags,
            offsets_load_flags: self.offsets_load_flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, A: Access, D: Dispatch, GLM: LoadMode, OLM: LoadMode>
    LoadConfig<E, A, D, GLM, OLM>
{
    /// Choose the [`LoadMode`] for the graph and offsets.
    pub fn mode<LM: LoadMode>(self) -> LoadConfig<E, A, D, LM, LM> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: self.graph_load_flags,
            offsets_load_flags: self.offsets_load_flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, A: Access, D: Dispatch> LoadConfig<E, A, D, Mmap, Mmap> {
    /// Set flags for memory-mapping (both graph and offsets).
    pub fn flags(self, flags: MemoryFlags) -> LoadConfig<E, A, D, Mmap, Mmap> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: flags,
            offsets_load_flags: flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, A: Access, D: Dispatch> LoadConfig<E, A, D, LoadMmap, LoadMmap> {
    /// Set flags for memory obtained from `mmap()` (both graph and offsets).
    pub fn flags(self, flags: MemoryFlags) -> LoadConfig<E, A, D, LoadMmap, LoadMmap> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: flags,
            offsets_load_flags: flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, A: Access, D: Dispatch, GLM: LoadMode, OLM: LoadMode>
    LoadConfig<E, A, D, GLM, OLM>
{
    /// Choose the [`LoadMode`] for the graph only.
    pub fn graph_mode<NGLM: LoadMode>(self) -> LoadConfig<E, A, D, NGLM, OLM> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: self.graph_load_flags,
            offsets_load_flags: self.offsets_load_flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, A: Access, D: Dispatch, OLM: LoadMode> LoadConfig<E, A, D, Mmap, OLM> {
    /// Set flags for memory-mapping the graph.
    pub fn graph_flags(self, flags: MemoryFlags) -> LoadConfig<E, A, D, Mmap, OLM> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: flags,
            offsets_load_flags: self.offsets_load_flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, A: Access, D: Dispatch, OLM: LoadMode> LoadConfig<E, A, D, LoadMmap, OLM> {
    /// Set flags for memory obtained from `mmap()` for the graph.
    pub fn graph_flags(self, flags: MemoryFlags) -> LoadConfig<E, A, D, LoadMmap, OLM> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: flags,
            offsets_load_flags: self.offsets_load_flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, D: Dispatch, GLM: LoadMode, OLM: LoadMode> LoadConfig<E, Random, D, GLM, OLM> {
    /// Choose the [`LoadMode`] for the graph only.
    pub fn offsets_mode<NOLM: LoadMode>(self) -> LoadConfig<E, Random, D, GLM, NOLM> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: self.graph_load_flags,
            offsets_load_flags: self.offsets_load_flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, D: Dispatch, GLM: LoadMode> LoadConfig<E, Random, D, GLM, Mmap> {
    /// Set flags for memory-mapping the offsets.
    pub fn offsets_flags(self, flags: MemoryFlags) -> LoadConfig<E, Random, D, GLM, Mmap> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: self.graph_load_flags,
            offsets_load_flags: flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, D: Dispatch, GLM: LoadMode> LoadConfig<E, Random, D, GLM, LoadMmap> {
    /// Set flags for memory obtained from `mmap()` for the graph.
    pub fn offsets_flags(self, flags: MemoryFlags) -> LoadConfig<E, Random, D, GLM, LoadMmap> {
        LoadConfig {
            basename: self.basename,
            graph_load_flags: self.graph_load_flags,
            offsets_load_flags: flags,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: Endianness, GLM: LoadMode, OLM: LoadMode> LoadConfig<E, Random, Dynamic, GLM, OLM> {
    /// Load a random-access graph with dynamic dispatch.
    pub fn load(
        mut self,
    ) -> anyhow::Result<BVGraph<DynCodesDecoderFactory<E, GLM::Factory<E>, OLM::Offsets>>>
    where
        for<'a> <<GLM as LoadMode>::Factory<E> as BitReaderFactory<E>>::BitReader<'a>:
            CodeRead<E> + BitSeek,
    {
        self.basename.set_extension("properties");
        let (num_nodes, num_arcs, comp_flags) = parse_properties::<E>(&self.basename)?;
        self.basename.set_extension("graph");
        let factory = GLM::new_factory(&self.basename, self.graph_load_flags)?;
        self.basename.set_extension("ef");
        let offsets = OLM::load_offsets(&self.basename, self.offsets_load_flags)?;

        Ok(BVGraph::new(
            DynCodesDecoderFactory::new(factory, offsets, comp_flags)?,
            comp_flags.min_interval_length,
            comp_flags.compression_window,
            num_nodes,
            num_arcs,
        ))
    }
}

impl<E: Endianness, GLM: LoadMode, OLM: LoadMode> LoadConfig<E, Sequential, Dynamic, GLM, OLM> {
    /// Load a sequential graph with dynamic dispatch.
    pub fn load(
        mut self,
    ) -> anyhow::Result<
        BVGraphSeq<DynCodesDecoderFactory<E, GLM::Factory<E>, EmptyDict<usize, usize>>>,
    >
    where
        for<'a> <<GLM as LoadMode>::Factory<E> as BitReaderFactory<E>>::BitReader<'a>: CodeRead<E>,
    {
        self.basename.set_extension("properties");
        let (num_nodes, num_arcs, comp_flags) = parse_properties::<E>(&self.basename)?;
        self.basename.set_extension("graph");
        let factory = GLM::new_factory(&self.basename, self.graph_load_flags)?;

        Ok(BVGraphSeq::new(
            DynCodesDecoderFactory::new(factory, MemCase::from(EmptyDict::default()), comp_flags)?,
            comp_flags.compression_window,
            comp_flags.min_interval_length,
            num_nodes,
            Some(num_arcs),
        ))
    }
}

impl<
        E: Endianness,
        GLM: LoadMode,
        OLM: LoadMode,
        const OUTDEGREES: usize,
        const REFERENCES: usize,
        const BLOCKS: usize,
        const INTERVALS: usize,
        const RESIDUALS: usize,
        const K: usize,
    >
    LoadConfig<E, Random, Static<OUTDEGREES, REFERENCES, BLOCKS, INTERVALS, RESIDUALS, K>, GLM, OLM>
{
    /// Load a random-access graph with static dispatch.
    pub fn load(
        mut self,
    ) -> anyhow::Result<
        BVGraph<
            ConstCodesDecoderFactory<
                E,
                GLM::Factory<E>,
                OLM::Offsets,
                OUTDEGREES,
                REFERENCES,
                BLOCKS,
                INTERVALS,
                RESIDUALS,
                K,
            >,
        >,
    >
    where
        for<'a> <<GLM as LoadMode>::Factory<E> as BitReaderFactory<E>>::BitReader<'a>:
            CodeRead<E> + BitSeek,
    {
        self.basename.set_extension("properties");
        let (num_nodes, num_arcs, comp_flags) = parse_properties::<E>(&self.basename)?;
        self.basename.set_extension("graph");
        let factory = GLM::new_factory(&self.basename, self.graph_load_flags)?;
        self.basename.set_extension("ef");
        let offsets = OLM::load_offsets(&self.basename, self.offsets_load_flags)?;

        Ok(BVGraph::new(
            ConstCodesDecoderFactory::new(factory, offsets, comp_flags)?,
            comp_flags.min_interval_length,
            comp_flags.compression_window,
            num_nodes,
            num_arcs,
        ))
    }
}

impl<
        E: Endianness,
        GLM: LoadMode,
        OLM: LoadMode,
        const OUTDEGREES: usize,
        const REFERENCES: usize,
        const BLOCKS: usize,
        const INTERVALS: usize,
        const RESIDUALS: usize,
        const K: usize,
    >
    LoadConfig<
        E,
        Sequential,
        Static<OUTDEGREES, REFERENCES, BLOCKS, INTERVALS, RESIDUALS, K>,
        GLM,
        OLM,
    >
{
    /// Load a sequential graph with static dispatch.
    pub fn load(
        mut self,
    ) -> anyhow::Result<
        BVGraphSeq<
            ConstCodesDecoderFactory<
                E,
                GLM::Factory<E>,
                EmptyDict<usize, usize>,
                OUTDEGREES,
                REFERENCES,
                BLOCKS,
                INTERVALS,
                RESIDUALS,
                K,
            >,
        >,
    >
    where
        for<'a> <<GLM as LoadMode>::Factory<E> as BitReaderFactory<E>>::BitReader<'a>: CodeRead<E>,
    {
        self.basename.set_extension("properties");
        let (num_nodes, num_arcs, comp_flags) = parse_properties::<E>(&self.basename)?;
        self.basename.set_extension("graph");
        let factory = GLM::new_factory(&self.basename, self.graph_load_flags)?;

        Ok(BVGraphSeq::new(
            ConstCodesDecoderFactory::new(
                factory,
                MemCase::from(EmptyDict::default()),
                comp_flags,
            )?,
            comp_flags.compression_window,
            comp_flags.min_interval_length,
            num_nodes,
            Some(num_arcs),
        ))
    }
}

/// Read the .properties file and return the endianness
pub fn get_endianess<P: AsRef<Path>>(basename: P) -> Result<String> {
    let path = format!("{}.properties", basename.as_ref().to_string_lossy());
    let f = std::fs::File::open(&path)
        .with_context(|| format!("Cannot open property file {}", path))?;
    let map = java_properties::read(BufReader::new(f))
        .with_context(|| format!("cannot parse {} as a java properties file", path))?;

    let endianness = map
        .get("endianness")
        .map(|x| x.to_string())
        .unwrap_or_else(|| BigEndian::NAME.to_string());

    Ok(endianness)
}

/// Read the .properties file and return the number of nodes, number of arcs and compression flags
/// for the graph. The endianness is checked against the expected one.
pub fn parse_properties<E: Endianness>(path: impl AsRef<Path>) -> Result<(usize, u64, CompFlags)> {
    let name = path.as_ref().to_string_lossy();
    let f = std::fs::File::open(&path)
        .with_context(|| format!("Cannot open property file {}", name))?;
    let map = java_properties::read(BufReader::new(f))
        .with_context(|| format!("cannot parse {} as a java properties file", name))?;

    let num_nodes = map
        .get("nodes")
        .with_context(|| format!("Missing 'nodes' property in {}", name))?
        .parse::<usize>()
        .with_context(|| format!("Cannot parse 'nodes' as usize in {}", name))?;
    let num_arcs = map
        .get("arcs")
        .with_context(|| format!("Missing 'arcs' property in {}", name))?
        .parse::<u64>()
        .with_context(|| format!("Cannot parse arcs as usize in {}", name))?;

    let comp_flags = CompFlags::from_properties::<E>(&map)
        .with_context(|| format!("Cannot parse compression flags from {}", name))?;
    Ok((num_nodes, num_arcs, comp_flags))
}
