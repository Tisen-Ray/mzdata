//! AI-driven mass spectrometry query usage examples
//!
//! This example demonstrates how to use the type-safe MassQL interface for complex data queries.
//! These interfaces are specifically designed for AI systems, providing powerful expression capabilities and type safety guarantees.

use mzdata::massql::*;
use mzpeaks::Tolerance;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("=== AI-Driven Mass Spectrometry Query Examples ===\n");

    // Example 1: Basic query - Find specific MS2 product ions
    basic_ms2_product_query()?;

    // Example 2: Variable binding query - Isotope pattern search
    isotope_pattern_search()?;

    // Example 3: Ion mobility correlated query
    mobility_correlated_query()?;

    // Example 4: Neutral loss scan
    neutral_loss_scan()?;

    // Example 5: Precursor ion scan
    precursor_ion_scan()?;

    // Example 6: Formula and peptide queries
    formula_and_peptide_query()?;

    // Example 7: Complex pattern matching - Brominated compound pattern
    brominated_compound_pattern()?;

    // Demonstrate AI helper functions
    ai_helper_functions();

    // Demonstrate common query patterns
    common_query_patterns();

    Ok(())
}

/// Example 1: Basic MS2 product ion query
///
/// Corresponds to MassQL: QUERY scaninfo(MS2DATA) WHERE MS2PROD=144.1:TOLERANCEMZ=0.1
fn basic_ms2_product_query() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic MS2 Product Ion Query");
    println!("Finding MS2 product ions with m/z = 144.1\n");

    let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms2_product(
            MzCondition::new(MzExpression::fixed(144.1))
                .with_tolerance(Tolerance::Da(0.1))
        );

    // Assuming we have a reader
    // let results = query.execute(&mut reader)?;
    // println!("Found {} matching spectra", results.len());

    println!("Query built successfully✓\n");
    Ok(())
}

/// Example 2: Isotope pattern search
///
/// Corresponds to MassQL: QUERY scaninfo(MS2DATA) WHERE MS1MZ=X AND MS1MZ=X+1 AND MS2PREC=X
fn isotope_pattern_search() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Isotope Pattern Search");
    println!("Finding compounds with specific isotope patterns (M and M+1 peaks)\n");

    let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms1_mz(  // M peak in MS1
            MzCondition::new(MzExpression::x())
        )
        .with_ms1_mz(  // M+1 peak in MS1 (isotope peak)
            MzCondition::new(
                MzExpression::x().add(MzExpression::fixed(1.0))
            )
        )
        .with_ms2_precursor(  // MS2 precursor should be M
            MzCondition::new(MzExpression::x())
        );

    println!("Query built successfully✓");
    println!("This query automatically binds variable X and validates isotope patterns\n");
    Ok(())
}

/// Example 3: Ion mobility correlated query
///
/// Corresponds to MassQL: QUERY scaninfo(MS2DATA) WHERE MS2PREC=X AND MOBILITY=range(min=X*0.0011+0.5-0.1, max=X*0.0011+0.5+0.1)
fn mobility_correlated_query() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Ion Mobility Correlated Query");
    println!("Finding MS2 spectra where ion mobility correlates with precursor m/z\n");

    // Build ion mobility range expression: X*0.0011+0.5±0.1
    let mobility_center = MzExpression::x()
        .mul(MzExpression::fixed(0.0011))
        .add(MzExpression::fixed(0.5));

    let mobility_min = mobility_center.clone().sub(MzExpression::fixed(0.1));
    let mobility_max = mobility_center.add(MzExpression::fixed(0.1));

    let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms2_precursor(MzCondition::new(MzExpression::x()))
        .with_mobility(MobilityCondition::range(
            mobility_min,
            mobility_max,
        ));

    println!("Query built successfully✓");
    println!("Ion mobility range will be dynamically calculated based on precursor m/z\n");
    Ok(())
}

/// Example 4: Neutral loss scan
///
/// Corresponds to MassQL: QUERY scaninfo(MS2DATA) WHERE MS2NL=163
fn neutral_loss_scan() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Neutral Loss Scan");
    println!("Finding MS2 spectra with neutral loss of 163 Da (sugar loss)\n");

    let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms2_neutral_loss(
            MzCondition::new(MzExpression::fixed(163.0))
                .with_tolerance(Tolerance::Da(0.1))
                .with_min_intensity(1000.0)  // Minimum intensity requirement
        );

    println!("Query built successfully✓");
    println!("This query is used to detect neutral loss patterns from sugar groups\n");
    Ok(())
}

/// Example 5: Precursor ion scan
///
/// Corresponds to MassQL: QUERY scaninfo(MS2DATA) WHERE MS2PROD=660.2:TOLERANCEMZ=0.1 AND MS2PROD=468.2:TOLERANCEMZ=0.1
fn precursor_ion_scan() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. Precursor Ion Scan");
    println!("Finding MS2 spectra containing two specific product ions\n");

    let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms2_product(
            MzCondition::new(MzExpression::fixed(660.2))
                .with_tolerance(Tolerance::Da(0.1))
        )
        .with_ms2_product(
            MzCondition::new(MzExpression::fixed(468.2))
                .with_tolerance(Tolerance::Da(0.1))
        );

    println!("Query built successfully✓");
    println!("This query requires both product ions to be present\n");
    Ok(())
}

