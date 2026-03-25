mod inference;
mod dispatch;
mod pipeline;

pub use dispatch::{MZReader, MZReaderType, MZReaderBuilder, IMMZReaderType};
#[cfg(feature = "async_partial")]
pub use dispatch::{AsyncMZReaderType, AsyncMZReader, AsyncMZReaderBuilder};



pub use inference::{infer_from_path, infer_from_stream, infer_format, MassSpectrometryFormat};

pub use pipeline::{MassSpectrometryReadWriteProcess, Source, Sink};

#[cfg(test)]
mod test {
    use std::{fs, io, path};

    #[cfg(feature = "bruker_tdf")]
    use std::time::Instant;

    use mzpeaks::{CentroidPeak, DeconvolutedPeak, IntensityMeasurement, MZLocated};

    use crate::{
        prelude::*,
        spectrum::{ArrayType, Spectrum},
        io::{DetailLevel, EICQuery, ExtractedIonChromatogramSource}
    };

    use super::*;

    fn manual_extract(reader: &mut MZReader<std::fs::File>, query: &EICQuery) -> Vec<(f64, f32)> {
        let mut points = Vec::new();
        let original = *reader.detail_level();
        reader.set_detail_level(DetailLevel::Lazy);

        for index in 0..reader.len() {
            let Some(spectrum) = reader.get_spectrum_by_index(index) else {
                continue;
            };
            let time = spectrum.start_time();
            if let Some(rt_min) = query.rt_min {
                if time < rt_min {
                    continue;
                }
            }
            if let Some(rt_max) = query.rt_max {
                if time > rt_max {
                    continue;
                }
            }
            if let Some(ms_level) = query.ms_level {
                if spectrum.ms_level() != ms_level {
                    continue;
                }
            }

            let min_intensity = query.min_intensity.unwrap_or_default();
            let intensity = if let Some(arrays) = spectrum.raw_arrays() {
                let mzs = arrays.mzs().unwrap();
                let intensities = arrays.intensities().unwrap();
                let lower = mzs.partition_point(|mz| *mz < query.mz_min);
                let upper = mzs.partition_point(|mz| *mz <= query.mz_max);
                intensities[lower..upper]
                    .iter()
                    .copied()
                    .filter(|intensity| *intensity >= min_intensity)
                    .sum()
            } else {
                spectrum
                    .peaks()
                    .iter()
                    .filter(|point| {
                        point.mz() >= query.mz_min
                            && point.mz() <= query.mz_max
                            && point.intensity() >= min_intensity
                    })
                    .map(|point| point.intensity())
                    .sum()
            };
            points.push((time, intensity));
        }

        reader.set_detail_level(original);
        points
    }

