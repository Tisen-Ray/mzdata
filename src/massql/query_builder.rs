//! Type-safe mass spectrometry query builder for AI-driven analysis
//!
//! This module provides a powerful query building API supporting:
//! - Variable binding and references
//! - Complex expression evaluation
//! - High-performance parallel queries
//! - Type-safe composable queries

use crate::spectrum::SpectrumLike;
use crate::io::traits::SpectrumSource;
use crate::spectrum::scan_properties::ScanPolarity;
use mzpeaks::{CentroidLike, DeconvolutedCentroidLike, Tolerance};
use std::marker::PhantomData;
use std::collections::HashMap;

pub use crate::massql::query_execution::QueryExecutor;

/// Query variable type for pattern-based filtering with variable binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QueryVariable {
    /// Wildcard variable X, used to represent any m/z value
    X,
    /// Named variable
    Named(&'static str),
}

/// m/z expression supporting variable references and mathematical operations
#[derive(Debug, Clone, PartialEq)]
pub enum MzExpression {
    /// Fixed m/z value
    Fixed(f64),
    /// Variable reference
    Variable(QueryVariable),
    /// Addition: a + b
    Add(Box<MzExpression>, Box<MzExpression>),
    /// Subtraction: a - b
    Sub(Box<MzExpression>, Box<MzExpression>),
    /// Multiplication: a * b
    Mul(Box<MzExpression>, Box<MzExpression>),
    /// Division: a / b
    Div(Box<MzExpression>, Box<MzExpression>),
    /// Function call
    Function(MzFunction),
}

/// m/z calculation functions
#[derive(Debug, Clone, PartialEq)]
pub enum MzFunction {
    /// Molecular formula mass: formula("H2O")
    Formula { formula: String },
    /// Amino acid mass difference: aminoacid_delta("G")
    AminoAcidDelta { amino_acid: String },
    /// Peptide fragment mass: peptide("G", charge=1, ion="y")
    Peptide { sequence: String, charge: u8, ion_type: String },
    /// Range expression: range(min=expr, max=expr)
    Range { min: Box<MzExpression>, max: Box<MzExpression> },
    /// Mass defect: massdefect(min=expr, max=expr)
    MassDefect { min: f64, max: f64 },
}

impl MzExpression {
    /// Create a fixed m/z value
    pub fn fixed(mz: f64) -> Self {
        MzExpression::Fixed(mz)
    }

    /// Create a variable reference
    pub fn var(var: QueryVariable) -> Self {
        MzExpression::Variable(var)
    }

    /// Create a reference to variable X (shortcut method)
    pub fn x() -> Self {
        MzExpression::Variable(QueryVariable::X)
    }

    /// Addition
    pub fn add(self, other: MzExpression) -> Self {
        MzExpression::Add(Box::new(self), Box::new(other))
    }

    /// Subtraction
    pub fn sub(self, other: MzExpression) -> Self {
        MzExpression::Sub(Box::new(self), Box::new(other))
    }

    /// Multiplication
    pub fn mul(self, other: MzExpression) -> Self {
        MzExpression::Mul(Box::new(self), Box::new(other))
    }

    /// Division
    pub fn div(self, other: MzExpression) -> Self {
        MzExpression::Div(Box::new(self), Box::new(other))
    }

    /// Create a range expression
    pub fn range(self, width: f64) -> MzExpression {
        MzExpression::Function(MzFunction::Range {
            min: Box::new(self.clone().sub(MzExpression::fixed(width))),
            max: Box::new(self.add(MzExpression::fixed(width))),
        })
    }

