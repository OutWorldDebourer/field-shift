//! Technical sheet sections added in iteration 4 (phase E): the farmer's proposal ("What if I grow…?"),
//! Prevention (pest risk lights with their reasons) and Biostimulants & micronutrients. Same pattern as the rest of
//! the sheet: a short summary, then the detail with sources; nothing without a citation; no product names or doses.
use crate::html::{bi, esc, num};
use serde_json::Value;

fn cap(s: &str) -> String { let mut c = s.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }

/// Italicises binomial species names inside parentheses ("(Sargassum vulgare, S. vulgare)") of already-escaped text.
fn species(s: &str) -> String {
    let binomial = |w: &str| {
        let mut it = w.split_whitespace();
        let (Some(g), Some(sp), None) = (it.next(), it.next(), it.next()) else { return false };
        let gen_ok = g.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && (g.ends_with('.') && g.len() == 2 || g.chars().skip(1).all(|c| c.is_ascii_lowercase()));
        gen_ok && sp.len() > 2 && sp.chars().all(|c| c.is_ascii_lowercase())
    };
    let mut out = String::new();
    let mut rest = s;
    while let Some(o) = rest.find('(') {
        let Some(cl) = rest[o..].find(')') else { break };
        out.push_str(&rest[..=o]);
        let inner = &rest[o + 1..o + cl];
        out.push_str(&inner.split(", ").map(|w| if binomial(w) { format!("<i>{w}</i>") } else { w.to_string() }).collect::<Vec<_>>().join(", "));
        out.push(')');
        rest = &rest[o + cl + 1..];
    }
    out.push_str(rest);
    out
}

fn t(es: bool, en: &str, es_: &str) -> String { if es { es_.to_string() } else { en.to_string() } }

fn cite(c: &Value) -> String {
    match c["url"].as_str().filter(|u| u.starts_with("http")) {
        Some(u) => format!("<a href=\"{}\">{}</a>", esc(u), esc(c["title"].as_str().unwrap_or(u))),
        None => String::new(),
    }
}

/// "Your proposal": the scored column, with why, evidence, gaps and risks.
pub fn proposal(rot: &Value, lang: &str) -> String {
    let es = lang == "es";
    let Some(p) = rot.get("proposal").filter(|p| p.is_object()) else { return String::new() };
    let sc = &p["scores"];
    let prov = |k: &str| p["provisional"].as_array().is_some_and(|a| a.iter().any(|x| x == k));
    let dots = |k: &str| { let n = sc[k].as_u64().unwrap_or(0).min(5) as usize; format!("<span role=\"img\" aria-label=\"{n} / 5\">{}{}</span>", "●".repeat(n), "○".repeat(5 - n)) };
    let row = |k: &str, en: &str, es_: &str| format!("<span>{}{}</span>{}", t(es, en, es_),
        if prov(k) { format!(" <span class=\"badge\">{}</span>", t(es, "provisional", "provisional")) } else { String::new() }, dots(k));
    let list = |k: &str| p[k].as_array().into_iter().flatten().map(|x| format!("<li>{}</li>", esc(&cap(&bi(if x.is_object() && x.get("text").is_some() { &x["text"] } else { x }, lang))))).collect::<String>();
    let mut cites: Vec<String> = p["citations"].as_array().into_iter().flatten().map(cite).filter(|s| !s.is_empty()).collect();
    cites.dedup(); cites.sort(); cites.dedup();
    format!("<h2>{}</h2><p>{}</p><p class=\"cite\">{} · {}</p><div class=\"scores prop\">{}{}{}{}{}</div>\
             <p><b>{}</b> {}</p><p><b>{}</b></p><ul class=\"gaps\">{}</ul><p><b>{}</b></p><ul class=\"gaps\">{}</ul><p><b>{}</b></p><ul class=\"gaps\">{}</ul>{}",
        esc(&bi(&p["name"], lang)),
        t(es, "The farmer's own idea, scored by the same engine and the same NASA signals as the strategies above (5 = best; for risk, 5 = lowest). A perennial is evaluated as a plantation, never as a rotation.",
              "La idea del propio agricultor, puntuada con el mismo motor y las mismas señales NASA que las estrategias de arriba (5 = mejor; en riesgo, 5 = menor). Un perenne se evalúa como plantación, nunca como rotación."),
        t(es, "Criterion", "Criterio"), t(es, "score out of 5, ● = 1", "puntaje sobre 5, ● = 1"),
        row("soil", "Soil health", "Salud del suelo"), row("water", "Water use", "Uso de agua"), row("pest_break", "Pest cycle break", "Corte del ciclo de plagas"),
        row("income", "Extra income", "Ingreso extra"), row("risk", "Low risk", "Bajo riesgo"),
        t(es, "Why:", "Por qué:"), esc(&cap(&bi(&p["why"], lang))),
        t(es, "Evidence (crop values from the cited sources below; field signals from the NASA readings table above, with their dates)", "Evidencia (valores del cultivo de las fuentes citadas abajo; señales del campo de la tabla de lecturas NASA de arriba, con sus fechas)"),
        list("evidence_list"), t(es, "Gaps", "Vacíos"), list("gaps_list"), t(es, "Risks", "Riesgos"), list("risks_list"),
        if cites.is_empty() { String::new() } else { format!("<p class=\"cite\">{} {}</p>", t(es, "Sources:", "Fuentes:"), cites.join(" · ")) })
}

