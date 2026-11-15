//! The Prevention section (iteration 4, phase D): for every crop in the conversation (current, previous, planned,
//! proposed) each pest gets a traffic light (low / medium / high) with a plain reason, from three signals:
//! NASA POWER degree-day generations vs the 1985–2024 normal, the NASA GEOS-FP 10-day forecast, and the weather that
//! favours the pest (cited crop research, phase D). Perennials get a year-round monitoring calendar, annuals a
//! calendar by stage. Actions: monitoring frequency, cultural practices, when to release natural enemies (Organic).
//! A pre-diagnosis: risk lights are a model of pressure, never a count of insects.
use crops::CropRecord;
use serde_json::{json, Value};

fn bi(en: impl Into<String>, es: impl Into<String>) -> Value { json!({ "en": en.into(), "es": es.into() }) }

/// Forecast facts the lights use: hottest day, lowest night, rain, and the dates.
struct Fc { tmax: f64, tmax_date: String, tmin: f64, rain: f64, days: usize, temps: Vec<(f64, f64)> }

fn forecast(ev: &Value) -> Option<Fc> {
    let d = ev["forecast"]["days"].as_array().filter(|d| !d.is_empty())?;
    let hot = d.iter().max_by(|a, b| a["tmax"].as_f64().unwrap_or(f64::MIN).partial_cmp(&b["tmax"].as_f64().unwrap_or(f64::MIN)).unwrap_or(std::cmp::Ordering::Equal))?;
    Some(Fc {
        tmax: hot["tmax"].as_f64()?, tmax_date: hot["date"].as_str().unwrap_or("").to_string(),
        tmin: d.iter().filter_map(|x| x["tmin"].as_f64()).fold(f64::MAX, f64::min),
        rain: d.iter().filter_map(|x| x["rain_mm"].as_f64()).sum(), days: d.len(),
        temps: d.iter().filter_map(|x| Some((x["tmin"].as_f64()?, x["tmax"].as_f64()?))).collect(),
    })
}

/// One pest's light: points from each signal, with the matching reason words.
fn light(row: &Value, cond: Option<&crops::model::PestConditions>, anomaly: Option<f64>, fc: Option<&Fc>) -> Value {
    let mut pts = 0; let mut en: Vec<String> = vec![]; let mut es: Vec<String> = vec![];
    let mut known = false;
    if let (Some(now), Some(norm)) = (row["generations_now"].as_f64(), row["generations_normal"].as_f64()) {
        known = true;
        let d = now - norm;
        if d >= 2.0 { pts += 2 } else if d >= 0.8 { pts += 1 }
        if d >= 0.8 { en.push(format!("{d:.1} more generations than normal this season")); es.push(format!("{d:.1} generaciones más que lo normal esta campaña")); }
        else if d <= -0.8 { en.push(format!("{:.1} fewer generations than normal", -d)); es.push(format!("{:.1} generaciones menos que lo normal", -d)); }
    }
    if let Some(c) = cond {
        known = true;
        let heat_lover = c.temp_min_c.is_some_and(|t| t >= 20.0) || c.favourable.en.to_lowercase().contains("hot") || c.favourable.en.to_lowercase().contains("warm");
        if let (Some(a), true) = (anomaly, heat_lover) {
            if a >= 1.0 { pts += 1; en.push(format!("hotter than normal ({a:+.1} °C)")); es.push(format!("más cálido que lo normal ({a:+.1} °C)")); }
        }
        if let Some(f) = fc {
            let fav_days = f.temps.iter().filter(|(lo, hi)| {
                let mean = (lo + hi) / 2.0;
                c.temp_min_c.is_none_or(|m| mean >= m) && c.temp_max_c.is_none_or(|m| mean <= m)
            }).count();
            if (c.temp_min_c.is_some() || c.temp_max_c.is_some()) && fav_days >= 3 {
                pts += 1;
                en.push(format!("{fav_days} of the next {} days in its favourable range (up to {:.0} °C)", f.days, f.tmax));
                es.push(format!("{fav_days} de los próximos {} días en su rango favorable (hasta {:.0} °C)", f.days, f.tmax));
            }
            match c.humidity.as_str() {
                "dry" if f.rain < 2.0 => { pts += 1; en.push("dry days ahead".into()); es.push("días secos por delante".into()); }
                "humid" if f.rain >= 10.0 => { pts += 1; en.push(format!("{:.0} mm of rain ahead", f.rain)); es.push(format!("{:.0} mm de lluvia por delante", f.rain)); }
                _ => {}
            }
        }
    }
    let level = if !known { "unknown" } else if pts >= 3 { "high" } else if pts >= 1 { "medium" } else { "low" };
    if en.is_empty() {
        if known { en.push("no signal above normal".into()); es.push("ninguna señal sobre lo normal".into()); }
        else { en.push("no published threshold or weather data for this pest: watch it, but its risk cannot be estimated".into()); es.push("sin umbral publicado ni datos de clima para esta plaga: vigílala, pero su riesgo no se puede estimar".into()); }
    }
    let cap = |v: &[String]| { let s = v.join(", "); let mut c = s.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() };
    json!({ "level": level, "points": pts, "reason": bi(cap(&en) + ".", cap(&es) + "."),
            "favourable": cond.map(|c| c.favourable.clone()), "citation": cond.map(|c| json!({ "title": c.citation.title, "url": c.citation.url })) })
}

