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

#[derive(Debug, Clone, PartialEq)]
pub struct EICQuery {
    pub mz_min: f64,
    pub mz_max: f64,
    pub rt_min: Option<f64>,
    pub rt_max: Option<f64>,
    pub ms_level: Option<u8>,
    pub mobility_min: Option<f64>,
    pub mobility_max: Option<f64>,
    pub min_intensity: Option<f32>,
}

impl EICQuery {
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

    pub fn with_rt_range(mut self, rt_min: f64, rt_max: f64) -> Self {
        self.rt_min = Some(rt_min);
        self.rt_max = Some(rt_max);
        self
    }

    pub fn with_ms_level(mut self, ms_level: u8) -> Self {
        self.ms_level = Some(ms_level);
        self
    }

    pub fn with_mobility_range(mut self, mobility_min: f64, mobility_max: f64) -> Self {
        self.mobility_min = Some(mobility_min);
        self.mobility_max = Some(mobility_max);
        self
    }

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

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedIonChromatogram {
    pub query: EICQuery,
    pub times: Vec<f64>,
    pub intensities: Vec<f32>,
}

impl ExtractedIonChromatogram {
    pub fn new(query: EICQuery) -> Self {
        Self {
            query,
            times: Vec::new(),
            intensities: Vec::new(),
        }
    }
}

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

pub trait ExtractedIonChromatogramSource<
    C: CentroidLike = CentroidPeak,
    D: DeconvolutedCentroidLike = DeconvolutedPeak,
    S: SpectrumLike<C, D> = MultiLayerSpectrum<C, D>,
>: SpectrumSource<C, D, S>
{
    fn extract_eics(
        &mut self,
        queries: &[EICQuery],
    ) -> Result<Vec<ExtractedIonChromatogram>, EICError>;

    fn extract_eic(&mut self, query: &EICQuery) -> Result<ExtractedIonChromatogram, EICError> {
        self.extract_eics(std::slice::from_ref(query))
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

    let original_detail_level = *reader.detail_level();
    reader.set_detail_level(DetailLevel::Lazy);
    let outcome = (|| -> Result<(), EICError> {
        for index in 0..reader.len() {
            if let Some(spectrum) = reader.get_spectrum_by_index(index) {
                process_spectrum(&spectrum, &prepared, &mut results)?;
            }
        }
        Ok(())
    })();
    reader.set_detail_level(original_detail_level);
    outcome?;
    Ok(results)
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

    let lower_index = mzs.partition_point(|mz| *mz < query.mz_min);
    let upper_index = mzs.partition_point(|mz| *mz <= query.mz_max);
    intensities[lower_index..upper_index]
        .iter()
        .copied()
        .filter(|intensity| *intensity >= query.min_intensity)
        .sum()
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
}