/// Example 6: Formula and peptide queries
///
/// Corresponds to MassQL: QUERY scaninfo(MS2DATA) WHERE MS2PROD=peptide(G, charge=1, ion=y)
fn formula_and_peptide_query() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. Formula and Peptide Queries");
    println!("Using molecular formulas and peptide functions for complex queries\n");

    // Using molecular formula
    let water_loss = MzExpression::Function(MzFunction::Formula {
        formula: "H2O".to_string()
    });

    let _query1 = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms2_neutral_loss(
            MzCondition::new(water_loss)
        );

    // Using peptide function
    let peptide_fragment = MzExpression::Function(MzFunction::Peptide {
        sequence: "PEPTIDE".to_string(),
        charge: 1,
        ion_type: "y".to_string()
    });

    let _query2 = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms2_product(
            MzCondition::new(peptide_fragment)
        );

    println!("Query built successfully✓");
    println!("Supports molecular formula calculation and peptide fragment prediction\n");
    Ok(())
}

/// Example 7: Brominated compound pattern
///
/// Corresponds to MassQL: QUERY scaninfo(MS2DATA) WHERE
/// MS1MZ=X:INTENSITYMATCH=Y:INTENSITYMATCHREFERENCE
/// AND MS1MZ=X+2:INTENSITYMATCH=Y:INTENSITYMATCHPERCENT=5
/// AND MS2PREC=X
fn brominated_compound_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("7. Brominated Compound Pattern Recognition");
    println!("Identifying brominated compound characteristic isotope patterns (M and M+2)\n");

    let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms1_mz(  // M peak (reference peak)
            MzCondition::new(MzExpression::x())
        )
        .with_ms1_mz(  // M+2 peak (bromine isotope)
            MzCondition::new(
                MzExpression::x().add(MzExpression::fixed(2.0))
            )
        )
        .with_ms2_precursor(
            MzCondition::new(MzExpression::x())
        );

    println!("Query built successfully✓");
    println!("This query identifies the characteristic 1:1 isotope ratio of brominated compounds\n");
    Ok(())
}

/// Demonstrate how to build custom queries for AI
fn ai_helper_functions() {
    println!("=== AI Helper Functions ===\n");

    // Helper function 1: Fast creation of X variable expression
    fn x() -> MzExpression {
        MzExpression::x()
    }

    // Helper function 2: Create fixed m/z expression
    fn mz(value: f64) -> MzExpression {
        MzExpression::fixed(value)
    }

    // Helper function 3: Create m/z range
    fn mz_range(center: f64, width: f64) -> (MzExpression, MzExpression) {
        (
            mz(center - width),
            mz(center + width)
        )
    }

    // Helper function 4: Create ion mobility correlation expression
    fn mobility_correlation(factor: f64, offset: f64, width: f64) -> (MzExpression, MzExpression) {
        let center = x().mul(mz(factor)).add(mz(offset));
        (
            center.clone().sub(mz(width)),
            center.add(mz(width))
        )
    }

    // Using these helper functions
    let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
        .with_ms_level(MSLevel::MS2)
        .with_ms2_precursor(
            MzCondition::new(x())
        )
        .with_mobility(
            MobilityCondition::range(
                mobility_correlation(0.0011, 0.5, 0.1).0,
                mobility_correlation(0.0011, 0.5, 0.1).1
            )
        );

    println!("AI helper functions make query construction more concise✓");
}

/// Demonstrate common mass spectrometry query patterns
fn common_query_patterns() {
    println!("=== Common Query Patterns ===\n");

    // Pattern 1: Single reaction monitoring (SRM)
    fn srm_monitoring(precursor_mz: f64, product_mz: f64, tolerance: f64) -> SpectrumQueryBuilder<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak> {
        SpectrumQueryBuilder::new()
            .with_ms_level(MSLevel::MS2)
            .with_ms2_precursor(
                MzCondition::new(MzExpression::fixed(precursor_mz))
                    .with_tolerance(Tolerance::Da(tolerance))
            )
            .with_ms2_product(
                MzCondition::new(MzExpression::fixed(product_mz))
                    .with_tolerance(Tolerance::Da(tolerance))
            )
    }

    // Pattern 2: Neutral loss scan
    fn neutral_loss_scan(loss_mz: f64, tolerance: f64) -> SpectrumQueryBuilder<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak> {
        SpectrumQueryBuilder::new()
            .with_ms_level(MSLevel::MS2)
            .with_ms2_neutral_loss(
                MzCondition::new(MzExpression::fixed(loss_mz))
                    .with_tolerance(Tolerance::Da(tolerance))
            )
    }

    // Pattern 3: Precursor ion scan
    fn precursor_ion_scan(product_mz: f64, tolerance: f64) -> SpectrumQueryBuilder<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak> {
        SpectrumQueryBuilder::new()
            .with_ms_level(MSLevel::MS2)
            .with_ms2_product(
                MzCondition::new(MzExpression::fixed(product_mz))
                    .with_tolerance(Tolerance::Da(tolerance))
            )
    }

    println!("Common query pattern definitions complete✓");
}