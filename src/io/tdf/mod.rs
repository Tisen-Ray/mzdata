//! Reader implementation for Bruker's TDF data files, [`TDFFrameReaderType`] for ion mobility frames
//! and [`TDFSpectrumReaderType`] for summed or sliced spectra.
//!
//! **Requires the `bruker_tdf` feature**
//!
//! Depends upon the `timsrust-tdf` / `timsrust-core` crates (0.6), a cross-platform,
//! pure Rust implementation of the Bruker-specific file reading behaviors, and
//! [`rusqlite`] for reading the SQLite3 .tdf files.
mod constants;
mod arrays;
mod sql;
mod reader;
mod calibration;

/// Error raised while opening or reading a Bruker TDF dataset.
///
/// Replaces the timsrust 0.4 facade-level `TimsRustError`: the 0.6
/// ecosystem exposes per-reader error types instead, and mzdata's own SQL
/// layer raises `rusqlite::Error` directly.
#[derive(Debug, thiserror::Error)]
pub enum TdfError {
    #[error("frame reader error: {0}")]
    FrameReader(#[from] timsrust_tdf::FrameReaderError),
    #[error("ion reader error: {0}")]
    IonReader(#[from] timsrust_core::FrameReaderError),
    #[error("metadata reader error: {0}")]
    MetadataReader(#[from] timsrust_tdf::MetadataReaderError),
    #[error("SQL error: {0}")]
    Sql(#[from] rusqlite::Error),
}

pub use reader::{
    TDFFrameReader, TDFFrameReaderType, TDFSpectrumReader, TDFSpectrumReaderType, is_tdf,
};
pub use sql::{ChromatographyData, SQLTrace};

pub use calibration::{
    CalibrationParameters,
    ConvertableDomain,
    IonMobilityCalibrationError,
    MzCalibration,
    Scan2ImConverter,
    TimsCalibration,
    Tof2MzConverter,
    MzCalibrationError,
    clamp_u32,
    TimsCalibrationModel,
    TimsCalibrationModel2,
    MzCalibrationModel,
    MzCalibrationModel2
};