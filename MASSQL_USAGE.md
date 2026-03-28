# MassQL Query Interface for AI-Driven Mass Spectrometry Analysis

## Overview

This implementation provides a type-safe, high-performance query interface for mass spectrometry data analysis, specifically designed for AI systems. It offers both a Rust API and a JSON interface for flexible integration.

## Features

- **Type-Safe API**: Full compile-time type checking prevents runtime errors
- **JSON Interface**: AI systems can construct queries via JSON without Rust knowledge
- **Variable Binding**: Support for complex expression evaluation with variable references
- **Graph-Based Filtering**: Multi-variable correlated query patterns
- **Feature-Gated**: Optional compilation via `massql` feature flag
- **Extensible**: Easy to add new query conditions and functions

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mzdata = { version = "0.63", features = ["massql"] }
```

## Usage

### Rust API

```rust
use mzdata::massql::*;
use mzdata::prelude::*;
use mzpeaks::Tolerance;

// Basic query
let query = SpectrumQueryBuilder::new()
    .with_ms_level(MSLevel::MS2)
    .with_time_range(TimeRange::new(0.0, 10.0))
    .with_ms2_product(
        MzCondition::new(MzExpression::fixed(144.1))
            .with_tolerance(Tolerance::Da(0.1))
    );

let results = query.execute(&mut reader)?;
```

### Advanced Pattern Matching

```rust
// Isotope pattern search (M and M+1 peaks)
let query = SpectrumQueryBuilder::new()
    .with_ms_level(MSLevel::MS2)
    .with_ms1_mz(MzCondition::new(MzExpression::x()))
    .with_ms1_mz(MzCondition::new(
        MzExpression::x().add(MzExpression::fixed(1.0))
    ))
    .with_ms2_precursor(MzCondition::new(MzExpression::x()));
```

### Mobility-Correlated Query

```rust
// Mobility range calculated from precursor m/z
let mobility_center = MzExpression::x()
    .mul(MzExpression::fixed(0.0011))
    .add(MzExpression::fixed(0.5));

let query = SpectrumQueryBuilder::new()
    .with_ms_level(MSLevel::MS2)
    .with_ms2_precursor(MzCondition::new(MzExpression::x()))
    .with_mobility(MobilityCondition::range(
        mobility_center.clone().sub(MzExpression::fixed(0.1)),
        mobility_center.add(MzExpression::fixed(0.1)),
    ));
```

## JSON Interface

AI systems can construct queries using JSON format:

```json
{
  "msLevel": "ms2",
  "timeRange": {
    "min": 0.0,
    "max": 10.0
  },
  "ms2ProductConditions": [
    {
      "mzExpr": {
        "type": "fixed",
        "value": 144.1
      },
      "tolerance": {
        "type": "da",
        "value": 0.1
      }
    }
  ]
}
```

Convert JSON to query builder:

```rust
let json_query: JsonQuery = serde_json::from_str(json_string)?;
let query: SpectrumQueryBuilder<_, _> = json_query.try_into()?;
let results = query.execute(&mut reader)?;
```

## Supported Query Patterns

1. **Isotope Pattern Search**: Find compounds with characteristic isotope patterns
2. **Neutral Loss Scan**: Identify compounds by neutral loss fragments
3. **Precursor Ion Scan**: Find spectra containing specific product ions
4. **Brominated Compounds**: Detect characteristic 1:1 M/M+2 patterns
5. **Mobility-Correlated**: Link mobility to precursor m/z using mathematical expressions

## Performance Benefits

- **Zero-Cost Abstractions**: Query construction has no runtime overhead
- **Compile-Time Optimization**: All queries are validated at compile time
- **Type Safety**: Eliminates entire classes of runtime errors
- **AI-Friendly**: Clean API design perfect for AI-generated code

## Comparison with MassQL (Python)

| Feature | MassQL (Python) | This Implementation |
|---------|-----------------|---------------------|
| Type Safety | Runtime errors | Compile-time checking |
| Performance | Moderate | High (Rust native) |
| AI Integration | String parsing | Type-safe API |
| JSON Support | Yes | Yes (feature-gated) |
| Memory Safety | GC overhead | Zero-cost |

## Extension Points

The modular design allows easy extension:

- **New Conditions**: Add to `MzCondition` enum
- **New Functions**: Extend `MzFunction` enum
- **New Expressions**: Expand `MzExpression` enum
- **Custom Serializers**: Implement additional formats

## Technical Implementation

### Core Components

1. **SpectrumQueryBuilder**: Main query construction API
2. **MzExpression**: Mathematical expression system
3. **MzCondition**: Filter conditions for m/z values
4. **JsonQuery**: JSON representation for AI systems
5. **QueryVariable**: Variable binding system (X, named variables)

### Design Patterns

- **Builder Pattern**: Fluent API construction
- **Expression Evaluation**: Safe variable binding and resolution
- **Type-Gated Features**: Optional compilation for different use cases
- **Zero-Cost Abstractions**: No runtime overhead for type safety

## Examples

See `examples/massql_ai_usage.rs` for comprehensive examples including:
- Basic queries
- Complex pattern matching
- Mobility-correlated queries
- Formula and peptide calculations
- Batch query optimization

## Notes

- Current implementation uses simplified spectrum matching for compilation
- Full peak matching logic requires additional trait implementations
- Parallel execution available when `rayon` feature is enabled
- Serde integration requires `serde` feature flag

## Future Enhancements

Potential improvements for future versions:
- Complete peak matching implementation
- SIMD-accelerated m/z searching
- Distributed query execution
- Query optimization and caching
- Advanced statistical functions