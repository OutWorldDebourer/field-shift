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
            if a >= 1.0 { pts += 1; en.push(format!("hotter than normal ({a:+.1} °C over the last 30 days, NASA POWER, regional)")); es.push(format!("más cálido que lo normal ({a:+.1} °C en los últimos 30 días, NASA POWER, regional)")); }
        }
        if let Some(f) = fc {
            let fav_days = f.temps.iter().filter(|(lo, hi)| {
                let mean = (lo + hi) / 2.0;
                c.temp_min_c.is_none_or(|m| mean >= m) && c.temp_max_c.is_none_or(|m| mean <= m)
            }).count();
            // Only a narrow range means "favourable"; wide lab development limits (e.g. 6.5–37 °C) would match every day.
            let narrow = matches!((c.temp_min_c, c.temp_max_c), (Some(a), Some(b)) if b - a <= 16.0);
            if narrow && fav_days >= 3 {
                pts += 1;
                let (lo, hi) = (c.temp_min_c.unwrap_or(0.0), c.temp_max_c.unwrap_or(0.0));
                en.push(format!("{fav_days} of the next {} days within its favourable {lo:.0}–{hi:.0} °C (NASA GEOS-FP forecast)", f.days));
                es.push(format!("{fav_days} de los próximos {} días dentro de su rango favorable de {lo:.0}–{hi:.0} °C (pronóstico NASA GEOS-FP)", f.days));
            }
            match c.humidity.as_str() {
                "dry" if f.rain < 2.0 => { pts += 1; en.push("dry days ahead".into()); es.push("días secos por delante".into()); }
                "humid" if f.rain >= 10.0 => { pts += 1; en.push(format!("{:.0} mm of rain ahead", f.rain)); es.push(format!("{:.0} mm de lluvia por delante", f.rain)); }
                _ => {}
            }
        }
    }
    // Two agreeing signals for "medium", three for "high": one regional signal alone is "low — watch".
    let level = if !known { "unknown" } else if pts >= 3 { "high" } else if pts >= 2 { "medium" } else { "low" };
    if en.is_empty() {
        if known { en.push("no signal above normal".into()); es.push("ninguna señal sobre lo normal".into()); }
        else { en.push("no published threshold or weather data for this pest: watch it, but its risk cannot be estimated".into()); es.push("sin umbral publicado ni datos de clima para esta plaga: vigílala, pero su riesgo no se puede estimar".into()); }
    }
    let cap = |v: &[String]| { let s = v.join(", "); let mut c = s.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() };
    let (lead_en, lead_es) = match (level, pts) {
        ("high", _) => ("Three or more signals agree: ", "Tres o más señales coinciden: "),
        ("medium", _) => ("Two signals agree: ", "Dos señales coinciden: "),
        ("low", 1) => ("Only one signal is up, so it stays low (keep watching): ", "Solo una señal está alta, así que queda en bajo (sigue vigilando): "),
        _ => ("", ""),
    };
    json!({ "level": level, "points": pts, "reason": bi(format!("{lead_en}{}.", if lead_en.is_empty() { cap(&en) } else { en.join(", ") }), format!("{lead_es}{}.", if lead_es.is_empty() { cap(&es) } else { es.join(", ") })),
            "favourable": cond.map(|c| c.favourable.clone()), "citation": cond.map(|c| json!({ "title": c.citation.title, "url": c.citation.url })) })
}

fn frequency(level: &str) -> (&'static str, &'static str) {
    match level {
        "high" => ("Check twice a week", "Revisa dos veces por semana"),
        "medium" => ("Check once a week", "Revisa una vez por semana"),
        "low" => ("Check every two weeks", "Revisa cada dos semanas"),
        _ => ("Check every week while you learn its pattern", "Revisa cada semana mientras conoces su comportamiento"),
    }
}