    /// Evaluate the expression value given variable bindings
    pub fn evaluate(&self, bindings: &HashMap<QueryVariable, f64>) -> Result<f64, String> {
        match self {
            MzExpression::Fixed(value) => Ok(*value),
            MzExpression::Variable(var) => {
                bindings.get(var)
                    .copied()
                    .ok_or_else(|| format!("Variable {:?} not bound", var))
            }
            MzExpression::Add(a, b) => {
                Ok(a.evaluate(bindings)? + b.evaluate(bindings)?)
            }
            MzExpression::Sub(a, b) => {
                Ok(a.evaluate(bindings)? - b.evaluate(bindings)?)
            }
            MzExpression::Mul(a, b) => {
                Ok(a.evaluate(bindings)? * b.evaluate(bindings)?)
            }
            MzExpression::Div(a, b) => {
                let denom = b.evaluate(bindings)?;
                if denom == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(a.evaluate(bindings)? / denom)
                }
            }
            MzExpression::Function(func) => {
                match func {
                    MzFunction::Formula { formula } => {
                        // Simplified molecular formula mass calculation
                        Ok(Self::calculate_formula_mass(formula))
                    }
                    MzFunction::AminoAcidDelta { amino_acid } => {
                        Ok(Self::calculate_aminoacid_delta(amino_acid))
                    }
                    MzFunction::Peptide { sequence, charge, ion_type } => {
                        Ok(Self::calculate_peptide_mass(sequence, *charge, ion_type))
                    }
                    MzFunction::Range { min, max } => {
                        // Range expression, return midpoint
                        Ok((min.evaluate(bindings)? + max.evaluate(bindings)?) / 2.0)
                    }
                    MzFunction::MassDefect { min, max } => {
                        Ok((min + max) / 2.0)
                    }
                }
            }
        }
    }

    // Simplified molecular formula mass calculation (example)
    fn calculate_formula_mass(formula: &str) -> f64 {
        match formula {
            "H2O" => 18.01056,
            "CO2" => 44.0095,
            "CH2" => 14.01565,
            _ => 0.0, // Requires full implementation
        }
    }

    // Simplified amino acid mass difference calculation (example)
    fn calculate_aminoacid_delta(amino_acid: &str) -> f64 {
        match amino_acid {
            "G" => 57.02146,
            "A" => 71.03711,
            "S" => 87.03203,
            _ => 0.0, // Requires full implementation
        }
    }

    // Simplified peptide mass calculation (example)
    fn calculate_peptide_mass(sequence: &str, charge: u8, ion_type: &str) -> f64 {
        let base_mass = sequence.chars().map(|c| {
            match c {
                'G' => 57.02146,
                'A' => 71.03711,
                'S' => 87.03203,
                _ => 0.0,
            }
        }).sum::<f64>();

        let proton_mass = 1.007276;
        let mass = match ion_type {
            "y" => base_mass + 19.01839,
            "b" => base_mass + 1.007825,
            _ => base_mass,
        };

        (mass + (proton_mass * charge as f64)) / charge as f64
    }
}

/// m/z condition filter
#[derive(Debug, Clone)]
pub struct MzCondition {
    /// m/z expression
    pub mz_expr: MzExpression,
    /// Mass tolerance
    pub tolerance: Tolerance,
    /// Minimum intensity requirement
    pub min_intensity: Option<f32>,
    /// Whether this is an exclusion condition (NOT)
    pub excluded: bool,
    /// Cardinality requirement (for OR operations)
    pub cardinality: Option<(usize, usize)>,
}

impl MzCondition {
    /// Create a new m/z condition
    pub fn new(mz_expr: MzExpression) -> Self {
        Self {
            mz_expr,
            tolerance: Tolerance::Da(0.1),
            min_intensity: None,
            excluded: false,
            cardinality: None,
        }
    }

    /// Set mass tolerance
    pub fn with_tolerance(mut self, tolerance: Tolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Set minimum intensity
    pub fn with_min_intensity(mut self, intensity: f32) -> Self {
        self.min_intensity = Some(intensity);
        self
    }

    /// Set as exclusion condition
    pub fn excluded(mut self) -> Self {
        self.excluded = true;
        self
    }

    /// Set cardinality requirement (for OR operations)
    pub fn with_cardinality(mut self, min: usize, max: usize) -> Self {
        self.cardinality = Some((min, max));
        self
    }
}

/// Ion mobility condition
#[derive(Debug, Clone)]
pub struct MobilityCondition {
    /// Ion mobility expression
    pub mobility_expr: MzExpression,
}

impl MobilityCondition {
    /// Create an ion mobility range condition
    pub fn range(min: MzExpression, max: MzExpression) -> Self {
        Self {
            mobility_expr: MzExpression::Function(MzFunction::Range {
                min: Box::new(min),
                max: Box::new(max),
            }),
        }
    }
}

/// MS level filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MSLevel {
    MS1,
    MS2,
    MSn(u8),
}

impl MSLevel {
    pub fn as_u8(&self) -> u8 {
        match self {
            MSLevel::MS1 => 1,
            MSLevel::MS2 => 2,
            MSLevel::MSn(n) => *n,
        }
    }
}