/// Prevention: the risk light of every pest of every crop in the conversation, with its reason and source.
pub fn prevention(c: &Value, lang: &str) -> String {
    let es = lang == "es";
    let level = |l: &str| match (l, es) {
        ("high", false) => "High", ("high", true) => "Alto", ("medium", false) => "Medium", ("medium", true) => "Medio",
        ("low", false) => "Low", ("low", true) => "Bajo", (_, false) => "No data", (_, true) => "Sin datos",
    };
    let mut h = format!("<h2>{}</h2><p>{}</p><div class=\"notice\"><b>{}</b><p>{}</p></div>", esc(&bi(&c["title"], lang)), esc(&bi(&c["summary"], lang)),
        t(es, "PEST PRESSURE MODEL", "MODELO DE PRESIÓN DE PLAGAS"), esc(&bi(&c["notice"], lang)));
    // Signals used, each with its dates and scale (the reasons below refer to these).
    let mut sig: Vec<String> = vec![];
    if let Some(s) = c.get("signals").filter(|s| s.is_object()) {
        if !s["temp_period"].is_null() {
            sig.push(format!("{} {:+.1} °C · {} · NASA POWER <span class=\"badge\">{}</span>{}", t(es, "Temperature vs 1985–2024 normal, last 30 days:", "Temperatura vs normal 1985–2024, últimos 30 días:"),
                s["temp_anomaly"].as_f64().unwrap_or(0.0), esc(s["temp_period"].as_str().unwrap_or("")), t(es, "regional context", "contexto regional"),
                if s["provisional"] == true { format!(" <span class=\"badge\">{}</span>", t(es, "provisional", "provisional")) } else { String::new() }));
        }
        if !s["season"].is_null() {
            sig.push(format!("{} {} · NASA POWER <span class=\"badge\">{}</span>", t(es, "Degree-day generations counted over the season window", "Generaciones por grados-día contadas en la ventana de campaña"), esc(s["season"].as_str().unwrap_or("")), t(es, "regional context", "contexto regional")));
        }
    }
    if let Some(f) = c.get("forecast").filter(|f| f.is_object()) {
        sig.push(format!("{} {} {} ({} – {}): {} {} °C ({}), {} mm · NASA GEOS-FP ~25 km <span class=\"badge\">{}</span>",
            t(es, "Forecast, next", "Pronóstico, próximos"), num(&f["days"]), t(es, "days", "días"), esc(f["from"].as_str().unwrap_or("")), esc(f["to"].as_str().unwrap_or("")),
            t(es, "highest", "máxima"), num(&f["tmax"]), esc(f["tmax_date"].as_str().unwrap_or("")), num(&f["rain_mm"]), t(es, "regional context", "contexto regional")));
    }
    if !sig.is_empty() { h.push_str(&format!("<p><b>{}</b></p><ul class=\"gaps\">{}</ul>", t(es, "Signals used", "Señales usadas"), sig.iter().map(|s| format!("<li>{s}</li>")).collect::<String>())); }
    for b in c["crops"].as_array().into_iter().flatten() {
        let prov = if b["reviewed"] == true { String::new() } else if b["researched"] == true { t(es, " · AI-researched, verify", " · investigado por IA, verificar") } else { t(es, " · not researched yet", " · aún no investigado") };
        h.push_str(&format!("<h3>{}{}</h3>{}<div class=\"scroll\"><table class=\"pvt\"><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
            esc(&cap(&bi(&b["crop"]["name"], lang))), prov,
            if b["note"].is_object() { format!("<p class=\"cite\">{}</p>", esc(&bi(&b["note"], lang))) } else { String::new() },
            t(es, "Pest", "Plaga"), t(es, "Risk", "Riesgo"), t(es, "Why", "Por qué"), t(es, "Monitoring and source", "Vigilancia y fuente")));
        for p in b["pests"].as_array().into_iter().flatten() {
            let l = &p["light"];
            let lv = l["level"].as_str().unwrap_or("unknown");
            h.push_str(&format!("<tr><td>{} <i>{}</i></td><td><span class=\"lvl {lv}\">{}</span></td><td>{}</td><td>{} <span class=\"cite\">{}</span></td></tr>",
                esc(&bi(&p["name"], lang)), esc(p["scientific"].as_str().unwrap_or("")), level(lv),
                esc(&bi(&l["reason"], lang)), esc(&bi(&p["monitoring"], lang)), cite(&l["citation"])));
        }
        h.push_str("</tbody></table></div>");
    }
    let src: Vec<String> = c["sources"].as_array().into_iter().flatten().filter_map(|s| {
        let url = s["url"].as_str().filter(|u| !u.is_empty())?;
        let name = if s["name"].is_object() { bi(&s["name"], lang) } else { s["name"].as_str().unwrap_or(url).to_string() };
        Some(format!("<a href=\"{}\">{}</a>", esc(url), esc(&name)))
    }).collect();
    if !src.is_empty() { h.push_str(&format!("<p class=\"cite\">{} {} {}</p>", t(es, "Sources:", "Fuentes:"), src.join(" · "), t(es, "· favourable weather: the source linked in each row", "· clima favorable: la fuente enlazada en cada fila"))); }
    h
}

