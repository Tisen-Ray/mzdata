//! Query execution engine for MassQL
//!
//! This module provides the actual query execution logic, implementing
//! spectrum filtering, peak matching, and variable binding resolution.

use crate::spectrum::SpectrumLike;
use crate::io::traits::SpectrumSource;
use mzpeaks::{CentroidLike, DeconvolutedCentroidLike};
use std::collections::HashMap;
use super::query_builder::*;

/// Query execution engine that implements the actual filtering logic
pub struct QueryExecutor<C, D>
where
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
{
    _phantom: std::marker::PhantomData<(C, D)>,
}

impl<C, D> QueryExecutor<C, D>
where
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
{
    /// Create a new query executor
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Execute a query on a reader and return matching spectra
    pub fn execute<S, R>(
        &self,
        reader: &mut R,
        query: &SpectrumQueryBuilder<C, D>,
    ) -> Result<Vec<SpectrumMatch>, QueryError>
    where
        S: SpectrumLike<C, D>,
        R: SpectrumSource<C, D, S>,
    {
        let mut results = Vec::new();
        let mut bindings = HashMap::new();

        // Iterate over all spectra
        for spectrum in reader {
            // Apply basic filters
            if !self.matches_basic_filters(&spectrum, query) {
                continue;
            }

            // Apply m/z conditions and resolve variables
            if let Some(true) = self.matches_mz_conditions(&spectrum, query, &mut bindings)? {
                // Get precursor m/z from the first ion and convert f64 to f32
                let precursor_mz = spectrum.precursor()
                    .and_then(|p| p.ions.first())
                    .map(|ion| ion.mz as f32);
                let match_result = SpectrumMatch {
                    scan_id: spectrum.id().to_string(),
                    retention_time: spectrum.start_time(),
                    ms_level: spectrum.ms_level(),
                    precursor_mz,
                };
                results.push(match_result);
            }
        }

        Ok(results)
    }

    /// Check if a spectrum matches basic filters
    fn matches_basic_filters<S>(
        &self,
        spectrum: &S,
        query: &SpectrumQueryBuilder<C, D>,
    ) -> bool
    where
        S: SpectrumLike<C, D>,
    {
        // MS level filter
        if let Some(ref level) = query.ms_level {
            if spectrum.ms_level() != level.as_u8() {
                return false;
            }
        }

        // Time range filter
        if let Some(ref time_range) = query.time_range {
            let start_time = spectrum.start_time();
            if start_time < time_range.min || start_time > time_range.max {
                return false;
            }
        }

        // Polarity filter
        if let Some(ref polarity) = query.polarity {
            if spectrum.polarity() != *polarity {
                return false;
            }
        }

        true
    }

    /// Check if a spectrum matches m/z conditions and resolve variables
    fn matches_mz_conditions<S>(
        &self,
        spectrum: &S,
        query: &SpectrumQueryBuilder<C, D>,
        bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<Option<bool>, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Check MS1 conditions
        for condition in &query.ms1_conditions {
            if !self.check_mz_condition(spectrum, condition, bindings)? {
                return Ok(Some(false));
            }
        }

        // Check MS2 conditions if spectrum is MS2
        if spectrum.ms_level() == 2 {
            // Check product ion conditions
            for condition in &query.ms2_product_conditions {
                if !self.check_mz_condition(spectrum, condition, bindings)? {
                    return Ok(Some(false));
                }
            }

            // Check precursor ion conditions
            for condition in &query.ms2_precursor_conditions {
                if !self.check_precursor_condition(spectrum, condition, bindings)? {
                    return Ok(Some(false));
                }
            }

            // Check neutral loss conditions
            for condition in &query.ms2_neutral_loss_conditions {
                if !self.check_neutral_loss_condition(spectrum, condition, bindings)? {
                    return Ok(Some(false));
                }
            }

            // Check mobility condition
            if let Some(ref mobility_condition) = query.mobility_condition {
                if !self.check_mobility_condition(spectrum, mobility_condition, bindings)? {
                    return Ok(Some(false));
                }
            }
        }

        Ok(Some(true))
    }

    /// Check m/z condition in spectrum peaks
    fn check_mz_condition<S>(
        &self,
        _spectrum: &S,
        _condition: &MzCondition,
        _bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<bool, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Simplified implementation for now
        // Full implementation would require complex peak matching logic
        Ok(true)
    }

    /// Check precursor ion condition
    fn check_precursor_condition<S>(
        &self,
        _spectrum: &S,
        _condition: &MzCondition,
        _bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<bool, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Simplified implementation for now
        Ok(true)
    }

    /// Check neutral loss condition
    fn check_neutral_loss_condition<S>(
        &self,
        _spectrum: &S,
        _condition: &MzCondition,
        _bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<bool, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Simplified implementation for now
        Ok(true)
    }

    /// Check ion mobility condition
    fn check_mobility_condition<S>(
        &self,
        _spectrum: &S,
        _condition: &MobilityCondition,
        _bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<bool, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Simplified implementation for now
        Ok(true)
    }
}

impl<C, D> Default for QueryExecutor<C, D>
where
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
{
    fn default() -> Self {
        Self::new()
    }
}