/// Where to look, one cultural practice and when to bring in natural enemies, by kind of pest (general extension
/// practice, no products or doses). Matched on the common and scientific names.
fn actions(level: &str, name_en: &str, sci: &str) -> Value {
    let k = format!("{} {}", name_en.to_lowercase(), sci.to_lowercase());
    let has = |w: &[&str]| w.iter().any(|x| k.contains(x));
    let (look, practice): ((&str, &str), (&str, &str)) = if has(&["aphid", "myzus", "acyrthosiphon", "aphis"]) {
        (("the underside of young leaves and the growing tips, for colonies and curled leaves", "el envés de las hojas tiernas y los brotes, buscando colonias y hojas enrolladas"),
         ("keep flowering borders for lady beetles and hoverflies; avoid excess nitrogen, which makes tender growth they prefer", "mantén bordes con flores para mariquitas y sírfidos; evita el exceso de nitrógeno, que da brotes tiernos que prefieren"))
    } else if has(&["leafminer", "liriomyza", "minador"]) {
        (("leaves for winding tunnels and fine puncture marks; yellow sticky traps show the flies", "las hojas, buscando galerías serpenteantes y picaduras finas; las trampas amarillas muestran las moscas"),
         ("remove and destroy heavily mined lower leaves; incorporate residues soon after harvest", "retira y destruye las hojas bajas muy minadas; incorpora los rastrojos pronto tras la cosecha"))
    } else if has(&["whitefl", "bemisia", "mosca blanca"]) {
        (("the underside of leaves for tiny white adults and scale-like nymphs; yellow sticky traps", "el envés de las hojas, buscando adultos blancos diminutos y ninfas; trampas amarillas"),
         ("clear weeds and old crop plants that host it between seasons", "elimina malezas y plantas viejas que la hospedan entre campañas"))
    } else if has(&["cutworm", "agrotis", "grub", "cyclocephala", "gusano de tierra"]) {
        (("the soil around the stem base at dusk, and plants cut at ground level", "el suelo alrededor del tallo al atardecer, y plantas cortadas a ras del suelo"),
         ("prepare the soil well before sowing and keep the field free of weeds before planting", "prepara bien el suelo antes de sembrar y mantén el campo sin malezas antes de plantar"))
    } else if has(&["weevil", "euscepes", "cylas", "tuber moth", "phthorimaea"]) {
        (("tubers or roots near the surface and cracks in the soil; damaged storage roots", "tubérculos o raíces cerca de la superficie y grietas del suelo; raíces dañadas"),
         ("hill up the soil and irrigate to close cracks; use clean planting material; harvest on time", "aporca e irriga para cerrar grietas; usa semilla limpia; cosecha a tiempo"))
    } else if has(&["armyworm", "spodoptera", "looper", "trichoplusia", "earworm", "helicoverpa", "borer", "diatraea", "caterpillar"]) {
        (("leaves, whorls and ears for holes, fresh frass and young larvae", "hojas, cogollos y mazorcas, buscando agujeros, excremento fresco y larvas pequeñas"),
         ("keep borders and weeds managed; plough in residues after harvest to expose pupae", "maneja bordes y malezas; incorpora los rastrojos tras la cosecha para exponer las pupas"))
    } else if has(&["mite", "tetranych", "oligonychus", "ácaro", "arañita"]) {
        (("the underside of leaves for fine stippling, bronzing and webbing", "el envés de las hojas, buscando punteado, bronceado y telarañas finas"),
         ("avoid water stress and dusty field edges, which favour mites", "evita el estrés hídrico y los bordes polvorientos, que favorecen a los ácaros"))
    } else if has(&["scale", "mealybug", "cochinilla", "fiorinia", "planococcus"]) {
        (("stems, bark and the underside of leaves for waxy or cottony insects", "tallos, corteza y envés de las hojas, buscando insectos cerosos o algodonosos"),
         ("prune and destroy heavily infested shoots; control ants that protect them", "poda y destruye brotes muy infestados; controla las hormigas que las protegen"))
    } else {
        (("leaves, stems and fruit for insects or damage", "hojas, tallos y frutos, buscando insectos o daños"),
         ("keep residues and weeds managed between seasons", "maneja rastrojos y malezas entre campañas"))
    };
    let (f_en, f_es) = frequency(level);
    json!({
        "look": bi(format!("{f_en}: {}.", look.0), format!("{f_es}: {}.", look.1)),
        "practice": bi(format!("Cultural practice: {}.", practice.0), format!("Práctica cultural: {}.", practice.1)),
        "release": bi("Release or protect its natural enemies when you find the first colonies or larvae, before damage spreads (see Organic).",
                      "Libera o protege a sus enemigos naturales cuando encuentres las primeras colonias o larvas, antes de que el daño se extienda (ver Orgánico)."),
    })
}

