//! Small statistics helpers: mean, sample standard deviation and linear-interpolated percentiles.
use serde::{Deserialize, Serialize};

pub fn mean(v: &[f64]) -> Option<f64> {
    if v.is_empty() { None } else { Some(v.iter().sum::<f64>() / v.len() as f64) }
}

pub fn sd(v: &[f64]) -> Option<f64> {
    if v.len() < 2 { return None; }
    let m = mean(v)?;
    Some((v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt())
}

/// Percentile `p` in 0..=100, linear interpolation between order statistics (Hyndman-Fan type 7).
pub fn percentile(v: &[f64], p: f64) -> Option<f64> {
    if v.is_empty() { return None; }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let h = (s.len() - 1) as f64 * p / 100.0;
    let (lo, hi) = (h.floor() as usize, h.ceil() as usize);
    Some(s[lo] + (h - lo as f64) * (s[hi] - s[lo]))
}

pub fn round(x: f64, digits: i32) -> f64 {
    let f = 10f64.powi(digits);
    (x * f).round() / f
}

/// Distribution of one indicator across the normal years.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NormalStat {
    pub mean: f64,
    pub sd: Option<f64>,
    pub p10: f64,
    pub p90: f64,
    pub n_years: usize,
}

impl NormalStat {
    pub fn from(v: &[f64]) -> Option<Self> {
        Some(Self {
            mean: round(mean(v)?, 2),
            sd: sd(v).map(|x| round(x, 2)),
            p10: round(percentile(v, 10.0)?, 2),
            p90: round(percentile(v, 90.0)?, 2),
            n_years: v.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_stats() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean(&v), Some(3.0));
        assert!((sd(&v).unwrap() - 1.5811).abs() < 1e-3);
        assert_eq!(percentile(&v, 50.0), Some(3.0));
        assert!((percentile(&v, 10.0).unwrap() - 1.4).abs() < 1e-9);
        assert!((percentile(&v, 90.0).unwrap() - 4.6).abs() < 1e-9);
    }
}
