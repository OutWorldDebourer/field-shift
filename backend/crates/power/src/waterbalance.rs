//! Simple climatic water balance: rain minus crop water demand (ET0 × Kc). Irrigation, runoff, deep percolation and
//! soil storage are not modelled; a negative value is the deficit irrigation would have to cover.

pub const METHOD: &str = "Climatic water balance = PRECTOTCORR total - ET0 × Kc (FAO-56 single crop coefficient, Kc from the request, default 1.0); irrigation, runoff, percolation and soil storage not modelled";
pub const DEFAULT_KC: f64 = 1.0;

/// Crop evapotranspiration ETc = Kc × ET0 (FAO-56 eq. 56).
pub fn etc(et0_mm: f64, kc: f64) -> f64 { et0_mm * kc }

/// Rain minus ETc in mm.
pub fn balance(rain_mm: f64, et0_mm: f64, kc: f64) -> f64 { rain_mm - etc(et0_mm, kc) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn balance_sign() {
        assert_eq!(balance(10.0, 100.0, 1.0), -90.0);
        assert!((balance(50.0, 40.0, 0.5) - 30.0).abs() < 1e-12);
    }
}
