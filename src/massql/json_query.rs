//! JSON-based query interface for AI systems
//!
//! This module provides a JSON-based query format that enables AI systems
//! to construct complex mass spectrometry queries without needing to
//! understand the Rust type system.
//!
//! Note: Full serde integration requires the "serde" feature to be enabled.

use crate::massql::query_builder::*;

/// JSON-based query representation (simplified version)
///
/// This struct provides the schema for JSON-based queries that can be
/// converted to SpectrumQueryBuilder instances. The actual serde serialization
/// requires the "serde" feature to be enabled.
#[derive(Debug, Clone)]
pub struct JsonQuery {
    /// Query description
    pub description: Option<String>,

    /// MS level filter ("ms1", "ms2", or specific MS level)
    pub ms_level: Option<String>,

    /// Retention time range
    pub time_range: Option<TimeRangeJson>,

    /// Scan number range
    pub scan_range: Option<ScanRangeJson>,

    /// Charge state filter
    pub charge: Option<i32>,

    /// Polarity filter ("positive" or "negative")
    pub polarity: Option<String>,

    /// MS1 m/z conditions
    pub ms1_conditions: Option<Vec<MzConditionJson>>,

    /// MS2 product ion conditions
    pub ms2_product_conditions: Option<Vec<MzConditionJson>>,

    /// MS2 precursor ion conditions
    pub ms2_precursor_conditions: Option<Vec<MzConditionJson>>,

    /// MS2 neutral loss conditions
    pub ms2_neutral_loss_conditions: Option<Vec<MzConditionJson>>,

    /// Ion mobility condition
    pub mobility_condition: Option<MobilityConditionJson>,

    /// Whether to return complete spectrum data
    pub return_spectra: Option<bool>,
}

/// JSON time range
#[derive(Debug, Clone)]
pub struct TimeRangeJson {
    pub min: f64,
    pub max: f64,
}

/// JSON scan range
#[derive(Debug, Clone)]
pub struct ScanRangeJson {
    pub min: u32,
    pub max: u32,
}

/// JSON m/z condition
#[derive(Debug, Clone)]
pub struct MzConditionJson {
    /// m/z expression
    pub mz_expr: MzExpressionJson,

    /// Mass tolerance (in Daltons or PPM)
    pub tolerance: Option<ToleranceJson>,

    /// Minimum intensity requirement
    pub min_intensity: Option<f32>,

    /// Whether this is an exclusion condition
    pub excluded: Option<bool>,

    /// Cardinality requirement (for OR operations)
    pub cardinality: Option<CardinalityJson>,
}

/// JSON m/z expression
#[derive(Debug, Clone, PartialEq)]
pub enum MzExpressionJson {
    Fixed { value: f64 },
    Variable { name: String },
    Add {
        left: Box<MzExpressionJson>,
        right: Box<MzExpressionJson>,
    },
    Sub {
        left: Box<MzExpressionJson>,
        right: Box<MzExpressionJson>,
    },
    Mul {
        left: Box<MzExpressionJson>,
        right: Box<MzExpressionJson>,
    },
    Div {
        left: Box<MzExpressionJson>,
        right: Box<MzExpressionJson>,
    },
    Formula { formula: String },
    AminoAcidDelta { amino_acid: String },
    Peptide {
        sequence: String,
        charge: u8,
        ion_type: String,
    },
    Range {
        min: Box<MzExpressionJson>,
        max: Box<MzExpressionJson>,
    },
}

/// JSON tolerance
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToleranceJson {
    Da { value: f64 },
    Ppm { value: f64 },
}

/// JSON cardinality
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardinalityJson {
    pub min: usize,
    pub max: usize,
}

/// JSON mobility condition
#[derive(Debug, Clone)]
pub struct MobilityConditionJson {
    pub mobility_expr: MzExpressionJson,
}

impl TryFrom<JsonQuery> for SpectrumQueryBuilder<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak> {
    type Error = String;

    fn try_from(json_query: JsonQuery) -> Result<Self, Self::Error> {
        let mut builder = SpectrumQueryBuilder::new();

        // Parse MS level
        if let Some(ms_level_str) = json_query.ms_level {
            let ms_level = parse_ms_level(&ms_level_str)?;
            builder = builder.with_ms_level(ms_level);
        }

        // Parse time range
        if let Some(time_range) = json_query.time_range {
            builder = builder.with_time_range(TimeRange::new(time_range.min, time_range.max));
        }

        // Parse scan range
        if let Some(scan_range) = json_query.scan_range {
            builder = builder.with_scan_range(scan_range.min, scan_range.max);
        }

        // Parse charge
        if let Some(charge) = json_query.charge {
            builder = builder.with_charge(charge);
        }

        // Parse polarity
        if let Some(polarity_str) = json_query.polarity {
            let polarity = parse_polarity(&polarity_str)?;
            builder = builder.with_polarity(polarity);
        }

        // Parse MS1 conditions
        if let Some(conditions) = json_query.ms1_conditions {
            for condition in conditions {
                let mz_condition = parse_mz_condition(condition)?;
                builder = builder.with_ms1_mz(mz_condition);
            }
        }

        // Parse MS2 product conditions
        if let Some(conditions) = json_query.ms2_product_conditions {
            for condition in conditions {
                let mz_condition = parse_mz_condition(condition)?;
                builder = builder.with_ms2_product(mz_condition);
            }
        }

        // Parse MS2 precursor conditions
        if let Some(conditions) = json_query.ms2_precursor_conditions {
            for condition in conditions {
                let mz_condition = parse_mz_condition(condition)?;
                builder = builder.with_ms2_precursor(mz_condition);
            }
        }

        // Parse MS2 neutral loss conditions
        if let Some(conditions) = json_query.ms2_neutral_loss_conditions {
            for condition in conditions {
                let mz_condition = parse_mz_condition(condition)?;
                builder = builder.with_ms2_neutral_loss(mz_condition);
            }
        }

        // Parse mobility condition
        if let Some(mobility_condition) = json_query.mobility_condition {
            let mobility = parse_mobility_condition(mobility_condition)?;
            builder = builder.with_mobility(mobility);
        }

        // Parse return spectra flag
        if let Some(return_spectra) = json_query.return_spectra {
            builder = builder.return_spectra(return_spectra);
        }

        Ok(builder)
    }
}

