use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use mzdata::io::{DetailLevel, EICProgress, EICProgressUnit, EICQuery};
use mzdata::prelude::*;
use mzdata::MZReader;

fn render_progress(prefix: &str, processed: usize, total: usize, unit: &str, started: Instant) {
    const WIDTH: usize = 24;
    let total = total.max(1);
    let filled = ((processed.min(total) * WIDTH) / total).min(WIDTH);
    let empty = WIDTH.saturating_sub(filled);
    let bar = format!("{}{}", "#".repeat(filled), "-".repeat(empty));
    let percent = (processed.min(total) as f64 / total as f64) * 100.0;
    eprint!(
        "\r{prefix:<18} [{bar}] {processed:>6}/{total:<6} {unit:<8} {percent:>5.1}% {:>6.1}s",
        started.elapsed().as_secs_f64()
    );
}

fn finish_progress(prefix: &str, total: usize, unit: &str, started: Instant) {
    render_progress(prefix, total, total, unit, started);
    eprintln!();
}

fn manual_extract(
    reader: &mut MZReader<std::fs::File>,
    query: &EICQuery,
    mut progress: impl FnMut(usize, usize),
) -> Vec<(f64, f32)> {
    let min_intensity = query.min_intensity.unwrap_or_default();
    let original = *reader.detail_level();
    reader.set_detail_level(DetailLevel::Full);
    let result = (|| {
        let mut points = Vec::new();
        let total = reader.len();
        for index in 0..total {
            let Some(spectrum) = reader.get_spectrum_by_index(index) else {
                progress(index + 1, total);
                continue;
            };
            if spectrum.ms_level() != query.ms_level.unwrap_or(spectrum.ms_level()) {
                progress(index + 1, total);
                continue;
            }
            let time = spectrum.start_time();
            if let Some(rt_min) = query.rt_min {
                if time < rt_min {
                    progress(index + 1, total);
                    continue;
                }
            }
            if let Some(rt_max) = query.rt_max {
                if time > rt_max {
                    progress(index + 1, total);
                    continue;
                }
            }

            let intensity = if let Some(arrays) = spectrum.raw_arrays() {
                let mzs = arrays.mzs().unwrap_or_default();
                let intensities = arrays.intensities().unwrap_or_default();
                mzs.iter()
                    .copied()
                    .zip(intensities.iter().copied())
                    .filter(|(mz, intensity)| {
                        *mz >= query.mz_min && *mz <= query.mz_max && *intensity >= min_intensity
                    })
                    .map(|(_, intensity)| intensity)
                    .sum()
            } else {
                spectrum
                    .peaks()
                    .iter()
                    .filter(|peak| {
                        peak.mz() >= query.mz_min
                            && peak.mz() <= query.mz_max
                            && peak.intensity() >= min_intensity
                    })
                    .map(|peak| peak.intensity())
                    .sum()
            };
            points.push((time, intensity));
            progress(index + 1, total);
        }
        points
    })();
    reader.set_detail_level(original);
    result
}

fn peak_query_from_first_ms1(path: &Path, half_width: f64) -> io::Result<EICQuery> {
    let mut reader = MZReader::open_path(path)?;
    let original = *reader.detail_level();
    reader.set_detail_level(DetailLevel::Full);
    let started = Instant::now();
    let result = (|| {
        let total = reader.len();
        for index in 0..total {
            let Some(spectrum) = reader.get_spectrum_by_index(index) else {
                render_progress("select query", index + 1, total, "spectra", started);
                continue;
            };
            if spectrum.ms_level() != 1 {
                render_progress("select query", index + 1, total, "spectra", started);
                continue;
            }

            let mut peaks: Vec<(f64, f32)> = if let Some(arrays) = spectrum.raw_arrays() {
                let mzs = arrays.mzs().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "missing m/z array")
                })?;
                let intensities = arrays.intensities().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "missing intensity array")
                })?;
                mzs.iter()
                    .copied()
                    .zip(intensities.iter().copied())
                    .collect()
            } else {
                spectrum
                    .peaks()
                    .iter()
                    .map(|peak| (peak.mz(), peak.intensity()))
                    .collect()
            };
            peaks.sort_by(|a, b| b.1.total_cmp(&a.1));
            if let Some((mz, _)) = peaks
                .into_iter()
                .find(|(mz, intensity)| mz.is_finite() && intensity.is_finite() && *intensity > 0.0)
            {
                render_progress("select query", index + 1, total, "spectra", started);
                eprintln!();
                return Ok(EICQuery::new(mz - half_width, mz + half_width).with_ms_level(1));
            }
            render_progress("select query", index + 1, total, "spectra", started);
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no MS1 spectrum found for query selection",
        ))
    })();
    reader.set_detail_level(original);
    result
}