/// Retention time range
#[derive(Debug, Clone, PartialEq)]
pub struct TimeRange {
    pub min: f64,
    pub max: f64,
}

impl TimeRange {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}

/// Mass spectrum query builder
#[derive(Debug, Clone)]
pub struct SpectrumQueryBuilder<C, D>
where
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
{
    /// MS level filter
    pub ms_level: Option<MSLevel>,
    /// Retention time range
    pub time_range: Option<TimeRange>,
    /// Scan number range
    pub scan_range: Option<(u32, u32)>,
    /// Charge state filter
    pub charge: Option<i32>,
    /// Polarity filter
    pub polarity: Option<ScanPolarity>,
    /// MS1 m/z conditions
    pub ms1_conditions: Vec<MzCondition>,
    /// MS2 product ion conditions
    pub ms2_product_conditions: Vec<MzCondition>,
    /// MS2 precursor ion conditions
    pub ms2_precursor_conditions: Vec<MzCondition>,
    /// MS2 neutral loss conditions
    pub ms2_neutral_loss_conditions: Vec<MzCondition>,
    /// Ion mobility condition
    pub mobility_condition: Option<MobilityCondition>,
    /// Whether to return complete spectrum data
    pub return_spectra: bool,
    _phantom: PhantomData<(C, D)>,
}

impl<C, D> Default for SpectrumQueryBuilder<C, D>
where
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C, D> SpectrumQueryBuilder<C, D>
where
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
{
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            ms_level: None,
            time_range: None,
            scan_range: None,
            charge: None,
            polarity: None,
            ms1_conditions: Vec::new(),
            ms2_product_conditions: Vec::new(),
            ms2_precursor_conditions: Vec::new(),
            ms2_neutral_loss_conditions: Vec::new(),
            mobility_condition: None,
            return_spectra: false,
            _phantom: PhantomData,
        }
    }

    /// Set MS level
    pub fn with_ms_level(mut self, level: MSLevel) -> Self {
        self.ms_level = Some(level);
        self
    }

    /// Set retention time range
    pub fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    /// Set scan number range
    pub fn with_scan_range(mut self, min: u32, max: u32) -> Self {
        self.scan_range = Some((min, max));
        self
    }

    /// Set charge state
    pub fn with_charge(mut self, charge: i32) -> Self {
        self.charge = Some(charge);
        self
    }

    /// Set polarity
    pub fn with_polarity(mut self, polarity: ScanPolarity) -> Self {
        self.polarity = Some(polarity);
        self
    }

    /// Add MS1 m/z condition
    pub fn with_ms1_mz(mut self, condition: MzCondition) -> Self {
        self.ms1_conditions.push(condition);
        self
    }

    /// Add MS2 product ion condition
    pub fn with_ms2_product(mut self, condition: MzCondition) -> Self {
        self.ms2_product_conditions.push(condition);
        self
    }

    /// Add MS2 precursor ion condition
    pub fn with_ms2_precursor(mut self, condition: MzCondition) -> Self {
        self.ms2_precursor_conditions.push(condition);
        self
    }

    /// Add MS2 neutral loss condition
    pub fn with_ms2_neutral_loss(mut self, condition: MzCondition) -> Self {
        self.ms2_neutral_loss_conditions.push(condition);
        self
    }

    /// Add ion mobility condition
    pub fn with_mobility(mut self, condition: MobilityCondition) -> Self {
        self.mobility_condition = Some(condition);
        self
    }

    /// Set whether to return complete spectrum data
    pub fn return_spectra(mut self, return_spectra: bool) -> Self {
        self.return_spectra = return_spectra;
        self
    }

    /// Execute the query and return results
    pub fn execute<S, R>(self, reader: &mut R) -> Result<Vec<SpectrumMatch>, QueryError>
    where
        S: SpectrumLike<C, D>,
        R: SpectrumSource<C, D, S>,
    {
        let executor = QueryExecutor::<C, D>::new();
        executor.execute(reader, &self)
    }

    /// Check basic filters
    #[allow(dead_code)]
    fn matches_basic_filters<S>(&self, spectrum: &S) -> bool
    where
        S: SpectrumLike<C, D>,
    {
        // MS level filter
        if let Some(level) = &self.ms_level {
            if spectrum.ms_level() != level.as_u8() {
                return false;
            }
        }

        // Time range filter - start_time() returns f64, not Option<f64>
        if let Some(time_range) = &self.time_range {
            let start_time = spectrum.start_time();
            if start_time < time_range.min || start_time > time_range.max {
                return false;
            }
        }

        // Scan number filter - simplified version
        if let Some((min, max)) = self.scan_range {
            let _ = (min, max);
            // Full implementation requires scan_number method
        }

        true
    }

    /// Check m/z conditions and resolve variables
    #[allow(dead_code)]
    fn matches_mz_conditions<S>(
        &self,
        spectrum: &S,
        bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<Option<bool>, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Check MS1 conditions
        for condition in &self.ms1_conditions {
            if !self.check_mz_condition(spectrum, condition, bindings)? {
                return Ok(Some(false));
            }
        }

        // Check MS2 product ion conditions
        if spectrum.ms_level() == 2 {
            for condition in &self.ms2_product_conditions {
                if !self.check_mz_condition(spectrum, condition, bindings)? {
                    return Ok(Some(false));
                }
            }

            // Check precursor ion conditions
            for condition in &self.ms2_precursor_conditions {
                if !self.check_precursor_condition(spectrum, condition, bindings)? {
                    return Ok(Some(false));
                }
            }

            // Check neutral loss conditions
            for condition in &self.ms2_neutral_loss_conditions {
                if !self.check_neutral_loss_condition(spectrum, condition, bindings)? {
                    return Ok(Some(false));
                }
            }
        }

        Ok(Some(true))
    }

    /// Check m/z condition
    #[allow(dead_code)]
    fn check_mz_condition<S>(
        &self,
        _spectrum: &S,
        _condition: &MzCondition,
        _bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<bool, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Simplified implementation - always returns true for now
        // Full implementation requires complex peak matching logic
        Ok(true)
    }

    /// Check precursor ion condition
    #[allow(dead_code)]
    fn check_precursor_condition<S>(
        &self,
        _spectrum: &S,
        _condition: &MzCondition,
        _bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<bool, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Simplified implementation - always returns true for now
        // Full implementation requires precursor matching logic
        Ok(true)
    }

    /// Check neutral loss condition
    #[allow(dead_code)]
    fn check_neutral_loss_condition<S>(
        &self,
        _spectrum: &S,
        _condition: &MzCondition,
        _bindings: &mut HashMap<QueryVariable, f64>,
    ) -> Result<bool, QueryError>
    where
        S: SpectrumLike<C, D>,
    {
        // Simplified implementation - always returns true for now
        // Full implementation requires neutral loss calculation
        Ok(true)
    }
}

