use mzpeaks::{
    CentroidLike, CentroidPeak, DeconvolutedCentroidLike, DeconvolutedPeak, IntensityMeasurement,
    MZLocated,
};
use thiserror::Error;

use crate::spectrum::{
    bindata::{ArrayRetrievalError, ArrayType, BuildFromArrayMap},
    MultiLayerSpectrum, SpectrumLike,
};

use super::{DetailLevel, MemorySpectrumSource, SpectrumSource};

#[cfg(feature = "mgf")]
use crate::io::mgf::MGFReaderType;
#[cfg(feature = "mzml")]
use crate::io::mzml::MzMLReaderType;
#[cfg(feature = "mzmlb")]
use crate::io::mzmlb::MzMLbReaderType;
#[cfg(feature = "thermo")]
use crate::io::thermo::ThermoRawReaderType;
use crate::prelude::SeekRead;

/// Public query contract for extracted ion chromatograms.
///
/// The Phase 1 surface keeps the required `m/z` window front and center and
/// layers optional RT, MS level, mobility, and minimum-intensity filtering on
/// top of that shared reader-native entry point.
#[derive(Debug, Clone, PartialEq)]
pub struct EICQuery {
    /// Required lower bound for the `m/z` window.
    pub mz_min: f64,
    /// Required upper bound for the `m/z` window.
    pub mz_max: f64,
    /// Optional lower retention-time bound.
    pub rt_min: Option<f64>,
    /// Optional upper retention-time bound.
    pub rt_max: Option<f64>,
    /// Optional MS level filter.
    pub ms_level: Option<u8>,
    /// Optional lower ion-mobility bound.
    pub mobility_min: Option<f64>,
    /// Optional upper ion-mobility bound.
    pub mobility_max: Option<f64>,
    /// Optional minimum per-point intensity threshold.
    pub min_intensity: Option<f32>,
}

impl EICQuery {
    /// Create a query for the required `m/z` range.
    #[must_use]
    pub fn new(mz_min: f64, mz_max: f64) -> Self {
        Self {
            mz_min,
            mz_max,
            rt_min: None,
            rt_max: None,
            ms_level: None,
            mobility_min: None,
            mobility_max: None,
            min_intensity: None,
        }
    }

    /// Add an optional retention-time range.
    #[must_use]
    pub fn with_rt_range(mut self, rt_min: f64, rt_max: f64) -> Self {
        self.rt_min = Some(rt_min);
        self.rt_max = Some(rt_max);
        self
    }

    /// Restrict the query to a single MS level.
    #[must_use]
    pub fn with_ms_level(mut self, ms_level: u8) -> Self {
        self.ms_level = Some(ms_level);
        self
    }

    /// Add an optional ion-mobility range.
    #[must_use]
    pub fn with_mobility_range(mut self, mobility_min: f64, mobility_max: f64) -> Self {
        self.mobility_min = Some(mobility_min);
        self.mobility_max = Some(mobility_max);
        self
    }

    /// Add an optional minimum intensity threshold.
    #[must_use]
    pub fn with_min_intensity(mut self, min_intensity: f32) -> Self {
        self.min_intensity = Some(min_intensity);
        self
    }

    pub(crate) fn prepare(&self) -> Result<PreparedEICQuery, EICError> {
        let (mz_min, mz_max) = normalize_bounds(self.mz_min, self.mz_max, "m/z")?;
        let (rt_min, rt_max) =
            normalize_optional_bounds(self.rt_min, self.rt_max, "retention time")?;
        let (mobility_min, mobility_max) =
            normalize_optional_bounds(self.mobility_min, self.mobility_max, "ion mobility")?;

        let min_intensity = self.min_intensity.unwrap_or_default();
        if !min_intensity.is_finite() || min_intensity < 0.0 {
            return Err(EICError::InvalidQuery(
                "minimum intensity must be a finite non-negative value".to_string(),
            ));
        }

        Ok(PreparedEICQuery {
            query: self.clone(),
            mz_min,
            mz_max,
            rt_min,
            rt_max,
            mobility_min,
            mobility_max,
            ms_level: self.ms_level,
            min_intensity,
        })
    }
}