/// Biostimulants & micronutrients from the Organic section (phase-D crop research), or the declared gap.
pub fn biostim(org: &Value, lang: &str) -> String {
    let es = lang == "es";
    let title = t(es, "Biostimulants and micronutrients", "Bioestimulantes y micronutrientes");
    let list = org["biostim"].as_array().filter(|l| !l.is_empty());
    let Some(list) = list else {
        let gap = bi(&org["biostim_gap"], lang);
        return if gap.is_empty() { String::new() } else { format!("<h2>{title}</h2><p>{}</p>", esc(&gap)) };
    };
    let mut h = format!("<h2>{title}</h2><div class=\"notice\"><b>{}</b><p>{}</p></div><ul class=\"gaps\">", t(es, "NO PRODUCTS OR DOSES", "SIN PRODUCTOS NI DOSIS"),
        t(es, "Product names and doses are your agronomist's call, ideally with a soil or leaf test. Items researched by AI and not yet reviewed say so.",
        "Los nombres de productos y las dosis los decide tu agrónomo, idealmente con un análisis de suelo o de hoja. Lo investigado por IA y aún no revisado lo dice."));
    for b in list {
        h.push_str(&format!("<li><b>{}</b>{} — {}{} <span class=\"cite\">{}</span></li>",
            species(&esc(&bi(&b["name"], lang))),
            if b["reviewed"] == false { t(es, " (AI-researched, verify)", " (investigado por IA, verificar)") } else { String::new() },
            species(&esc(&bi(&b["why"], lang))),
            if b["signal"].is_object() { format!(" <i>{}</i>", esc(&bi(&b["signal"], lang))) } else { String::new() },
            cite(&b["citation"])));
    }
    h.push_str("</ul>");
    h
}

#[cfg(test)]
mod tests {
    #[test]
    fn species_in_parentheses_are_italic() {
        assert_eq!(super::species("Brown seaweed extracts (Sargassum vulgare, S. vulgare, Padina)"),
                   "Brown seaweed extracts (<i>Sargassum vulgare</i>, <i>S. vulgare</i>, Padina)");
        assert_eq!(super::species("Zinc (Zn)"), "Zinc (Zn)");
    }
}
