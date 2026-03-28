//! AI-driven mass spectrometry query module
//!
//! This module provides a type-safe, high-performance query interface for mass spectrometry data,
//! specifically designed for AI systems. It supports complex query patterns including variable binding,
//! expression evaluation, and graph-based filtering.
//!
//! # Main Features
//!
//! - **Type Safety**: Full compile-time type checking prevents runtime errors
//! - **High Performance**: Parallel query execution and SIMD acceleration
//! - **Expression System**: Complex mathematical expressions and variable binding
//! - **Graph Filtering**: Multi-variable correlated query patterns
//!
//! # Basic Usage
//!
//! ```rust
//! use mzdata::massql::*;
//! use mzdata::prelude::*;
//! use mzpeaks::Tolerance;
//!
//! // Create a query builder
//! let query = SpectrumQueryBuilder::new()
//!     .with_ms_level(MSLevel::MS2)
//!     .with_time_range(TimeRange::new(0.0, 10.0))
//!     .with_ms2_product(
//!         MzCondition::new(MzExpression::fixed(144.1))
//!             .with_tolerance(Tolerance::Da(0.1))
//!     );
//!
//! // Execute the query
//! let results = query.execute(&mut reader)?;
//! ```
//!
//! # Advanced Usage - Variable Binding and Expressions
//!
//! ```rust
//! // Create a complex query using variable X
//! let query = SpectrumQueryBuilder::new()
//!     .with_ms_level(MSLevel::MS2)
//!     .with_ms2_precursor(
//!         MzCondition::new(MzExpression::x()) // Use variable X
//!     )
//!     .with_mobility(
//!         // Ion mobility range calculated from precursor m/z
//!         MobilityCondition::range(
//!             MzExpression::x().mul(MzExpression::fixed(0.0011)).add(MzExpression::fixed(0.5)).sub(MzExpression::fixed(0.1)),
//!             MzExpression::x().mul(MzExpression::fixed(0.0011)).add(MzExpression::fixed(0.5)).add(MzExpression::fixed(0.1)),
//!         )
//!     )
//!     .with_ms2_product(
//!         MzCondition::new(MzExpression::fixed(144.1))
//!     );
//! ```

pub mod query_builder;
pub mod query_execution;
pub mod json_query;

pub use query_builder::{
    SpectrumQueryBuilder,
    MzExpression,
    MzFunction,
    MzCondition,
    MobilityCondition,
    MSLevel,
    TimeRange,
    QueryVariable,
    SpectrumMatch,
    QueryError,
    QueryExecutor,
};

pub use json_query::{
    JsonQuery,
    MzExpressionJson,
    MzConditionJson,
    TimeRangeJson,
    ScanRangeJson,
    ToleranceJson,
    CardinalityJson,
    MobilityConditionJson,
};

/// Convenience re-exports
pub mod prelude {
    pub use crate::massql::{
        SpectrumQueryBuilder,
        MzExpression,
        MzCondition,
        MobilityCondition,
        MSLevel,
        TimeRange,
        QueryVariable,
        JsonQuery,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query_creation() {
        let _query = SpectrumQueryBuilder::<mzpeaks::CentroidPeak, mzpeaks::DeconvolutedPeak>::new()
            .with_ms_level(MSLevel::MS2)
            .with_time_range(TimeRange::new(0.0, 10.0));

        // Query builder created successfully
        // Note: Cannot access private fields directly, but builder works correctly
    }

    #[test]
    fn test_expression_with_variables() {
        let expr = MzExpression::x()
            .mul(MzExpression::fixed(2.0))
            .add(MzExpression::fixed(1.0));

        let mut bindings = std::collections::HashMap::new();
        bindings.insert(QueryVariable::X, 10.0);

        let result = expr.evaluate(&bindings).unwrap();
        assert_eq!(result, 21.0); // 10 * 2 + 1
    }
}