fn main() -> io::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut manual_check = false;
    let mut path_arg = None;
    for arg in args.by_ref() {
        if arg == "--manual-check" {
            manual_check = true;
        } else if path_arg.is_none() {
            path_arg = Some(arg);
        } else {
            eprintln!("unexpected argument: {arg}");
            process::exit(2);
        }
    }

    let path = PathBuf::from(path_arg.unwrap_or_else(|| {
        eprintln!(
            "usage: cargo run --example bruker_eic --features bruker_tdf,nalgebra -- <path-to-.d> [--manual-check]"
        );
        process::exit(1);
    }));
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("dataset not found: {}", path.display()),
        ));
    }

    eprintln!("dataset: {}", path.display());
    let query = peak_query_from_first_ms1(&path, 0.01)?;
    eprintln!(
        "query: mz=[{:.6}, {:.6}] ms_level={:?}",
        query.mz_min, query.mz_max, query.ms_level
    );

    let mut reader = MZReader::open_path(&path)?;
    let extract_started = Instant::now();
    let mut last_update = None;
    let eic = reader
        .extract_eic_with_progress(&query, &mut |update: EICProgress| {
            let unit = match update.unit {
                EICProgressUnit::Spectra => "spectra",
                EICProgressUnit::TdfEntries => "entries",
            };
            last_update = Some((update.total, unit));
            render_progress(
                "extract_eic",
                update.processed,
                update.total,
                unit,
                extract_started,
            );
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    if let Some((total, unit)) = last_update {
        finish_progress("extract_eic", total, unit, extract_started);
    } else {
        eprintln!("extract_eic       [########################] done");
    }

    let actual: Vec<_> = eic
        .times
        .iter()
        .copied()
        .zip(eic.intensities.iter().copied())
        .collect();
    let mut max_abs_diff = 0.0f32;

    if manual_check {
        let mut manual_reader = MZReader::open_path(&path)?;
        let manual_started = Instant::now();
        let mut manual_total = 0usize;
        let expected = manual_extract(&mut manual_reader, &query, |processed, total| {
            manual_total = total;
            render_progress("manual check", processed, total, "spectra", manual_started);
        });
        finish_progress("manual check", manual_total, "spectra", manual_started);

        if actual.len() != expected.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "point count mismatch: actual {} vs expected {}",
                    actual.len(),
                    expected.len()
                ),
            ));
        }

        for ((time, intensity), (expected_time, expected_intensity)) in
            actual.iter().copied().zip(expected.iter().copied())
        {
            if (time - expected_time).abs() >= 1e-9 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("time mismatch: {time} vs {expected_time}"),
                ));
            }
            let diff = (intensity - expected_intensity).abs();
            max_abs_diff = max_abs_diff.max(diff);
            if diff >= 1e-3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "intensity mismatch at time {time}: {intensity} vs {expected_intensity}"
                    ),
                ));
            }
        }
    }

    println!("points: {}", actual.len());
    if manual_check {
        println!("max_abs_diff: {:.6}", max_abs_diff);
    }
    if let Some((time, intensity)) = actual.iter().copied().find(|(_, intensity)| *intensity > 0.0)
    {
        println!("first_nonzero_point: time={time:.4} intensity={intensity:.4}");
    }
    Ok(())
}