/// Computed EIC results returned by the reader-native extraction API.
///
/// The output stays distinct from file-native chromatogram objects because it is
/// a derived view built from a query rather than a source artifact loaded from
/// disk.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedIonChromatogram {
    /// The query that produced this extracted trace.
    pub query: EICQuery,
    /// Retention times for the extracted trace.
    pub times: Vec<f64>,
    /// Summed intensities aligned with `times`.
    pub intensities: Vec<f32>,
}

impl ExtractedIonChromatogram {
    /// Create an empty chromatogram shell for a query.
    #[must_use]
    pub fn new(query: EICQuery) -> Self {
        Self {
            query,
            times: Vec::new(),
            intensities: Vec::new(),
        }
    }
}

/// Errors raised by the shared EIC extraction surface.
#[derive(Debug, Error)]
pub enum EICError {
    #[error("{0}")]
    InvalidQuery(String),
    #[error(transparent)]
    ArrayRetrieval(#[from] ArrayRetrievalError),
    #[cfg(feature = "bruker_tdf")]
    #[error(transparent)]
    TimsRust(#[from] timsrust::TimsRustError),
}

/// Units reported by EIC progress callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EICProgressUnit {
    Spectra,
    TdfEntries,
}

/// Incremental progress update for EIC extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EICProgress {
    pub processed: usize,
    pub total: usize,
    pub unit: EICProgressUnit,
}

/// Reader-native EIC extraction for `MZReader`-style types.
///
/// Callers keep the normal reader workflow and invoke `extract_eic` or
/// `extract_eics` on the reader itself, while backends decide how to satisfy the
/// shared query contract.
pub trait ExtractedIonChromatogramSource<
    C: CentroidLike = CentroidPeak,
    D: DeconvolutedCentroidLike = DeconvolutedPeak,
    S: SpectrumLike<C, D> = MultiLayerSpectrum<C, D>,