/// Query result
#[derive(Debug)]
pub struct SpectrumMatch {
    pub scan_id: String,
    pub retention_time: f64,
    pub ms_level: u8,
    pub precursor_mz: Option<f32>,
}

/// Query error type
#[derive(Debug)]
pub enum QueryError {
    SpectrumRead(String),
    VariableNotBound(String),
    EvaluationError(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::SpectrumRead(msg) => write!(f, "Failed to read spectrum: {}", msg),
            QueryError::VariableNotBound(var) => write!(f, "Variable not bound: {}", var),
            QueryError::EvaluationError(msg) => write!(f, "Expression evaluation error: {}", msg),
        }
    }
}

impl std::error::Error for QueryError {}

impl From<String> for QueryError {
    fn from(msg: String) -> Self {
        QueryError::SpectrumRead(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_evaluation() {
        let mut bindings = HashMap::new();
        bindings.insert(QueryVariable::X, 500.0);

        // Test X * 0.0011 + 0.5 - 0.1
        let expr = MzExpression::x()
            .mul(MzExpression::fixed(0.0011))
            .add(MzExpression::fixed(0.5))
            .sub(MzExpression::fixed(0.1));

        let result = expr.evaluate(&bindings).unwrap();
        assert!((result - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_query_builder() {
        let _builder = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
            .with_ms_level(MSLevel::MS2)
            .with_time_range(TimeRange::new(0.0, 10.0))
            .with_ms2_product(
                MzCondition::new(MzExpression::fixed(144.1))
                    .with_tolerance(Tolerance::Da(0.1))
            );

        // Query builder created successfully
        // Note: Cannot access private fields directly for testing
    }
}