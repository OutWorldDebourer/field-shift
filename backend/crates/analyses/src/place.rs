//! Place name for a marked field: reverse geocoding with OpenStreetMap Nominatim (ODbL, attribution in the UI), so
//! the consultant names the real town, region and country instead of whatever label the client last held.
use serde_json::Value;
use std::time::Duration;

/// "Virú, La Libertad, Perú" in the analysis language, or None when the lookup fails or times out.
pub async fn reverse(http: &reqwest::Client, lat: f64, lon: f64, lang: &str) -> Option<String> {
    let url = format!("https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat={lat:.5}&lon={lon:.5}&zoom=12&addressdetails=1");
    let v: Value = http.get(url).header("Accept-Language", if lang == "es" { "es" } else { "en" })
        .timeout(Duration::from_secs(6)).send().await.ok()?.error_for_status().ok()?.json().await.ok()?;
    let a = &v["address"];
    let pick = |keys: &[&str]| keys.iter().find_map(|k| a[*k].as_str().map(str::to_string));
    let local = pick(&["village", "town", "city", "municipality", "hamlet", "suburb", "city_district", "county"]);
    let region = pick(&["state", "region", "province"]);
    let country = pick(&["country"]);
    let mut parts: Vec<String> = vec![];
    for p in [local, region, country].into_iter().flatten() {
        if !parts.iter().any(|x| x == &p) { parts.push(p); }
    }
    if parts.is_empty() { None } else { Some(parts.join(", ")) }
}
