//! FAO-56 Penman-Monteith reference evapotranspiration (grass, daily step).
//! Allen, R.G., Pereira, L.S., Raes, D., Smith, M. (1998). Crop evapotranspiration: guidelines for computing
//! crop water requirements. FAO Irrigation and Drainage Paper 56. Rome: FAO. https://www.fao.org/4/x0490e/x0490e00.htm

pub const METHOD: &str = "FAO-56 Penman-Monteith grass reference ET0 (Allen et al. 1998, FAO Irrigation and Drainage Paper 56, eq. 6), daily step, G = 0; inputs from POWER T2M_MAX, T2M_MIN, T2MDEW (actual vapour pressure, eq. 14; RH2M fallback, eq. 19), WS2M, ALLSKY_SFC_SW_DWN, PS (or eq. 7 from the POWER cell elevation), Rso from eq. 37";

/// Saturation vapour pressure e°(T) in kPa (FAO-56 eq. 11).
pub fn sat_vp(t: f64) -> f64 {
    0.6108 * (17.27 * t / (t + 237.3)).exp()
}

/// Atmospheric pressure in kPa from elevation in m (FAO-56 eq. 7).
pub fn pressure_from_elevation(z: f64) -> f64 {
    101.3 * ((293.0 - 0.0065 * z) / 293.0).powf(5.26)
}

/// Extraterrestrial radiation Ra in MJ m-2 day-1 (FAO-56 eq. 21-25).
pub fn ra(lat_deg: f64, doy: u32) -> f64 {
    let j = doy as f64;
    let phi = lat_deg.to_radians();
    let dr = 1.0 + 0.033 * (2.0 * std::f64::consts::PI * j / 365.0).cos();
    let delta = 0.409 * (2.0 * std::f64::consts::PI * j / 365.0 - 1.39).sin();
    let ws = (-phi.tan() * delta.tan()).clamp(-1.0, 1.0).acos();
    24.0 * 60.0 / std::f64::consts::PI * 0.0820 * dr
        * (ws * phi.sin() * delta.sin() + phi.cos() * delta.cos() * ws.sin())
}

/// Inputs for one day. `ea` is actual vapour pressure (kPa), `rs` shortwave in MJ m-2 day-1, `u2` wind at 2 m (m/s).
pub struct Et0Input {
    pub tmax: f64,
    pub tmin: f64,
    pub ea: f64,
    pub rs: f64,
    pub u2: f64,
    pub lat: f64,
    pub doy: u32,
    pub elevation: f64,
    pub pressure_kpa: f64,
}

/// ET0 in mm/day (FAO-56 eq. 6, G = 0 for daily steps).
pub fn et0(i: &Et0Input) -> f64 {
    let tmean = (i.tmax + i.tmin) / 2.0;
    let es = (sat_vp(i.tmax) + sat_vp(i.tmin)) / 2.0;
    let delta = 4098.0 * sat_vp(tmean) / (tmean + 237.3).powi(2);
    let gamma = 0.665e-3 * i.pressure_kpa;
    let ra = ra(i.lat, i.doy);
    let rso = (0.75 + 2e-5 * i.elevation) * ra;
    let rns = (1.0 - 0.23) * i.rs;
    let sigma = 4.903e-9;
    let ratio = if rso > 0.0 { (i.rs / rso).min(1.0) } else { 0.5 };
    let rnl = sigma * ((i.tmax + 273.16).powi(4) + (i.tmin + 273.16).powi(4)) / 2.0
        * (0.34 - 0.14 * i.ea.max(0.0).sqrt())
        * (1.35 * ratio - 0.35);
    let rn = rns - rnl;
    let num = 0.408 * delta * rn + gamma * 900.0 / (tmean + 273.0) * i.u2 * (es - i.ea).max(0.0);
    let den = delta + gamma * (1.0 + 0.34 * i.u2);
    (num / den).max(0.0)
}

/// ET0 from POWER daily fields; `None` if a required input is missing.
#[allow(clippy::too_many_arguments)]
pub fn et0_from_power(tmax: Option<f64>, tmin: Option<f64>, tdew: Option<f64>, rh: Option<f64>, ws: Option<f64>,
                      rs: Option<f64>, ps: Option<f64>, lat: f64, doy: u32, elevation: f64) -> Option<f64> {
    let (tmax, tmin, u2, rs) = (tmax?, tmin?, ws?, rs?);
    let ea = match (tdew, rh) {
        (Some(td), _) => sat_vp(td),
        (None, Some(rh)) => rh / 100.0 * (sat_vp(tmax) + sat_vp(tmin)) / 2.0,
        _ => return None,
    };
    let pressure_kpa = ps.unwrap_or_else(|| pressure_from_elevation(elevation));
    Some(et0(&Et0Input { tmax, tmin, ea, rs, u2, lat, doy, elevation, pressure_kpa }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FAO-56 Example 8: Ra at 20°S on 3 September (doy 246) = 32.2 MJ m-2 day-1.
    #[test]
    fn ra_example_8() {
        assert!((ra(-20.0, 246) - 32.2).abs() < 0.1, "{}", ra(-20.0, 246));
    }

    /// FAO-56 Example 18: Brussels (50°48' N, 100 m), 6 July (doy 187). Published: Ra 41.09, ET0 = 3.9 mm/day.
    #[test]
    fn et0_example_18_brussels() {
        let lat = 50.0 + 48.0 / 60.0;
        assert!((ra(lat, 187) - 41.09).abs() < 0.05, "Ra {}", ra(lat, 187));
        let p = pressure_from_elevation(100.0);
        assert!((p - 100.1).abs() < 0.05, "P {p}");
        // ea from RHmax 84 % and RHmin 63 % (eq. 17) = 1.409 kPa in the example.
        let ea = (sat_vp(12.3) * 0.84 + sat_vp(21.5) * 0.63) / 2.0;
        assert!((ea - 1.409).abs() < 0.005, "ea {ea}");
        // u2 = 2.078 m/s (10 km/h at 10 m converted, eq. 47); Rs = 22.07 MJ m-2 day-1 (n = 9.25 h).
        let e = et0(&Et0Input { tmax: 21.5, tmin: 12.3, ea, rs: 22.07, u2: 2.078, lat, doy: 187, elevation: 100.0, pressure_kpa: p });
        assert!((e - 3.9).abs() < 0.1, "ET0 {e}");
    }

    #[test]
    fn missing_input_gives_none() {
        assert!(et0_from_power(Some(25.0), Some(15.0), None, None, Some(2.0), Some(20.0), None, -11.5, 300, 224.0).is_none());
        let v = et0_from_power(Some(25.0), Some(15.0), None, Some(70.0), Some(2.0), Some(20.0), None, -11.5, 300, 224.0).unwrap();
        assert!(v > 2.0 && v < 6.0, "{v}");
    }
}