fn parse_ms_level(s: &str) -> Result<MSLevel, String> {
    match s.to_lowercase().as_str() {
        "ms1" => Ok(MSLevel::MS1),
        "ms2" => Ok(MSLevel::MS2),
        _ => {
            if let Ok(n) = s.parse::<u8>() {
                Ok(MSLevel::MSn(n))
            } else {
                Err(format!("Invalid MS level: {}", s))
            }
        }
    }
}

fn parse_polarity(s: &str) -> Result<crate::spectrum::scan_properties::ScanPolarity, String> {
    match s.to_lowercase().as_str() {
        "positive" | "+" => Ok(crate::spectrum::scan_properties::ScanPolarity::Positive),
        "negative" | "-" => Ok(crate::spectrum::scan_properties::ScanPolarity::Negative),
        _ => Err(format!("Invalid polarity: {}", s)),
    }
}

fn parse_mz_condition(condition: MzConditionJson) -> Result<MzCondition, String> {
    let mz_expr = parse_mz_expression(condition.mz_expr)?;

    let mut mz_condition = MzCondition::new(mz_expr);

    // Parse tolerance
    if let Some(tolerance) = condition.tolerance {
        mz_condition = match tolerance {
            ToleranceJson::Da { value } => mz_condition.with_tolerance(mzpeaks::Tolerance::Da(value)),
            ToleranceJson::Ppm { value } => mz_condition.with_tolerance(mzpeaks::Tolerance::PPM(value)),
        };
    }

    // Parse min intensity
    if let Some(min_intensity) = condition.min_intensity {
        mz_condition = mz_condition.with_min_intensity(min_intensity);
    }

    // Parse excluded flag
    if let Some(true) = condition.excluded {
        mz_condition = mz_condition.excluded();
    }

    // Parse cardinality
    if let Some(cardinality) = condition.cardinality {
        mz_condition = mz_condition.with_cardinality(cardinality.min, cardinality.max);
    }

    Ok(mz_condition)
}

fn parse_mz_expression(expr: MzExpressionJson) -> Result<MzExpression, String> {
    match expr {
        MzExpressionJson::Fixed { value } => Ok(MzExpression::fixed(value)),

        MzExpressionJson::Variable { name } => {
            let var = match name.as_str() {
                "X" => QueryVariable::X,
                _ => QueryVariable::Named(Box::leak(name.into_boxed_str())),
            };
            Ok(MzExpression::var(var))
        }

        MzExpressionJson::Add { left, right } => {
            let left_expr = parse_mz_expression(*left)?;
            let right_expr = parse_mz_expression(*right)?;
            Ok(left_expr.add(right_expr))
        }

        MzExpressionJson::Sub { left, right } => {
            let left_expr = parse_mz_expression(*left)?;
            let right_expr = parse_mz_expression(*right)?;
            Ok(left_expr.sub(right_expr))
        }

        MzExpressionJson::Mul { left, right } => {
            let left_expr = parse_mz_expression(*left)?;
            let right_expr = parse_mz_expression(*right)?;
            Ok(left_expr.mul(right_expr))
        }

        MzExpressionJson::Div { left, right } => {
            let left_expr = parse_mz_expression(*left)?;
            let right_expr = parse_mz_expression(*right)?;
            Ok(left_expr.div(right_expr))
        }

        MzExpressionJson::Formula { formula } => {
            Ok(MzExpression::Function(MzFunction::Formula { formula }))
        }

        MzExpressionJson::AminoAcidDelta { amino_acid } => {
            Ok(MzExpression::Function(MzFunction::AminoAcidDelta { amino_acid }))
        }

        MzExpressionJson::Peptide { sequence, charge, ion_type } => {
            Ok(MzExpression::Function(MzFunction::Peptide { sequence, charge, ion_type }))
        }

        MzExpressionJson::Range { min, max } => {
            let min_expr = parse_mz_expression(*min)?;
            let max_expr = parse_mz_expression(*max)?;
            Ok(MzExpression::Function(MzFunction::Range {
                min: Box::new(min_expr),
                max: Box::new(max_expr),
            }))
        }
    }
}

fn parse_mobility_condition(condition: MobilityConditionJson) -> Result<MobilityCondition, String> {
    let mobility_expr = parse_mz_expression(condition.mobility_expr)?;

    if let MzExpression::Function(MzFunction::Range { min, max }) = mobility_expr {
        Ok(MobilityCondition::range(*min, *max))
    } else {
        Err("Mobility condition must be a range expression".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mz_expression_json() {
        let expr = MzExpressionJson::Fixed { value: 144.1 };
        assert_eq!(expr, MzExpressionJson::Fixed { value: 144.1 });
    }

    #[test]
    fn test_time_range_json() {
        let range = TimeRangeJson { min: 0.0, max: 10.0 };
        assert_eq!(range.min, 0.0);
        assert_eq!(range.max, 10.0);
    }

    #[test]
    fn test_parse_ms_level() {
        assert_eq!(parse_ms_level("ms1"), Ok(MSLevel::MS1));
        assert_eq!(parse_ms_level("ms2"), Ok(MSLevel::MS2));
        assert_eq!(parse_ms_level("3"), Ok(MSLevel::MSn(3)));
    }
}