>: SpectrumSource<C, D, S>
{
    /// Extract a batch of chromatograms for the supplied queries.
    fn extract_eics(
        &mut self,
        queries: &[EICQuery],
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError>;

    /// Extract a batch of chromatograms while reporting incremental progress.
    fn extract_eics_with_progress(
        &mut self,
        queries: &[EICQuery],
        progress: &mut dyn FnMut(EICProgress),
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        let _ = progress;
        self.extract_eics(queries)
    }

    /// Extract a single chromatogram through the shared batch entry point.
    fn extract_eic(&mut self, query: &EICQuery) -> Result<ExtractedIonChromatogram, EICError> {
        self.extract_eics(std::slice::from_ref(query))
            .map(|mut eics| eics.remove(0))
    }

    /// Extract a single chromatogram while reporting incremental progress.
    fn extract_eic_with_progress(
        &mut self,
        query: &EICQuery,
        progress: &mut dyn FnMut(EICProgress),
    ) -> Result<ExtractedIonChromatogram, EICError> {
        self.extract_eics_with_progress(std::slice::from_ref(query), progress)
            .map(|mut eics| eics.remove(0))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedEICQuery {
    pub(crate) query: EICQuery,
    pub(crate) mz_min: f64,
    pub(crate) mz_max: f64,
    pub(crate) rt_min: Option<f64>,
    pub(crate) rt_max: Option<f64>,
    pub(crate) mobility_min: Option<f64>,
    pub(crate) mobility_max: Option<f64>,
    pub(crate) ms_level: Option<u8>,
    pub(crate) min_intensity: f32,
}

impl PreparedEICQuery {
    pub(crate) fn matches_spectrum<
        C: CentroidLike,
        D: DeconvolutedCentroidLike,
        S: SpectrumLike<C, D>,
    >(
        &self,
        spectrum: &S,
    ) -> bool {
        let rt = spectrum.start_time();
        if let Some(rt_min) = self.rt_min {
            if rt < rt_min {
                return false;
            }
        }
        if let Some(rt_max) = self.rt_max {
            if rt > rt_max {
                return false;
            }
        }
        if let Some(ms_level) = self.ms_level {
            if spectrum.ms_level() != ms_level {
                return false;
            }
        }
        if self.mobility_min.is_some() || self.mobility_max.is_some() {
            let Some(mobility) = spectrum.ion_mobility() else {
                return false;
            };
            if let Some(mobility_min) = self.mobility_min {
                if mobility < mobility_min {
                    return false;
                }
            }
            if let Some(mobility_max) = self.mobility_max {
                if mobility > mobility_max {
                    return false;
                }
            }
        }
        true
    }
}

pub(crate) fn prepare_queries(queries: &[EICQuery]) -> Result<Vec<PreparedEICQuery>, EICError> {
    queries.iter().map(EICQuery::prepare).collect()
}

pub(crate) fn initialize_results(queries: &[PreparedEICQuery]) -> Vec<ExtractedIonChromatogram> {
    queries
        .iter()
        .map(|query| ExtractedIonChromatogram::new(query.query.clone()))
        .collect()
}

pub(crate) fn extract_eics_from_spectra<
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
    S: SpectrumLike<C, D>,
    R: SpectrumSource<C, D, S> + ?Sized,
>(
    reader: &mut R,
    queries: &[EICQuery],
) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
    if queries.is_empty() {
        return Ok(Vec::new());
    }

    let prepared = prepare_queries(queries)?;
    let mut results = initialize_results(&prepared);
    extract_eics_from_prepared_queries(reader, &prepared, &mut results)?;
    Ok(results)
}

pub(crate) fn extract_eics_from_spectra_with_progress<
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
    S: SpectrumLike<C, D>,
    R: SpectrumSource<C, D, S> + ?Sized,
>(
    reader: &mut R,
    queries: &[EICQuery],
    progress: &mut dyn FnMut(EICProgress),
) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
    if queries.is_empty() {
        return Ok(Vec::new());
    }

    let prepared = prepare_queries(queries)?;
    let mut results = initialize_results(&prepared);
    extract_eics_from_prepared_queries_with_progress(
        reader,
        &prepared,
        &mut results,
        Some(progress),
    )?;
    Ok(results)
}

fn extract_eics_from_prepared_queries<
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
    S: SpectrumLike<C, D>,
    R: SpectrumSource<C, D, S> + ?Sized,
>(
    reader: &mut R,
    prepared: &[PreparedEICQuery],
    results: &mut [ExtractedIonChromatogram],
) -> Result<(), EICError> {
    extract_eics_from_prepared_queries_with_progress(reader, prepared, results, None)
}

fn extract_eics_from_prepared_queries_with_progress<
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
    S: SpectrumLike<C, D>,
    R: SpectrumSource<C, D, S> + ?Sized,
>(
    reader: &mut R,
    prepared: &[PreparedEICQuery],
    results: &mut [ExtractedIonChromatogram],
    mut progress: Option<&mut dyn FnMut(EICProgress)>,
) -> Result<(), EICError> {
    let original_detail_level = *reader.detail_level();
    reader.set_detail_level(DetailLevel::Lazy);
    let total = reader.len();
    let outcome = (|| -> Result<(), EICError> {
        for index in 0..total {
            if let Some(spectrum) = reader.get_spectrum_by_index(index) {
                process_spectrum(&spectrum, prepared, results)?;
            }
            if let Some(progress) = progress.as_deref_mut() {
                progress(EICProgress {
                    processed: index + 1,
                    total,
                    unit: EICProgressUnit::Spectra,
                });
            }
        }
        Ok(())
    })();
    reader.set_detail_level(original_detail_level);
    outcome
}

fn process_spectrum<C: CentroidLike, D: DeconvolutedCentroidLike, S: SpectrumLike<C, D>>(
    spectrum: &S,
    prepared: &[PreparedEICQuery],
    results: &mut [ExtractedIonChromatogram],
) -> Result<(), EICError> {
    let candidate_queries: Vec<usize> = prepared
        .iter()
        .enumerate()
        .filter_map(|(index, query)| query.matches_spectrum(spectrum).then_some(index))
        .collect();

    if candidate_queries.is_empty() {
        return Ok(());
    }

    let time = spectrum.start_time();
    if let Some(arrays) = spectrum.raw_arrays() {
        if arrays.has_array(&ArrayType::MZArray) && arrays.has_array(&ArrayType::IntensityArray) {
            let mzs = arrays.mzs()?;
            let intensities = arrays.intensities()?;
            for query_index in candidate_queries {
                let intensity = sum_array_range(&mzs, &intensities, &prepared[query_index]);
                results[query_index].times.push(time);
                results[query_index].intensities.push(intensity);
            }
            return Ok(());
        }
    }

    let peak_data = spectrum.peaks();
    let mut sums = vec![0.0f32; candidate_queries.len()];
    for point in peak_data.iter() {
        let mz = point.mz();
        let intensity = point.intensity();
        for (slot, query_index) in candidate_queries.iter().copied().enumerate() {
            let query = &prepared[query_index];
            if intensity >= query.min_intensity && mz >= query.mz_min && mz <= query.mz_max {
                sums[slot] += intensity;
            }
        }
    }

    for (slot, query_index) in candidate_queries.into_iter().enumerate() {
        results[query_index].times.push(time);
        results[query_index].intensities.push(sums[slot]);
    }
    Ok(())
}

fn sum_array_range(mzs: &[f64], intensities: &[f32], query: &PreparedEICQuery) -> f32 {
    if mzs.is_empty() || intensities.is_empty() {
        return 0.0;
    }

    let hi_bound = mzs.len().min(intensities.len());
    let mzs = &mzs[..hi_bound];
    let intensities = &intensities[..hi_bound];
    let (lower_index, upper_index) = ordered_array_bounds(mzs, query);

    intensities[lower_index..upper_index]
        .iter()
        .copied()
        .filter(|intensity| *intensity >= query.min_intensity)
        .sum()
}

fn ordered_array_bounds(mzs: &[f64], query: &PreparedEICQuery) -> (usize, usize) {
    (
        mzs.partition_point(|mz| *mz < query.mz_min),
        mzs.partition_point(|mz| *mz <= query.mz_max),
    )
}

fn normalize_bounds(a: f64, b: f64, label: &str) -> Result<(f64, f64), EICError> {
    if !a.is_finite() || !b.is_finite() {
        return Err(EICError::InvalidQuery(format!(
            "{label} bounds must be finite values"
        )));
    }
    if a <= b {
        Ok((a, b))
    } else {
        Ok((b, a))
    }
}

fn normalize_optional_bounds(
    a: Option<f64>,
    b: Option<f64>,
    label: &str,
) -> Result<(Option<f64>, Option<f64>), EICError> {
    match (a, b) {
        (Some(a), Some(b)) => normalize_bounds(a, b, label).map(|(lo, hi)| (Some(lo), Some(hi))),
        (Some(a), None) => {
            if !a.is_finite() {
                return Err(EICError::InvalidQuery(format!(
                    "{label} lower bound must be finite"
                )));
            }
            Ok((Some(a), None))
        }
        (None, Some(b)) => {
            if !b.is_finite() {
                return Err(EICError::InvalidQuery(format!(
                    "{label} upper bound must be finite"
                )));
            }
            Ok((None, Some(b)))
        }
        (None, None) => Ok((None, None)),
    }
}

#[cfg(feature = "mgf")]
impl<
        R: SeekRead,
        C: CentroidLike + From<CentroidPeak>,
        D: DeconvolutedCentroidLike + From<DeconvolutedPeak>,
    > ExtractedIonChromatogramSource<C, D, MultiLayerSpectrum<C, D>> for MGFReaderType<R, C, D>
{
    fn extract_eics(
        &mut self,
        queries: &[EICQuery],
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra(self, queries)
    }

    fn extract_eics_with_progress(
        &mut self,
        queries: &[EICQuery],
        progress: &mut dyn FnMut(EICProgress),
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra_with_progress(self, queries, progress)
    }
}

#[cfg(feature = "mzml")]
impl<
        R: SeekRead,
        C: CentroidLike + BuildFromArrayMap,
        D: DeconvolutedCentroidLike + BuildFromArrayMap,
    > ExtractedIonChromatogramSource<C, D, MultiLayerSpectrum<C, D>> for MzMLReaderType<R, C, D>
{
    fn extract_eics(
        &mut self,
        queries: &[EICQuery],
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra(self, queries)
    }

    fn extract_eics_with_progress(
        &mut self,
        queries: &[EICQuery],
        progress: &mut dyn FnMut(EICProgress),
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra_with_progress(self, queries, progress)
    }
}

#[cfg(feature = "mzmlb")]
impl<C: CentroidLike + BuildFromArrayMap, D: DeconvolutedCentroidLike + BuildFromArrayMap>
    ExtractedIonChromatogramSource<C, D, MultiLayerSpectrum<C, D>> for MzMLbReaderType<C, D>
{
    fn extract_eics(
        &mut self,
        queries: &[EICQuery],
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra(self, queries)
    }

    fn extract_eics_with_progress(
        &mut self,
        queries: &[EICQuery],
        progress: &mut dyn FnMut(EICProgress),
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra_with_progress(self, queries, progress)
    }
}

#[cfg(feature = "thermo")]
impl<
        C: CentroidLike + From<CentroidPeak> + BuildFromArrayMap,
        D: DeconvolutedCentroidLike + From<DeconvolutedPeak> + BuildFromArrayMap,
    > ExtractedIonChromatogramSource<C, D, MultiLayerSpectrum<C, D>> for ThermoRawReaderType<C, D>
{
    fn extract_eics(
        &mut self,
        queries: &[EICQuery],
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra(self, queries)
    }

    fn extract_eics_with_progress(
        &mut self,
        queries: &[EICQuery],
        progress: &mut dyn FnMut(EICProgress),
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra_with_progress(self, queries, progress)
    }
}

impl<C: CentroidLike, D: DeconvolutedCentroidLike, S: SpectrumLike<C, D> + Clone>
    ExtractedIonChromatogramSource<C, D, S> for MemorySpectrumSource<C, D, S>
{
    fn extract_eics(
        &mut self,
        queries: &[EICQuery],
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra(self, queries)
    }

    fn extract_eics_with_progress(
        &mut self,
        queries: &[EICQuery],
        progress: &mut dyn FnMut(EICProgress),
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError> {
        extract_eics_from_spectra_with_progress(self, queries, progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mzpeaks::peak_set::PeakSetVec;

    use crate::io::offset_index::OffsetIndex;
    use crate::params::{ControlledVocabulary, Param};
    use crate::spectrum::bindata::{
        ArrayType, BinaryArrayMap, BinaryCompressionType, BinaryDataArrayType, DataArray,
    };
    use crate::spectrum::scan_properties::{
        Acquisition, ScanPolarity, SignalContinuity, SpectrumDescription,
    };

    #[test]
    fn query_builder_preserves_the_full_phase_one_filter_set() {
        let query = EICQuery::new(501.5, 500.5)
            .with_rt_range(12.5, 10.0)
            .with_ms_level(2)
            .with_mobility_range(1.4, 1.1)
            .with_min_intensity(42.0);

        let prepared = query.prepare().expect("query should be valid");

        assert_eq!(prepared.query, query);
        assert_eq!(prepared.mz_min, 500.5);
        assert_eq!(prepared.mz_max, 501.5);
        assert_eq!(prepared.rt_min, Some(10.0));
        assert_eq!(prepared.rt_max, Some(12.5));
        assert_eq!(prepared.ms_level, Some(2));
        assert_eq!(prepared.mobility_min, Some(1.1));
        assert_eq!(prepared.mobility_max, Some(1.4));
        assert_eq!(prepared.min_intensity, 42.0_f32);
    }

    #[test]
    fn extracted_chromatogram_keeps_the_query_and_starts_empty() {
        let query = EICQuery::new(100.0, 101.0);
        let chromatogram = ExtractedIonChromatogram::new(query.clone());

        assert_eq!(chromatogram.query, query);
        assert!(chromatogram.times.is_empty());
        assert!(chromatogram.intensities.is_empty());
    }

    #[derive(Clone)]
    struct TrackingSpectrumSource {
        spectra: Vec<MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak>>,
        detail_level: DetailLevel,
        reads: usize,
        index: OffsetIndex,
    }

    impl TrackingSpectrumSource {
        fn new(spectra: Vec<MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak>>) -> Self {
            let mut index = OffsetIndex::new("spectrum".to_string());
            for (i, spectrum) in spectra.iter().enumerate() {
                index.insert(spectrum.id().to_string(), i as u64);
            }

            Self {
                spectra,
                detail_level: DetailLevel::Full,
                reads: 0,
                index,
            }
        }
    }

    impl Iterator for TrackingSpectrumSource {
        type Item = MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak>;

        fn next(&mut self) -> Option<Self::Item> {
            panic!("extract_eics_from_spectra should read by index, not by iterator")
        }
    }

    impl
        SpectrumSource<
            CentroidPeak,
            DeconvolutedPeak,
            MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak>,
        > for TrackingSpectrumSource
    {
        fn reset(&mut self) {
            self.reads = 0;
        }

        fn detail_level(&self) -> &DetailLevel {
            &self.detail_level
        }

        fn set_detail_level(&mut self, detail_level: DetailLevel) {
            self.detail_level = detail_level;
        }

        fn get_spectrum_by_id(
            &mut self,
            id: &str,
        ) -> Option<MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak>> {
            self.index
                .index_of(id)
                .and_then(|index| self.get_spectrum_by_index(index))
        }

        fn get_spectrum_by_index(
            &mut self,
            index: usize,
        ) -> Option<MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak>> {
            assert_eq!(
                self.detail_level,
                DetailLevel::Lazy,
                "portable EIC extraction should switch the reader to lazy loading"
            );
            self.reads += 1;
            self.spectra.get(index).cloned()
        }

        fn get_index(&self) -> &OffsetIndex {
            &self.index
        }

        fn set_index(&mut self, index: OffsetIndex) {
            self.index = index;
        }
    }

    fn make_description(id: &str) -> SpectrumDescription {
        let mut description = SpectrumDescription::default();
        description.id = id.to_string();
        description.signal_continuity = SignalContinuity::Centroid;
        description.polarity = ScanPolarity::Unknown;
        description.acquisition = Acquisition::default();
        description
    }

    fn make_timed_description(id: &str, start_time: f64, ms_level: u8) -> SpectrumDescription {
        let mut description = make_description(id);
        description.ms_level = ms_level;
        description.acquisition.first_scan_mut().unwrap().start_time = start_time;
        description
    }

    fn make_array_spectrum(
        id: &str,
        mzs: &[f64],
        intensities: &[f32],
    ) -> MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak> {
        assert_eq!(mzs.len(), intensities.len());

        let mut arrays = BinaryArrayMap::new();

        let mut mz_array = DataArray::from_name_type_size(
            &ArrayType::MZArray,
            BinaryDataArrayType::Float64,
            mzs.len() * std::mem::size_of::<f64>(),
        );
        mz_array.compression = BinaryCompressionType::Decoded;
        for value in mzs {
            mz_array.data.extend(value.to_le_bytes());
        }

        let mut intensity_array = DataArray::from_name_type_size(
            &ArrayType::IntensityArray,
            BinaryDataArrayType::Float32,
            intensities.len() * std::mem::size_of::<f32>(),
        );
        intensity_array.compression = BinaryCompressionType::Decoded;
        for value in intensities {
            intensity_array.data.extend(value.to_le_bytes());
        }

        arrays.add(mz_array);
        arrays.add(intensity_array);

        MultiLayerSpectrum::new(make_description(id), Some(arrays), None, None)
    }

    fn make_timed_array_spectrum(
        id: &str,
        start_time: f64,
        ms_level: u8,
        mzs: &[f64],
        intensities: &[f32],
    ) -> MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak> {
        let mut spectrum = make_array_spectrum(id, mzs, intensities);
        spectrum.description = make_timed_description(id, start_time, ms_level);
        spectrum
    }

    fn make_timed_mobility_array_spectrum(
        id: &str,
        start_time: f64,
        ms_level: u8,
        mobility: f64,
        mzs: &[f64],
        intensities: &[f32],
    ) -> MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak> {
        let mut spectrum = make_timed_array_spectrum(id, start_time, ms_level, mzs, intensities);
        let scan = spectrum.description.acquisition.first_scan_mut().unwrap();
        scan.params = Some(Box::new(vec![Param::builder()
            .name("inverse reduced ion mobility")
            .controlled_vocabulary(ControlledVocabulary::MS)
            .accession(1002815)
            .value(mobility)
            .build()]));
        spectrum
    }

    fn make_peak_spectrum(
        id: &str,
        peaks: Vec<CentroidPeak>,
    ) -> MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak> {
        MultiLayerSpectrum::new(
            make_description(id),
            None,
            Some(PeakSetVec::new(peaks)),
            None,
        )
    }

    fn make_timed_peak_spectrum(
        id: &str,
        start_time: f64,
        ms_level: u8,
        peaks: Vec<CentroidPeak>,
    ) -> MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak> {
        let mut spectrum = make_peak_spectrum(id, peaks);
        spectrum.description = make_timed_description(id, start_time, ms_level);
        spectrum
    }

    #[test]
    fn extract_eics_from_spectra_uses_lazy_index_reads_and_restores_detail_level() {
        let spectrum = make_peak_spectrum("scan=1", vec![CentroidPeak::new(101.0, 42.0, 0)]);
        let mut source = TrackingSpectrumSource::new(vec![spectrum]);
        let query = EICQuery::new(100.5, 101.5);

        let eics = extract_eics_from_spectra(&mut source, &[query]).expect("query should succeed");

        assert_eq!(source.detail_level, DetailLevel::Full);
        assert_eq!(source.reads, 1);
        assert_eq!(eics[0].intensities, vec![42.0]);
        assert_eq!(eics[0].times, vec![0.0]);
    }

    #[test]
    fn ordered_array_windows_sum_only_the_matching_interval() {
        let spectrum = make_array_spectrum(
            "scan=2",
            &[100.0, 101.0, 102.0, 103.0],
            &[5.0, 10.0, 30.0, 20.0],
        );
        let mut source = TrackingSpectrumSource::new(vec![spectrum]);
        let queries = vec![EICQuery::new(101.5, 102.5), EICQuery::new(100.1, 100.2)];

        let eics =
            extract_eics_from_spectra(&mut source, &queries).expect("queries should succeed");

        assert_eq!(source.detail_level, DetailLevel::Full);
        assert_eq!(source.reads, 1);
        assert_eq!(eics[0].times, vec![0.0]);
        assert_eq!(eics[0].intensities, vec![30.0]);
        assert_eq!(eics[1].times, vec![0.0]);
        assert_eq!(eics[1].intensities, vec![0.0]);
    }

    #[test]
    fn peak_fallback_keeps_summation_intact_without_raw_arrays() {
        let spectrum = make_peak_spectrum(
            "scan=3",
            vec![
                CentroidPeak::new(100.0, 5.0, 0),
                CentroidPeak::new(101.0, 10.0, 1),
                CentroidPeak::new(102.0, 30.0, 2),
                CentroidPeak::new(103.0, 20.0, 3),
            ],
        );
        let mut source = TrackingSpectrumSource::new(vec![spectrum]);
        let queries = vec![EICQuery::new(101.5, 102.5), EICQuery::new(100.1, 100.2)];

        let eics =
            extract_eics_from_spectra(&mut source, &queries).expect("queries should succeed");

        assert_eq!(source.detail_level, DetailLevel::Full);
        assert_eq!(source.reads, 1);
        assert_eq!(eics[0].times, vec![0.0]);
        assert_eq!(eics[0].intensities, vec![30.0]);
        assert_eq!(eics[1].times, vec![0.0]);
        assert_eq!(eics[1].intensities, vec![0.0]);
    }

    #[test]
    fn portable_eic_regression_keeps_zero_points_for_matching_spectra() {
        let spectra = vec![
            make_timed_array_spectrum("scan=4", 1.0, 1, &[101.8, 102.1, 103.0], &[12.0, 18.0, 7.0]),
            make_timed_peak_spectrum(
                "scan=5",
                2.0,
                1,
                vec![
                    CentroidPeak::new(100.0, 5.0, 0),
                    CentroidPeak::new(103.0, 9.0, 1),
                ],
            ),
            make_timed_array_spectrum("scan=6", 3.0, 2, &[101.9, 102.0], &[50.0, 60.0]),
        ];
        let mut source = TrackingSpectrumSource::new(spectra);
        let query = EICQuery::new(101.5, 102.5).with_ms_level(1);

        let eics = extract_eics_from_spectra(&mut source, &[query]).expect("query should succeed");
        let eic = &eics[0];

        assert_eq!(source.detail_level, DetailLevel::Full);
        assert_eq!(source.reads, 3);
        assert_eq!(eic.times, vec![1.0, 2.0]);
        assert_eq!(eic.intensities, vec![30.0, 0.0]);
    }

    #[test]
    fn portable_fallback_honors_one_sided_mobility_filters() {
        let spectra = vec![
            make_timed_mobility_array_spectrum("scan=7", 1.0, 1, 1.05, &[100.0], &[5.0]),
            make_timed_mobility_array_spectrum("scan=8", 2.0, 1, 1.35, &[100.0], &[7.0]),
        ];
        let mut source = TrackingSpectrumSource::new(spectra);
        let mut query = EICQuery::new(99.5, 100.5);
        query.mobility_min = Some(1.2);

        let eics = extract_eics_from_spectra(&mut source, &[query]).expect("query should succeed");
        let eic = &eics[0];

        assert_eq!(source.detail_level, DetailLevel::Full);
        assert_eq!(source.reads, 2);
        assert_eq!(eic.times, vec![2.0]);
        assert_eq!(eic.intensities, vec![7.0]);
    }
}