/// A species' key across crops ("Liriomyza huidobrensis" on pea and on lettuce is the same pest).
fn species_key(sci: &str) -> String { sci.split_whitespace().take(2).collect::<Vec<_>>().join(" ").to_lowercase().replace(['(', ')', ','], "") }

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
    let mut counts = (0, 0, 0, 0);
    // One name and one set of researched conditions per species, taken from the first crop (current first) that has them.
    let mut names: std::collections::HashMap<String, crops::model::L10n> = Default::default();
    let mut conds: std::collections::HashMap<String, crops::model::PestConditions> = Default::default();
    for (c, _) in crops {
        for p in &c.traits.pests {
            let k = species_key(&p.scientific_name);
            names.entry(k.clone()).or_insert_with(|| p.name.clone());
            if let Some(cd) = &p.conditions { conds.entry(k).or_insert_with(|| cd.clone()); }
        }
    }
    let mut seen_on: std::collections::HashMap<String, Vec<String>> = Default::default();
    for (c, _) in crops { for p in &c.traits.pests { seen_on.entry(species_key(&p.scientific_name)).or_default().push(c.name_en().to_string()); } }
    for (c, role) in crops {
        let mut pests = vec![];
        for p in &c.traits.pests {
            let key = species_key(&p.scientific_name);
            let name = names.get(&key).cloned().unwrap_or_else(|| p.name.clone());
            let cond = conds.get(&key).or(p.conditions.as_ref());
            let also: Vec<String> = seen_on.get(&key).into_iter().flatten().filter(|n| n.as_str() != c.name_en()).cloned().collect();
            let row = rows.iter().find(|r| r["scientific"].as_str() == Some(p.scientific_name.as_str()) && r["crop"]["key"].as_str() == Some(c.key.as_str())).cloned().unwrap_or(Value::Null);
            let l = light(&row, cond, anomaly, fc.as_ref());
            match l["level"].as_str() { Some("high") => counts.0 += 1, Some("medium") => counts.1 += 1, Some("low") => counts.2 += 1, _ => counts.3 += 1 }
            let act = actions(l["level"].as_str().unwrap_or(""), &name.en, &p.scientific_name);
            pests.push(json!({
                "name": name, "scientific": p.scientific_name, "light": l,
                "generations_now": row["generations_now"], "generations_normal": row["generations_normal"], "season": row["season"],
                "monitoring": act["look"], "practice": act["practice"], "release": act["release"],
                "also_on": also,
            }));
        }
        pests.sort_by_key(|p| match p["light"]["level"].as_str() { Some("high") => 0, Some("medium") => 1, Some("low") => 2, _ => 3 });
        let watch: Vec<Value> = pests.iter().filter(|p| matches!(p["light"]["level"].as_str(), Some("high" | "medium"))).cloned().collect();
        out_crops.push(json!({
            "crop": { "key": c.key, "name": c.name(), "crop_type": c.crop_type, "role": format!("{role:?}").to_lowercase() },
            "pests": pests, "calendar": calendar(c, if watch.is_empty() { &pests } else { &watch }),
            "researched": c.traits.extras_reviewed.is_some(), "reviewed": c.traits.extras_reviewed == Some(true),
            "note": match role {
                crate::pests::Role::Previous => bi("Grown before: its pests can survive on residues and volunteer plants, and some also attack your current crop.", "Cultivo anterior: sus plagas pueden sobrevivir en rastrojos y plantas voluntarias, y algunas también atacan tu cultivo actual."),
                crate::pests::Role::Proposal => bi("Your proposal: how its pests would fare under this season's weather.", "Tu propuesta: cómo les iría a sus plagas con el clima de esta campaña."),
                crate::pests::Role::Next => bi("Planned next: what its pests would meet now.", "Próximo cultivo: lo que encontrarían sus plagas ahora."),
                _ => Value::Null,
            },
        }));
    }
    let fc_v = fc.as_ref().map(|f| json!({ "tmax": f.tmax, "tmax_date": f.tmax_date, "tmin": f.tmin, "rain_mm": (f.rain * 10.0).round() / 10.0, "days": f.days,
        "source": ev["forecast"]["source"], "run": ev["forecast"]["run"], "badge": "regional context" }));
    json!({
        "title": bi("Prevention: pest risk on your field", "Prevención: riesgo de plagas en tu campo"),
        "summary": bi(format!("{} high, {} medium, {} low and {} without data, from NASA degree-days, the 10-day forecast and the weather each pest likes.", counts.0, counts.1, counts.2, counts.3),
                      format!("{} altos, {} medios, {} bajos y {} sin datos, según grados-día NASA, el pronóstico de 10 días y el clima que favorece a cada plaga.", counts.0, counts.1, counts.2, counts.3)),
        "counts": { "high": counts.0, "medium": counts.1, "low": counts.2, "unknown": counts.3 },
        "crops": out_crops, "forecast": fc_v, "temp_anomaly": anomaly,
        "notice": bi("A model of pest pressure, not a count of insects: confirm in the field and with your agronomist.", "Un modelo de presión de plagas, no un conteo de insectos: confírmalo en el campo y con tu agrónomo."),
        "sources": [
            { "name": bi("NASA POWER daily temperature (degree-days), regional ~50 km", "Temperatura diaria NASA POWER (grados-día), regional ~50 km"), "url": "https://power.larc.nasa.gov/" },
            { "name": bi("NASA GMAO GEOS-FP forecast, regional ~25 km", "Pronóstico NASA GMAO GEOS-FP, regional ~25 km"), "url": "https://gmao.gsfc.nasa.gov/GMAO_products/NRT_products.php" },
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
        assert!(r.contains("hotter than normal (+5.0 °C") && r.contains("dry days ahead") && r.starts_with("Three or more signals agree"), "{r}");
    }

    #[test]
    fn wide_lab_limits_do_not_count_as_favourable() {
        let fc = Fc { tmax: 25.0, tmax_date: "d".into(), tmin: 15.0, rain: 5.0, days: 10, temps: vec![(15.0, 25.0); 10] };
        let l = light(&json!({ "generations_now": 5.0, "generations_normal": 5.0 }), Some(&cond(Some(6.5), Some(37.3), "any")), Some(0.0), Some(&fc));
        assert_eq!(l["level"], "low");
    }

    #[test]
    fn no_data_is_unknown_not_low() {
        assert_eq!(light(&Value::Null, None, Some(5.0), None)["level"], "unknown");
        let row = json!({ "generations_now": 5.0, "generations_normal": 5.1 });
        assert_eq!(light(&row, None, None, None)["level"], "low");
        // One regional signal alone is "low — watch", not medium.
        assert_eq!(light(&row, Some(&cond(None, None, "any")), Some(5.0), None)["level"], "low");
    }
}