fn monitoring(level: &str) -> Value {
    match level {
        "high" => bi("Check twice a week: underside of young leaves, growing tips and field edges.", "Revisa dos veces por semana: envés de hojas tiernas, brotes y bordes del campo."),
        "medium" => bi("Check once a week, more often in warm spells.", "Revisa una vez por semana, más seguido en días cálidos."),
        "low" => bi("Check every two weeks.", "Revisa cada dos semanas."),
        _ => bi("Check every week while you learn its pattern.", "Revisa cada semana mientras conoces su comportamiento."),
    }
}

/// Calendar: perennials all year (months), annuals by stage (no sowing date in the data: stages without dates).
fn calendar(c: &CropRecord, watch: &[Value]) -> Value {
    let names: Vec<Value> = watch.iter().map(|p| p["name"].clone()).collect();
    if c.crop_type == crops::CropType::Perennial {
        return json!({ "kind": "year-round", "note": bi("A perennial is watched all year; check monthly at least, weekly in warm months.", "Un perenne se vigila todo el año; revisa al menos cada mes y cada semana en los meses cálidos."), "watch": names });
    }
    let stage = |en: &str, es: &str, act_en: &str, act_es: &str| json!({ "stage": bi(en, es), "action": bi(act_en, act_es), "watch": names });
    json!({ "kind": "by-stage", "note": bi("Sowing date not given: stages are shown without dates.", "Sin fecha de siembra: las etapas se muestran sin fechas."),
        "stages": [
            stage("Sowing and establishment", "Siembra y establecimiento", "Clean field edges and residues; check seedlings twice a week.", "Limpia bordes y rastrojos; revisa las plántulas dos veces por semana."),
            stage("Vegetative growth", "Crecimiento vegetativo", "Weekly checks of young leaves; keep flowering borders for natural enemies.", "Revisión semanal de hojas tiernas; mantén bordes con flores para los enemigos naturales."),
            stage("Flowering and fruit set", "Floración y cuajado", "Most sensitive stage: check twice a week and protect pollinators and allies.", "Etapa más sensible: revisa dos veces por semana y protege polinizadores y aliados."),
            stage("Harvest and after", "Cosecha y después", "Incorporate or compost residues soon; do not pile them at the edge.", "Incorpora o composta los rastrojos pronto; no los amontones en el borde."),
        ] })
}

