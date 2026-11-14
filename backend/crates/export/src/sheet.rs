//! The technical sheet for the agronomist: printable (A4) and readable on a phone. Field, crops, every reading with
//! source, date, resolution and scale badge (licences listed once), cited sections, every information gap, the notice.
use crate::html::{bi, esc, num};
use analyses::AnalysisState;
use serde_json::Value;
use std::collections::BTreeSet;

const CSS: &str = r#"
@font-face{font-family:"Source Serif 4";font-style:italic;font-weight:400;src:url(/fonts/source-serif-4-latin-400-italic.woff2) format("woff2")}
@font-face{font-family:"Source Sans 3";font-weight:400;src:url(/fonts/source-sans-3-latin-400-normal.woff2) format("woff2")}
@font-face{font-family:"Source Sans 3";font-weight:600;src:url(/fonts/source-sans-3-latin-600-normal.woff2) format("woff2")}
@font-face{font-family:"IBM Plex Mono";font-weight:400;src:url(/fonts/ibm-plex-mono-latin-400-normal.woff2) format("woff2")}
@font-face{font-family:"IBM Plex Mono";font-weight:500;src:url(/fonts/ibm-plex-mono-latin-500-normal.woff2) format("woff2")}
:root{--ink:#14202b;--muted:#46545f;--rule:#cfd8df;--amber:#a65f00;--amber-bg:#fff6e6;--blue:#2e70ce;--paper:#fff;--wash:#f2f6f9;--field:#1f6b3d}
*{box-sizing:border-box}
html{-webkit-text-size-adjust:100%}
body{margin:0;background:#e6ecf0;color:var(--ink);font:16px/1.55 "Source Sans 3",system-ui,sans-serif}
.page{max-width:860px;margin:0 auto 32px;background:var(--paper);padding:44px 56px;box-shadow:0 1px 3px rgba(0,0,0,.12)}
h1,h2{font-family:"Source Serif 4",Georgia,serif;font-style:italic;font-weight:400;margin:0}
h1{font-size:36px;line-height:1.1}
h2{font-size:24px;margin:36px 0 12px;padding-top:16px;border-top:1px solid var(--rule)}
h3{font-size:16px;font-weight:600;margin:16px 0 4px}
p{margin:6px 0 10px}
.mono{font-family:"IBM Plex Mono",ui-monospace,monospace}
.run{display:flex;justify-content:space-between;gap:12px;font:13px "IBM Plex Mono",monospace;color:var(--muted);text-transform:uppercase;letter-spacing:.06em;border-bottom:1px solid var(--ink);padding-bottom:8px;margin-bottom:18px}
.run{flex-wrap:wrap}.run span{white-space:nowrap}
.head{display:grid;grid-template-columns:1fr 132px;gap:24px;align-items:start}
.tile{width:132px;height:132px;border-radius:6px;display:block}
.notice{margin:20px 0;border:1.5px solid var(--ink);border-radius:6px;background:var(--wash);padding:12px 16px}
.notice b{display:block;font:500 13px "IBM Plex Mono",monospace;letter-spacing:.08em;color:var(--ink);margin-bottom:4px}
.notice p{margin:0;font-weight:600}
.meta{display:grid;grid-template-columns:repeat(4,1fr);gap:10px;margin-top:18px}
.meta div{background:var(--wash);padding:10px 12px;border-radius:6px}
.meta b{display:block;font:400 12px "IBM Plex Mono",monospace;color:var(--muted);text-transform:uppercase;letter-spacing:.05em}
.readings{display:grid;gap:0;border-top:1px solid var(--ink)}
.r{display:grid;grid-template-columns:minmax(0,24fr) minmax(0,15fr) minmax(0,31fr) minmax(0,30fr);gap:10px;padding:10px 0;border-bottom:1px solid var(--rule);align-items:start}
.r .lab{font-weight:600}
.r .val{font-weight:600}
.r .val.key{color:var(--amber)}
.r .src{font:13px/1.45 "IBM Plex Mono",monospace;color:var(--muted)}
.rh{font:400 12px "IBM Plex Mono",monospace;color:var(--muted);text-transform:uppercase;letter-spacing:.05em;border-bottom:0;padding-bottom:6px}
.badge{display:inline-block;font:12px "IBM Plex Mono",monospace;padding:1px 7px;border-radius:9px;border:1px solid #9aa7b1;white-space:nowrap;margin-top:4px;color:var(--muted)}
.badge.field{border-color:var(--ink);background:var(--ink);color:#fff}
.lic{font:13px/1.5 "IBM Plex Mono",monospace;color:var(--muted);margin-top:10px}
table{width:100%;border-collapse:collapse;font-size:15px}
th{font:400 12px "IBM Plex Mono",monospace;text-align:left;color:var(--muted);text-transform:uppercase;letter-spacing:.04em;border-bottom:1px solid var(--ink);padding:6px}
td{padding:8px 6px;border-bottom:1px solid var(--rule);vertical-align:top}
td.n{font-weight:600;white-space:nowrap}
.scroll{overflow-x:auto}
.cite{font-size:14px;color:var(--muted);overflow-wrap:anywhere}
td,.r>span,li,p{overflow-wrap:anywhere;min-width:0}
.cite a,.page a{color:var(--ink);text-decoration:underline;text-decoration-color:#9aa7b1}
.strat{display:grid;grid-template-columns:repeat(3,1fr);gap:12px;align-items:stretch}
.strat>div{border:1px solid var(--rule);border-radius:6px;padding:12px 14px}
.strat .cur{border-color:var(--blue);box-shadow:inset 0 3px 0 var(--blue)}
.strat h3{margin-top:0}
.tl{font:13px "IBM Plex Mono",monospace;color:var(--blue)}
.scores{display:grid;grid-template-columns:1fr auto;gap:2px 10px;font-size:14px;margin:10px 0}
.scores span:nth-child(even){font:14px "IBM Plex Mono",monospace;letter-spacing:1px}
.strat ul,.gaps{margin:4px 0 8px 18px;padding:0}
.strat li{font-size:14px;margin:2px 0}
.actions{max-width:860px;margin:16px auto;display:flex;gap:10px;justify-content:flex-end;padding:0 4px}
.actions button,.actions a{font:600 15px "Source Sans 3",sans-serif;padding:11px 18px;border-radius:6px;border:1px solid var(--ink);background:var(--ink);color:#fff;cursor:pointer;text-decoration:none;min-height:44px}
.actions a{background:#fff;color:var(--ink)}
footer{margin-top:28px;font-size:14px;color:var(--muted)}
@media (max-width:640px){
 .page{padding:20px 18px;margin:0}
 .actions{padding:0 18px;justify-content:stretch}.actions>*{flex:1;text-align:center}
 .head{grid-template-columns:1fr 84px}.tile{width:84px;height:84px}
 h1{font-size:30px}
 .meta{grid-template-columns:1fr 1fr}
 .r{grid-template-columns:1fr auto;gap:4px 12px}
 .r .lab{grid-column:1}.r .val{grid-column:2;text-align:right}
 .r .pw,.r .src{grid-column:1 / -1}
 .rh{display:none}
 .strat{grid-template-columns:1fr}
}
@media print{
 body{background:#fff;font-size:11pt}.page{box-shadow:none;margin:0;max-width:none;padding:0}.actions{display:none}
 h2{break-after:avoid}h3{break-after:avoid}.r,.strat>div,tr{break-inside:avoid}
 .cite a::after{content:" " attr(href);font:8pt "IBM Plex Mono",monospace;color:#555;word-break:break-all}
 .r .src,.cite,.lic,th{font-size:9.5pt}
 *{-webkit-print-color-adjust:exact;print-color-adjust:exact}
 @page{size:A4;margin:16mm 16mm 20mm;@bottom-center{content:"Field Shift · pre-diagnosis, not a diagnosis · page " counter(page);font:8pt "IBM Plex Mono",monospace;color:#555}}
}
"#;

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new() }
}

fn coord(lat: f64, lon: f64) -> String {
    format!("{:.4}°{} {:.4}°{}", lat.abs(), if lat < 0.0 { "S" } else { "N" }, lon.abs(), if lon < 0.0 { "W" } else { "E" })
}

/// Field outline as a small dark tile with the white hairline and the lock-on reticle (ties the sheet to the orbit view).
fn tile(ring: &[[f64; 2]]) -> String {
    let br = |x: f64, y: f64, dx: f64, dy: f64| format!("<path d=\"M{} {}h{}M{} {}v{}\" />", x, y, dx, x, y, dy);
    let brackets = [br(14.0, 14.0, 18.0, 18.0, ), br(118.0, 14.0, -18.0, 18.0), br(14.0, 118.0, 18.0, -18.0), br(118.0, 118.0, -18.0, -18.0)].join("");
    let poly = if ring.len() >= 3 {
        let (mut w, mut s, mut e, mut n) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for [x, y] in ring { w = w.min(*x); e = e.max(*x); s = s.min(*y); n = n.max(*y); }
        let k = (e - w).max(n - s).max(1e-9);
        let pts: Vec<String> = ring.iter().map(|[x, y]| format!("{:.1},{:.1}", 34.0 + (x - w) / k * 64.0, 98.0 - (y - s) / k * 64.0)).collect();
        format!("<polygon points=\"{}\" fill=\"rgba(255,255,255,.08)\" stroke=\"#fff\" stroke-width=\"1.4\" />", pts.join(" "))
    } else { "<circle cx=\"66\" cy=\"66\" r=\"4\" fill=\"#fff\" />".into() };
    format!("<svg class=\"tile\" viewBox=\"0 0 132 132\" role=\"img\" aria-label=\"Field outline · contorno del campo\"><rect width=\"132\" height=\"132\" rx=\"6\" fill=\"#16222c\" />{poly}<g stroke=\"#fff\" stroke-width=\"1.6\" fill=\"none\">{brackets}</g></svg>")
}

pub fn render(st: &AnalysisState, lang: &str, csv_url: &str) -> String {
    let es = lang == "es";
    let t = |en: &str, es_: &str| if es { es_.to_string() } else { en.to_string() };
    let ev = st.evidence.clone().unwrap_or(Value::Null);
    let crop = st.crop_current.as_ref().map(|c| {
        let ty = match (c["crop_type"].as_str().unwrap_or(""), es) {
            ("annual", true) => "anual", ("semi-perennial", true) => "semiperenne", ("perennial", true) => "perenne", (x, _) => x,
        }.to_string();
        format!("{} (<i>{}</i>) · {}", esc(&bi(&c["names"], lang)), esc(c["scientific"].as_str().unwrap_or("")), esc(&ty))
    }).unwrap_or_else(|| t("not stated", "no indicado"));
    let prev: Vec<String> = st.crop_history.iter().map(|c| bi(&c["names"], lang)).collect();
    let generated = ev["generated_at"].as_str().unwrap_or("").chars().take(10).collect::<String>();
    let label = if es && st.field.label == "Huaral valley, Peru" { "Valle de Huaral, Perú".to_string() } else { st.field.label.clone() };

    let mut h = String::with_capacity(48_000);
    h.push_str(&format!("<!doctype html><html lang=\"{lang}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{} · Field Shift</title><style>{CSS}</style></head><body>",
        t("Technical sheet", "Ficha técnica")));
    h.push_str(&format!("<div class=\"actions\"><a href=\"{}\">{}</a><button type=\"button\" onclick=\"print()\">{}</button></div>",
        esc(csv_url), t("Download CSV", "Descargar CSV"), t("Save as PDF / Print", "Guardar PDF / Imprimir")));
    h.push_str("<main class=\"page\">");
    h.push_str(&format!("<div class=\"run\"><span>Field Shift · {}</span><span>{} {}</span></div>", t("technical sheet", "ficha técnica"), t("Generated", "Generado"), esc(&generated)));
    h.push_str(&format!("<div class=\"head\"><div><h1>{}</h1><div class=\"mono\" style=\"color:var(--muted);margin-top:6px\">{}</div></div>{}</div>",
        esc(&label), coord(st.field.center.lat, st.field.center.lon), tile(&st.field.ring)));
    let demo = if st.demo_mode { t(" Consultant replies in this analysis came from the scripted demo mode; all readings are live NASA data.", " Las respuestas del consultor en este análisis vinieron del modo demo guionizado; todas las lecturas son datos reales de la NASA.") } else { String::new() };
    h.push_str(&format!("<div class=\"notice\"><b>{}</b><p>{}{}</p></div>", t("PRE-DIAGNOSIS · NOT A DIAGNOSIS", "PREDIAGNÓSTICO · NO ES UN DIAGNÓSTICO"),
        t("Prepared for the farmer's agronomist or extension engineer, who makes the final decision. Regional satellite context is not a field measurement.",
          "Preparado para el agrónomo o ingeniero extensionista del agricultor, quien toma la decisión final. El contexto satelital regional no es una medición de la parcela."), demo));
    h.push_str(&format!("<div class=\"meta\"><div><b>{}</b>{}</div><div><b>{}</b>{}</div><div><b>{}</b>{}</div><div><b>{}</b>{}</div></div>",
        t("Area", "Área"), if st.field.area_ha > 0.0 { format!("{:.2} ha", st.field.area_ha) } else { t("point, no outline", "punto, sin contorno") },
        t("Crop now", "Cultivo actual"), crop,
        t("Before", "Antes"), esc(&if prev.is_empty() { t("not stated", "no indicado") } else { prev.join(", ") }),
        t("Readings", "Lecturas"), ev["readings"].as_array().map(|r| r.len()).unwrap_or(0)));

    // Readings: grid rows on desktop, stacked cards on a phone; licences listed once below.
    h.push_str(&format!("<h2>{}</h2>", t("NASA and satellite readings", "Lecturas de la NASA y satelitales")));
    let readings = ev["readings"].as_array().cloned().unwrap_or_default();
    let mut licences = BTreeSet::new();
    if readings.is_empty() {
        h.push_str(&format!("<p>{}</p>", t("The evidence package has not been built yet.", "El paquete de evidencia aún no se ha generado.")));
    } else {
        h.push_str(&format!("<p>{}</p>", t("Plain words first, the figure second. Field-level readings measure the plot (10–30 m); regional readings describe the surrounding area.",
            "Primero en palabras simples, luego la cifra. Las lecturas de nivel de campo miden la parcela (10–30 m); las regionales describen la zona.")));
        h.push_str(&format!("<div class=\"readings\"><div class=\"r rh\"><span>{}</span><span>{}</span><span>{}</span><span>{}</span></div>",
            t("Reading", "Lectura"), t("Value", "Valor"), t("Plain words", "En palabras"), t("Source · date · resolution", "Fuente · fecha · resolución")));
        for r in &readings {
            let badge = r["badge"].as_str().unwrap_or("");
            let field = badge == "field-level";
            let badge_txt = match (field, es) { (true, true) => "nivel de campo", (false, true) => "contexto regional", (true, false) => "field-level", (false, false) => "regional context" };
            let key = r["key"] == true;
            let v = r["value"].as_f64();
            let val = match v { Some(x) if key => format!("{x:+.1}"), _ => num(&r["value"]) };
            let unit = unit_l(&r["unit"].as_str().unwrap_or("").replace("°C vs normal", if es { "°C vs normal" } else { "°C vs normal" }), es);
            if let Some(l) = r["license"].as_str().filter(|l| !l.is_empty()) { licences.insert(format!("{}: {}", r["source"].as_str().unwrap_or("").split(" + ").next().unwrap_or(""), l)); }
            h.push_str(&format!("<div class=\"r\"><span class=\"lab\">{}</span><span class=\"val{}\">{} {}</span><span class=\"pw\">{}</span><span class=\"src\">{} · <span style=\"white-space:nowrap\">{}</span> · {}<br><span class=\"badge{}\">{}</span></span></div>",
                esc(&bi(&r["label"], lang)), if key { " key" } else { "" }, esc(&val).replace(' ', "&nbsp;"), esc(&unit).replace(' ', "&nbsp;"), esc(&cap(&bi(&r["verdict"], lang))),
                esc(&src_l(r["source"].as_str().unwrap_or(""), es)), esc(r["date"].as_str().unwrap_or("")), esc(&res_l(r["resolution"].as_str().unwrap_or(""), es)),
                if field { " field" } else { "" }, badge_txt));
        }
        h.push_str("</div>");
        let bare: Vec<String> = ev["bare_soil"].as_array().into_iter().flatten().filter_map(|b| Some(format!("{} – {}", b["start"].as_str()?, b["end"].as_str()?))).collect();
        if !bare.is_empty() {
            h.push_str(&format!("<p class=\"cite\"><b>{}</b> {}</p>", t("Bare-soil periods on this field (NDVI below 0.2 on two clear images in a row; Sentinel-2 / Landsat, field-level):", "Periodos de suelo desnudo en este campo (NDVI menor a 0.2 en dos imágenes despejadas seguidas; Sentinel-2 / Landsat, nivel de campo):"), esc(&bare.join(" · "))));
        }
        h.push_str(&format!("<div class=\"lic\">{} {}</div>", t("Licences:", "Licencias:"), licences.iter().map(|l| esc(l)).collect::<Vec<_>>().join(" · ")));
    }
    if let Some(pests) = ev["pests"].as_array().filter(|p| !p.is_empty()) {
        h.push_str(&format!("<h3>{}</h3><div class=\"scroll\"><table><thead><tr><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
            t("Reference pests (general published models, not specific to this crop): generations from degree-days, NASA POWER temperature, regional", "Plagas de referencia (modelos publicados generales, no específicos de este cultivo): generaciones por grados-día, temperatura NASA POWER, regional"),
            t("Pest", "Plaga"), t("This season", "Esta campaña"), t("40-year normal", "Normal de 40 años")));
        for p in pests {
            h.push_str(&format!("<tr><td>{} <i>{}</i><div class=\"cite\">{}</div></td><td class=\"n\">{}</td><td class=\"n\">{}</td></tr>",
                esc(p["name"].as_str().unwrap_or("")), esc(p["scientific"].as_str().unwrap_or("")), esc(p["citation"].as_str().unwrap_or("")),
                num(&p["generations_now"]), num(&p["generations_normal"])));
        }
        h.push_str("</tbody></table></div>");
    }
    if let Some(pj) = ev.get("projection").filter(|p| p.is_object()) {
        h.push_str(&format!("<p class=\"cite\">{}: {} °C, {} % {} ({}, {}; {}). {}</p>",
            t("Climate projection", "Proyección climática"), num(&pj["d_temp_c"]), num(&pj["d_rain_pct"]), t("rain", "lluvia"),
            esc(pj["horizon"].as_str().unwrap_or("")), esc(pj["scenario"].as_str().unwrap_or("")),
            esc(&pj["models"].as_array().map(|m| m.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ")).unwrap_or_default()),
            t("A projection, not a forecast.", "Una proyección, no un pronóstico.")));
    }

    // Result sections (the "Your field" summary is the readings above)
    let mut gaps: Vec<String> = ev["gaps"].as_array().into_iter().flatten().map(|g| bi(&g["text"], lang)).collect();
    for s in st.sections.iter().filter(|s| s.key != "sheet" && s.key != "field") {
        let Some(c) = &s.content else { continue };
        h.push_str(&format!("<h2>{}</h2>", esc(&bi(&c["title"], lang))));
        let summary = bi(&c["summary"], lang);
        if !summary.is_empty() { h.push_str(&format!("<p>{}</p>", esc(&summary))); }
        if let Some(cp) = c["crop_pests"].as_array().filter(|p| !p.is_empty()) {
            h.push_str(&format!("<div class=\"scroll\"><table><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
                t("This crop's pests (published thresholds)", "Plagas de este cultivo (umbrales publicados)"), t("This season", "Esta campaña"), t("40-year normal", "Normal de 40 años"), t("Threshold", "Umbral")));
            for p in cp {
                let cit = &p["citation"];
                h.push_str(&format!("<tr><td>{} <i>{}</i><div class=\"cite\"><a href=\"{}\">{}</a></div></td><td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{} °C · {} °C·{}</td></tr>",
                    esc(&bi(&p["name"], lang)), esc(p["scientific"].as_str().unwrap_or("")), esc(cit["url"].as_str().unwrap_or("")), esc(cit["title"].as_str().unwrap_or("")),
                    num(&p["generations_now"]), num(&p["generations_normal"]), num(&p["lower_threshold_c"]), num(&p["degree_days"]), t("day", "día")));
            }
            h.push_str("</tbody></table></div>");
        }
        if let Some(strats) = c["strategies"].as_array().filter(|s| !s.is_empty()) {
            // Evidence shared by every strategy (the field's NASA signals) is printed once above the cards.
            let ev_en = |x: &Value| -> Vec<String> { x["evidence_list"].as_array().into_iter().flatten().map(|e| bi(e, "en")).collect() };
            let common: Vec<String> = ev_en(&strats[0]).into_iter().filter(|e| strats.iter().all(|x| ev_en(x).contains(e))).collect();
            let common_l: Vec<String> = strats[0]["evidence_list"].as_array().into_iter().flatten().filter(|e| common.contains(&bi(e, "en"))).map(|e| cap(&bi(e, lang))).collect();
            if !common_l.is_empty() {
                h.push_str(&format!("<p class=\"cite\"><b>{}</b> {}</p>", t("This field's signals, used by all three:", "Señales de este campo, usadas por las tres:"), esc(&common_l.join(" · "))));
            }
            h.push_str("<div class=\"strat\">");
            for x in strats {
                let sc = &x["scores"];
                let dots = |k: &str| { let n = (sc[k].as_u64().unwrap_or(0) as usize).min(5); format!("<span role=\"img\" aria-label=\"{n} / 5\">{}{}</span>", "●".repeat(n), "○".repeat(5 - n)) };
                let list = |k: &str| x[k].as_array().into_iter().flatten().filter(|i| k != "evidence_list" || !common.contains(&bi(i, "en"))).map(|i| format!("<li>{}</li>", esc(&cap(&bi(i, lang))))).collect::<String>();
                let tl_key = if es { "timeline_es" } else { "timeline" };
                h.push_str(&format!("<div class=\"{}\"><h3>{}</h3><div class=\"tl\">{}</div><div class=\"scores\"><span>{}</span><span>{}</span><span>{}</span><span>{}</span><span>{}</span><span>{}</span><span>{}</span><span>{}</span><span>{}</span><span>{}</span></div><p><b>{}</b> {}</p><b>{}</b><ul>{}</ul><b>{}</b><ul>{}</ul><b>{}</b><ul>{}</ul></div>",
                    if x["current"] == true { "cur" } else { "" }, esc(&bi(&x["name"], lang)),
                    esc(&x[tl_key].as_array().or(x["timeline"].as_array()).map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" → ")).unwrap_or_default()),
                    t("Soil", "Suelo"), dots("soil"), t("Water", "Agua"), dots("water"), t("Pest break", "Corte de plagas"), dots("pest_break"), t("Income", "Ingreso"), dots("income"), t("Low risk", "Bajo riesgo"), dots("risk"),
                    t("Why:", "Por qué:"), esc(&bi(&x["why"], lang)), t("Evidence", "Evidencia"), list("evidence_list"), t("Gaps", "Vacíos"), list("gaps_list"), t("Risks", "Riesgos"), list("risks_list")));
                for g in x["gaps_list"].as_array().into_iter().flatten() { gaps.push(bi(g, lang)); }
            }
            h.push_str(&format!("</div><p class=\"cite\">{}</p>", t("Scores 1–5: more dots = better (for risk, more dots = lower risk).", "Puntajes 1–5: más puntos = mejor (en riesgo, más puntos = menor riesgo).")));
        }
        // Items; consecutive biological-control items become one table
        let items = c["items"].as_array().cloned().unwrap_or_default();
        let is_bio = |it: &Value| { let ti = bi(&it["title"], "en").to_lowercase(); ti.starts_with("biological control") };
        let bio: Vec<&Value> = items.iter().filter(|i| is_bio(i)).collect();
        for it in items.iter().filter(|i| !is_bio(i)) {
            let title = bi(&it["title"], lang);
            let body = cap(&bi(&it["body"], lang));
            if title.is_empty() { h.push_str(&format!("<p>{}</p>", esc(&body))); } else { h.push_str(&format!("<p><b>{}.</b> {}</p>", esc(&title), esc(&body))); }
            let cites: Vec<String> = it["citations"].as_array().into_iter().flatten().map(|c| {
                let url = c["url"].as_str().unwrap_or("");
                format!("<a href=\"{}\">{}</a>", esc(url), esc(c["title"].as_str().unwrap_or(url)))
            }).collect();
            if !cites.is_empty() { h.push_str(&format!("<p class=\"cite\">{} {}</p>", t("Sources:", "Fuentes:"), cites.join(" · "))); }
        }
        if !bio.is_empty() {
            h.push_str(&format!("<h3>{}</h3><div class=\"scroll\"><table><thead><tr><th>{}</th><th>{}</th></tr></thead><tbody>",
                t("Biological control", "Control biológico"), t("Agent and target", "Agente y objetivo"), t("Source", "Fuente")));
            for it in bio {
                let cites: Vec<String> = it["citations"].as_array().into_iter().flatten().map(|c| {
                    let url = c["url"].as_str().unwrap_or("");
                    format!("<a href=\"{}\">{}</a>", esc(url), esc(c["title"].as_str().unwrap_or(url)))
                }).collect();
                h.push_str(&format!("<tr><td>{}</td><td class=\"cite\">{}</td></tr>", esc(&cap(&bi(&it["body"], lang))), cites.join(" · ")));
            }
            h.push_str("</tbody></table></div>");
        }
        for g in c["gaps"].as_array().into_iter().flatten() { gaps.push(bi(g, lang)); }
        for g in c["declared_gaps"].as_array().into_iter().flatten() { gaps.push(bi(g, lang)); }
    }
    for s in st.sections.iter().filter(|s| s.key == "field") {
        for g in s.content.as_ref().and_then(|c| c["declared_gaps"].as_array()).into_iter().flatten() { gaps.push(bi(g, lang)); }
    }
    if readings.iter().any(|r| r["id"] == "water_balance") {
        gaps.push(t("Irrigation is not modelled: the water balance is rain minus reference evapotranspiration.", "El riego no está modelado: el balance hídrico es lluvia menos evapotranspiración de referencia."));
    }
    let mut seen = BTreeSet::new();
    gaps.retain(|g| !g.trim().is_empty() && seen.insert(g.clone()));
    h.push_str(&format!("<h2>{}</h2>", t("Information gaps", "Vacíos de información")));
    if gaps.is_empty() { h.push_str(&format!("<p>{}</p>", t("None declared.", "Ninguno declarado."))); }
    else { h.push_str(&format!("<ul class=\"gaps\">{}</ul>", gaps.iter().map(|g| format!("<li>{}</li>", esc(&cap(g)))).collect::<String>())); }

    h.push_str(&format!("<footer>{}<br>{}</footer>",
        t("Field Shift, team InnoHuando, NASA Space Apps Challenge 2026. Open source (Apache-2.0).", "Field Shift, equipo InnoHuando, NASA Space Apps Challenge 2026. Código abierto (Apache-2.0)."),
        t("Every reading comes from the listed datasets; agronomic coefficients (Kc, thresholds, water needs) come from the cited literature. Advice text was assembled with AI assistance (Claude, Anthropic) from those sources.",
          "Cada lectura proviene de los conjuntos de datos listados; los coeficientes agronómicos (Kc, umbrales, necesidades de agua) provienen de la literatura citada. Los textos se armaron con ayuda de IA (Claude, Anthropic) a partir de esas fuentes.")));
    h.push_str("</main></body></html>");
    h
}

fn unit_l(u: &str, es: bool) -> String {
    if !es { return u.to_string(); }
    match u { "days" => "días", "generations" => "generaciones", "°C·day" => "°C·día", other => other }.to_string()
}

fn res_l(r: &str, es: bool) -> String {
    let short = if r.contains("about 50 km") || r.contains("0.5°") { "~50 km (0.5° × 0.625°)".to_string() } else { r.to_string() };
    if es { short.replace("grid cell", "celda") } else { short }
}

fn src_l(s: &str, es: bool) -> String {
    if es { s.replace("published threshold", "umbral publicado") } else { s.to_string() }
}
