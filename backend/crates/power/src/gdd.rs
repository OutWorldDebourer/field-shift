//! Degree-day helpers: crop growing degree days (averaging method) and the single-sine method with
//! horizontal upper cutoff used for insects (Zalom, F.G., Goodell, P.B., Wilson, L.T., Barnett, W.W., Bentley, W.J. 1983.
//! Degree-days: the calculation and use of heat units in pest management. UC DANR Leaflet 21373).
use std::f64::consts::PI;

pub const GDD_METHOD: &str = "Growing degree days, averaging method: sum of max(0, (Tmax + Tmin)/2 - base), POWER T2M_MAX / T2M_MIN";
pub const SINE_METHOD: &str = "Degree-days by the single-sine method with horizontal upper cutoff (Zalom et al. 1983, UC DANR Leaflet 21373), POWER T2M_MAX / T2M_MIN";

/// Daily GDD by the averaging method.
pub fn gdd_avg(tmax: f64, tmin: f64, base: f64) -> f64 {
    ((tmax + tmin) / 2.0 - base).max(0.0)
}

/// Daily degree-days by the single-sine method, lower threshold `tl`, optional upper threshold `tu` (horizontal cutoff).
pub fn dd_sine(tmin: f64, tmax: f64, tl: f64, tu: Option<f64>) -> f64 {
    let (tmin, tmax) = if tmin <= tmax { (tmin, tmax) } else { (tmax, tmin) };
    let tu = tu.unwrap_or(f64::INFINITY);
    if tmin >= tu { return tu - tl; }
    if tmax <= tl { return 0.0; }
    let m = (tmax + tmin) / 2.0;
    let a = (tmax - tmin) / 2.0;
    if tmin >= tl && tmax <= tu { return m - tl; }
    let t1 = if tmin < tl { ((tl - m) / a).clamp(-1.0, 1.0).asin() } else { -PI / 2.0 };
    let t2 = if tmax > tu { ((tu - m) / a).clamp(-1.0, 1.0).asin() } else { PI / 2.0 };
    // Integral of (m + a sin θ - tl) for θ in [t1, t2], plus (tu - tl) for θ in [t2, π/2], over the half cycle π.
    let lower = (m - tl) * (t2 - t1) + a * (t1.cos() - t2.cos());
    let upper = if tmax > tu { (tu - tl) * (PI / 2.0 - t2) } else { 0.0 };
    (lower + upper) / PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gdd_averaging() {
        assert_eq!(gdd_avg(30.0, 20.0, 10.0), 15.0);
        assert_eq!(gdd_avg(12.0, 4.0, 10.0), 0.0);
    }

    /// Brute-force integral of the same sine day curve with the same thresholds.
    fn numeric(tmin: f64, tmax: f64, tl: f64, tu: Option<f64>) -> f64 {
        let n = 200_000;
        let (m, a) = ((tmax + tmin) / 2.0, (tmax - tmin) / 2.0);
        let mut s = 0.0;
        for k in 0..n {
            let th = -PI / 2.0 + PI * (k as f64 + 0.5) / n as f64;
            let t = (m + a * th.sin()).min(tu.unwrap_or(f64::INFINITY));
            s += (t - tl).max(0.0);
        }
        s / n as f64
    }

    #[test]
    fn sine_matches_numeric_integral() {
        for &(tmin, tmax, tl, tu) in &[
            (0.0, 20.0, 10.0, None),
            (5.0, 35.0, 10.0, Some(30.0)),
            (15.0, 35.0, 10.0, Some(30.0)),
            (2.0, 12.0, 4.0, Some(30.0)),
            (16.0, 22.0, 12.57, None),
            (-3.0, 8.0, 4.0, Some(30.0)),
        ] {
            let a = dd_sine(tmin, tmax, tl, tu);
            let b = numeric(tmin, tmax, tl, tu);
            assert!((a - b).abs() < 1e-3, "{tmin} {tmax} {tl} {tu:?}: {a} vs {b}");
        }
        assert_eq!(dd_sine(0.0, 3.0, 4.0, None), 0.0);
        assert_eq!(dd_sine(31.0, 35.0, 4.0, Some(30.0)), 26.0);
    }
}