    #[cfg(feature = "bruker_tdf")]
    fn peak_queries_from_first_ms1(
        reader: &mut MZReader<std::fs::File>,
        count: usize,
        half_width: f64,
    ) -> io::Result<Vec<EICQuery>> {
        let original = *reader.detail_level();
        reader.set_detail_level(DetailLevel::Full);
        let result = (|| {
            for index in 0..reader.len() {
                let Some(spectrum) = reader.get_spectrum_by_index(index) else {
                    continue;
                };
                if spectrum.ms_level() != 1 {
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
                let queries = peaks
                    .into_iter()
                    .filter(|(mz, intensity)| mz.is_finite() && intensity.is_finite() && *intensity > 0.0)
                    .take(count)
                    .map(|(mz, _)| EICQuery::new(mz - half_width, mz + half_width).with_ms_level(1))
                    .collect();
                return Ok(queries);
            }
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no MS1 spectrum found for query selection",
            ))
        })();
        reader.set_detail_level(original);
        result
    }

    #[cfg(feature = "bruker_tdf")]
    fn bruker_desktop_datasets() -> [std::path::PathBuf; 2] {
        let data_dir = path::Path::new(r"C:\Users\ray\Desktop\MS_Data_20260318");
        [
            data_dir.join(
                "20260205_0130helaQC-withDDM_200pg_08uL_nano2_SH_ultra2_shortViper_20min_150nl_0120qitian-75um_1500V_Slot2-2_1_2839.d",
            ),
            data_dir.join(
                "20260305_0305helaQC-withDDM_200pg_1uL_nano2_SH_AIP_shortViper_20min_150nl_0305qitian-75um_1500V_Slot2-2_1_2861.d",
            ),
        ]
    }

    #[cfg(feature = "mzml")]
    #[test]
    fn infer_mzml() {
        let path = path::Path::new("./test/data/small.mzML");
        assert!(path.exists());
        let (fmt, zipped) = infer_from_path(path);
        assert_eq!(fmt, MassSpectrometryFormat::MzML);
        assert!(!zipped);
    }

    #[test]
    fn infer_mgf() {
        let path = path::Path::new("./test/data/small.mgf");
        assert!(path.exists());
        let (fmt, zipped) = infer_from_path(path);
        assert_eq!(fmt, MassSpectrometryFormat::MGF);
        assert!(!zipped);
    }

    #[cfg(feature = "thermo")]
    #[test]
    fn infer_thermo() {
        let path = path::Path::new("./test/data/small.RAW");
        let (fmt, zipped) = infer_from_path(path);
        assert_eq!(fmt, MassSpectrometryFormat::ThermoRaw);
        assert!(!zipped);
    }

    #[cfg(feature = "mzml")]
    #[test]
    fn infer_open() {
        let path = path::Path::new("./test/data/small.mzML");
        assert!(path.exists());
        if let Ok(mut reader) = MZReader::open_path(path) {
            assert_eq!(reader.len(), 48);
            assert_eq!(*reader.detail_level(), DetailLevel::Full);
            if let Some(spec) = reader.get_spectrum_by_index(10) {
                let spec: Spectrum = spec;
                assert!(spec.index() == 10);
                assert!(spec.id() == "controllerType=0 controllerNumber=1 scan=11");
                if let Some(data_arrays) = &spec.arrays {
                    assert!(data_arrays.has_array(&ArrayType::MZArray));
                    assert!(data_arrays.has_array(&ArrayType::IntensityArray));
                    let mzs = data_arrays.mzs().unwrap();
                    assert!(mzs.len() == 941);
                }
            } else {
                panic!("Failed to retrieve spectrum by index")
            }

            assert_eq!(reader.get_spectrum_by_id("controllerType=0 controllerNumber=1 scan=11").unwrap().index(), 10);

            if let Some(spec) = reader.get_spectrum_by_time(0.358558333333) {
                assert_eq!(spec.index(), 34);
            } else {
                panic!("Failed to retrieve spectrum by time")
            }

        } else {
            panic!("Failed to open file")
        }
    }

    #[cfg(feature = "thermo")]
    #[test]
    fn infer_open_thermo() {
        let path = path::Path::new("./test/data/small.RAW");
        assert!(path.exists());
        if let Ok(mut reader) = MZReader::open_path(path) {
            assert_eq!(reader.len(), 48);
            assert_eq!(*reader.detail_level(), DetailLevel::Full);
            if let Some(spec) = reader.get_spectrum_by_index(10) {
                let spec: Spectrum = spec;
                assert_eq!(spec.index(), 10);
                assert_eq!(spec.id(), "controllerType=0 controllerNumber=1 scan=11");
                assert_eq!(spec.peaks().len(), 941);
            } else {
                panic!("Failed to retrieve spectrum by index")
            }

            assert_eq!(reader.get_spectrum_by_id("controllerType=0 controllerNumber=1 scan=11").unwrap().index(), 10);

            if let Some(spec) = reader.get_spectrum_by_time(0.358558333333) {
                assert_eq!(spec.index(), 34);
            } else {
                panic!("Failed to retrieve spectrum by time")
            }

        } else {
            panic!("Failed to open file")
        }
    }

    #[test]
    fn test_source_conv() -> io::Result<()> {
        let s = Source::<CentroidPeak, DeconvolutedPeak>::from("text/path".as_ref());
        assert!(matches!(s, Source::PathLike(_)));

        let fh = Box::new(io::BufReader::new(fs::File::open("./test/data/small.mgf")?)) as Box<dyn SeekRead + Send>;
        let rs: Source<CentroidPeak, DeconvolutedPeak> = (fh, MassSpectrometryFormat::MGF).into();
        assert!(matches!(rs, Source::Reader(_, _)));

        Ok(())
    }

    #[test]
    fn test_eic_query_builder_carries_the_full_phase_one_filter_set() {
        let query = EICQuery::new(650.0, 651.0)
            .with_rt_range(10.0, 12.5)
            .with_ms_level(2)
            .with_mobility_range(1.1, 1.4)
            .with_min_intensity(42.0);

        assert_eq!(query.mz_min, 650.0);
        assert_eq!(query.mz_max, 651.0);
        assert_eq!(query.rt_min, Some(10.0));
        assert_eq!(query.rt_max, Some(12.5));
        assert_eq!(query.ms_level, Some(2));
        assert_eq!(query.mobility_min, Some(1.1));
        assert_eq!(query.mobility_max, Some(1.4));
        assert_eq!(query.min_intensity, Some(42.0_f32));
    }

    #[cfg(feature = "mzml")]
    #[test]
    fn test_dispatch_mzreader() -> io::Result<()> {
        let mut reader = MZReader::open_path("./test/data/small.mzML")?;

        let n = reader.len();
        let n_ms1 = reader.iter().filter(|s| s.ms_level() == 1).count();
        let n_msn = reader.iter().filter(|s| s.ms_level() == 2).count();

        assert_eq!(n, 48);
        assert_eq!(n, n_ms1 + n_msn);
        Ok(())
    }

    #[cfg(feature = "mzml")]
    #[test]
    fn test_infer_stream() -> io::Result<()> {
        let mut mzml_file = fs::File::open("./test/data/small.mzML")?;
        let (form, gzip) = infer_from_stream(&mut mzml_file)?;
        assert_eq!(form, MassSpectrometryFormat::MzML);
        assert!(!gzip);

        mzml_file = fs::File::open("./test/data/20200204_BU_8B8egg_1ug_uL_7charges_60_min_Slot2-11_1_244.mzML.gz")?;
        let (form, gzip) = infer_from_stream(&mut mzml_file)?;
        assert_eq!(form, MassSpectrometryFormat::MzML);
        assert!(gzip);

        let mut mgf_file = fs::File::open("./test/data/small.mgf")?;
        let (form, gzip) = infer_from_stream(&mut mgf_file)?;
        assert_eq!(form, MassSpectrometryFormat::MGF);
        assert!(!gzip);
        Ok(())
    }

    #[cfg(feature = "mzml")]
    #[test]
    fn test_extract_eic_dispatch_mzml() -> io::Result<()> {
        let mut reader = MZReader::open_path("./test/data/small.mzML")?;
        let query = EICQuery::new(644.0, 645.0);
        let manual = manual_extract(&mut reader, &query);

        let eic = reader.extract_eic(&query).unwrap();
        assert_eq!(eic.times.len(), eic.intensities.len());
        assert!(!eic.times.is_empty());
        assert_eq!(eic.times.len(), manual.len());
        for ((time, intensity), (expected_time, expected_intensity)) in eic
            .times
            .iter()
            .copied()
            .zip(eic.intensities.iter().copied())
            .zip(manual.into_iter())
        {
            assert!((time - expected_time).abs() < 1e-9);
            assert!((intensity - expected_intensity).abs() < 1e-3);
        }
        Ok(())
    }

    #[cfg(feature = "mzml")]
    #[test]
    fn test_extract_eics_dispatch_batch_mzml() -> io::Result<()> {
        let mut reader = MZReader::open_path("./test/data/small.mzML")?;
        let queries = vec![EICQuery::new(644.0, 645.0), EICQuery::new(810.3, 810.5)];
        let expected: Vec<_> = queries
            .iter()
            .map(|query| manual_extract(&mut reader, query))
            .collect();

        let eics = reader.extract_eics(&queries).unwrap();
        assert_eq!(eics.len(), queries.len());
        for (eic, expected) in eics.into_iter().zip(expected.into_iter()) {
            assert_eq!(eic.times.len(), expected.len());
            for ((time, intensity), (expected_time, expected_intensity)) in eic
                .times
                .iter()
                .copied()
                .zip(eic.intensities.iter().copied())
                .zip(expected.into_iter())
            {
                assert!((time - expected_time).abs() < 1e-9);
                assert!((intensity - expected_intensity).abs() < 1e-3);
            }
        }
        Ok(())
    }

    #[cfg(feature = "bruker_tdf")]
    #[test]
    fn test_extract_eic_dispatch_tdf() -> io::Result<()> {
        let mut reader = MZReader::open_path("test/data/diaPASEF.d")?;
        let query = EICQuery::new(500.0, 501.0).with_ms_level(1);
        let manual = manual_extract(&mut reader, &query);

        let eic = reader.extract_eic(&query).unwrap();
        assert_eq!(eic.times.len(), eic.intensities.len());
        assert_eq!(eic.times.len(), manual.len());
        for ((time, intensity), (expected_time, expected_intensity)) in eic
            .times
            .iter()
            .copied()
            .zip(eic.intensities.iter().copied())
            .zip(manual.into_iter())
        {
            assert!((time - expected_time).abs() < 1e-9);
            assert!(
                (intensity - expected_intensity).abs() < 1e-3,
                "tdf mismatch at time {time}: got {intensity}, expected {expected_intensity}"
            );
        }
        Ok(())
    }

    #[cfg(feature = "bruker_tdf")]
    #[test]
    #[ignore = "local benchmark against desktop Bruker datasets"]
    fn test_extract_eic_bruker_desktop_regression() -> io::Result<()> {
        let datasets = bruker_desktop_datasets();

        for dataset in datasets {
            assert!(dataset.exists(), "missing dataset {}", dataset.display());

            let mut query_reader = MZReader::open_path(&dataset)?;
            let queries = peak_queries_from_first_ms1(&mut query_reader, 3, 0.01)?;
            drop(query_reader);

            let mut fast_reader = MZReader::open_path(&dataset)?;
            let start = Instant::now();
            let eics = fast_reader.extract_eics(&queries).unwrap();
            let fast_elapsed = start.elapsed();

            let mut manual_reader = MZReader::open_path(&dataset)?;
            let start = Instant::now();
            let expected: Vec<_> = queries
                .iter()
                .map(|query| manual_extract(&mut manual_reader, query))
                .collect();
            let manual_elapsed = start.elapsed();

            println!("dataset: {}", dataset.display());
            println!("  fast_elapsed: {:?}", fast_elapsed);
            println!("  manual_elapsed: {:?}", manual_elapsed);

            for (query_index, (eic, expected)) in eics.iter().zip(expected.iter()).enumerate() {
                assert_eq!(
                    eic.times.len(),
                    expected.len(),
                    "point count mismatch for dataset {} query {}",
                    dataset.display(),
                    query_index
                );
                let mut max_abs_diff = 0.0f32;
                for ((time, intensity), (expected_time, expected_intensity)) in eic
                    .times
                    .iter()
                    .copied()
                    .zip(eic.intensities.iter().copied())
                    .zip(expected.iter().copied())
                {
                    assert!(
                        (time - expected_time).abs() < 1e-9,
                        "time mismatch for dataset {} query {}: {time} vs {expected_time}",
                        dataset.display(),
                        query_index
                    );
                    let diff = (intensity - expected_intensity).abs();
                    max_abs_diff = max_abs_diff.max(diff);
                    assert!(
                        diff < 1e-3,
                        "intensity mismatch for dataset {} query {} at time {time}: got {intensity}, expected {expected_intensity}",
                        dataset.display(),
                        query_index
                    );
                }
                println!(
                    "  query[{query_index}] mz=[{:.6}, {:.6}] points={} max_abs_diff={:.6}",
                    eic.query.mz_min,
                    eic.query.mz_max,
                    eic.times.len(),
                    max_abs_diff
                );
            }
        }

        Ok(())
    }

    #[cfg(feature = "bruker_tdf")]
    #[test]
    #[ignore = "local smoke test against desktop Bruker datasets"]
    fn test_extract_eic_bruker_desktop_smoke() -> io::Result<()> {
        let datasets = bruker_desktop_datasets();

        for dataset in datasets {
            assert!(dataset.exists(), "missing dataset {}", dataset.display());
            println!("dataset: {}", dataset.display());
            println!("  step 1/3: selecting one MS1 query");

            let mut query_reader = MZReader::open_path(&dataset)?;
            let query = peak_queries_from_first_ms1(&mut query_reader, 1, 0.01)?
                .into_iter()
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no query produced"))?;
            drop(query_reader);

            println!(
                "  step 2/3: fast extract for mz=[{:.6}, {:.6}]",
                query.mz_min, query.mz_max
            );
            let mut fast_reader = MZReader::open_path(&dataset)?;
            let fast_start = Instant::now();
            let eic = fast_reader.extract_eic(&query).unwrap();
            let fast_elapsed = fast_start.elapsed();

            println!("  step 3/3: manual validation");
            let mut manual_reader = MZReader::open_path(&dataset)?;
            let manual_start = Instant::now();
            let expected = manual_extract(&mut manual_reader, &query);
            let manual_elapsed = manual_start.elapsed();

            assert_eq!(eic.times.len(), expected.len());
            let mut max_abs_diff = 0.0f32;
            for ((time, intensity), (expected_time, expected_intensity)) in eic
                .times
                .iter()
                .copied()
                .zip(eic.intensities.iter().copied())
                .zip(expected.into_iter())
            {
                assert!((time - expected_time).abs() < 1e-9);
                let diff = (intensity - expected_intensity).abs();
                max_abs_diff = max_abs_diff.max(diff);
                assert!(
                    diff < 1e-3,
                    "intensity mismatch for dataset {} at time {time}: got {intensity}, expected {expected_intensity}",
                    dataset.display()
                );
            }

            println!("  fast_elapsed: {:?}", fast_elapsed);
            println!("  manual_elapsed: {:?}", manual_elapsed);
            println!("  points: {}", eic.times.len());
            println!("  max_abs_diff: {:.6}", max_abs_diff);
        }

        Ok(())
    }

    #[cfg(feature = "bruker_tdf")]
    #[test]
    #[ignore = "local benchmark against desktop Bruker datasets (fast path only)"]
    fn test_extract_eic_bruker_desktop_fast_bench() -> io::Result<()> {
        let datasets = bruker_desktop_datasets();

        for dataset in datasets {
            assert!(dataset.exists(), "missing dataset {}", dataset.display());
            println!("dataset: {}", dataset.display());
            println!("  step 1/2: selecting three MS1 queries");

            let mut query_reader = MZReader::open_path(&dataset)?;
            let queries = peak_queries_from_first_ms1(&mut query_reader, 3, 0.01)?;
            drop(query_reader);

            println!("  step 2/2: running fast batch extract for {} queries", queries.len());
            let mut fast_reader = MZReader::open_path(&dataset)?;
            let start = Instant::now();
            let eics = fast_reader.extract_eics(&queries).unwrap();
            let elapsed = start.elapsed();

            println!("  fast_elapsed: {:?}", elapsed);
            for (query_index, eic) in eics.iter().enumerate() {
                println!(
                    "  query[{query_index}] mz=[{:.6}, {:.6}] points={}",
                    eic.query.mz_min,
                    eic.query.mz_max,
                    eic.times.len()
                );
            }
        }

        Ok(())
    }
}