/// Builds the section content for the analysis's crops and its evidence package.
pub fn build(crops: &[(CropRecord, crate::pests::Role)], ev: &Value) -> Value {
    let anomaly = ev["readings"].as_array().and_then(|r| r.iter().find(|x| x["id"] == "temp_anomaly")).and_then(|x| x["value"].as_f64());
    let fc = forecast(ev);
    let rows = ev["pests"].as_array().cloned().unwrap_or_default();
    let mut out_crops = vec![];
    let mut counts = (0, 0, 0);
    for (c, role) in crops {
        let mut pests = vec![];
        for p in &c.traits.pests {
            let row = rows.iter().find(|r| r["scientific"].as_str() == Some(p.scientific_name.as_str()) && r["crop"]["key"].as_str() == Some(c.key.as_str())).cloned().unwrap_or(Value::Null);
            let l = light(&row, p.conditions.as_ref(), anomaly, fc.as_ref());
            match l["level"].as_str() { Some("high") => counts.0 += 1, Some("medium") => counts.1 += 1, Some("low") => counts.2 += 1, _ => {} }
            pests.push(json!({
                "name": p.name, "scientific": p.scientific_name, "light": l,
                "generations_now": row["generations_now"], "generations_normal": row["generations_normal"],
                "monitoring": monitoring(l["level"].as_str().unwrap_or("")),
            }));
        }
        pests.sort_by_key(|p| match p["light"]["level"].as_str() { Some("high") => 0, Some("medium") => 1, Some("low") => 2, _ => 3 });
        let watch: Vec<Value> = pests.iter().filter(|p| matches!(p["light"]["level"].as_str(), Some("high" | "medium"))).cloned().collect();
        out_crops.push(json!({
            "crop": { "key": c.key, "name": c.name(), "crop_type": c.crop_type, "role": format!("{role:?}").to_lowercase() },
            "pests": pests, "calendar": calendar(c, if watch.is_empty() { &pests } else { &watch }),
            "researched": c.traits.extras_reviewed.is_some(), "reviewed": c.traits.extras_reviewed == Some(true),
        }));
    }
    let fc_v = fc.as_ref().map(|f| json!({ "tmax": f.tmax, "tmax_date": f.tmax_date, "tmin": f.tmin, "rain_mm": (f.rain * 10.0).round() / 10.0, "days": f.days,
        "source": ev["forecast"]["source"], "run": ev["forecast"]["run"], "badge": "regional context" }));
    json!({
        "title": bi("Prevention: pest risk on your field", "Prevención: riesgo de plagas en tu campo"),
        "summary": bi(format!("{} high, {} medium, {} low risk lights from NASA degree-days, the 10-day forecast and the weather each pest likes.", counts.0, counts.1, counts.2),
                      format!("{} altos, {} medios y {} bajos, según grados-día NASA, el pronóstico de 10 días y el clima que favorece a cada plaga.", counts.0, counts.1, counts.2)),
        "crops": out_crops, "forecast": fc_v, "temp_anomaly": anomaly,
        "notice": bi("A model of pest pressure, not a count of insects: confirm in the field and with your agronomist.", "Un modelo de presión de plagas, no un conteo de insectos: confírmalo en el campo y con tu agrónomo."),
        "sources": [
            { "name": "NASA POWER daily temperature (degree-days), regional ~50 km", "url": "https://power.larc.nasa.gov/" },
            { "name": "NASA GMAO GEOS-FP forecast, regional ~25 km", "url": "https://gmao.gsfc.nasa.gov/GMAO_products/NRT_products.php" },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cond(min: Option<f64>, max: Option<f64>, hum: &str) -> crops::model::PestConditions {
        crops::model::PestConditions { favourable: crops::model::L10n::new("hot, dry weather", "clima cálido y seco"), temp_min_c: min, temp_max_c: max,
            humidity: hum.into(), season: None, citation: crops::model::Citation { title: "t".into(), author: "a".into(), year: None, url: "https://x.org".into() } }
    }

    #[test]
    fn spider_mite_in_a_hot_dry_spell_is_high() {
        let fc = Fc { tmax: 30.0, tmax_date: "2026-11-15".into(), tmin: 18.0, rain: 0.4, days: 10, temps: vec![(20.0, 30.0); 10] };
        let row = json!({ "generations_now": 8.0, "generations_normal": 5.5 });
        let l = light(&row, Some(&cond(Some(22.0), Some(35.0), "dry")), Some(5.0), Some(&fc));
        assert_eq!(l["level"], "high");
        let r = l["reason"]["en"].as_str().unwrap();
        assert!(r.contains("hotter than normal (+5.0 °C)") && r.contains("dry days ahead"), "{r}");
    }

    #[test]
    fn no_data_is_unknown_not_low() {
        assert_eq!(light(&Value::Null, None, Some(5.0), None)["level"], "unknown");
        let row = json!({ "generations_now": 5.0, "generations_normal": 5.1 });
        assert_eq!(light(&row, None, None, None)["level"], "low");
    }
}
