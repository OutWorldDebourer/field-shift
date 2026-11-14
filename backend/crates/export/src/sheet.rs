//! The technical sheet for the agronomist: one printable page set (A4) with the field, crops, every reading with
//! source, date, resolution, scale badge and licence, the result sections with citations, gaps and the notice.
use crate::html::{bi, esc, num};
use analyses::AnalysisState;
use serde_json::Value;

const CSS: &str = r#"
@font-face{font-family:"Source Serif 4";font-style:italic;font-weight:400;src:url(/fonts/source-serif-4-latin-400-italic.woff2) format("woff2")}
@font-face{font-family:"Source Sans 3";font-weight:400;src:url(/fonts/source-sans-3-latin-400-normal.woff2) format("woff2")}
@font-face{font-family:"Source Sans 3";font-weight:600;src:url(/fonts/source-sans-3-latin-600-normal.woff2) format("woff2")}
@font-face{font-family:"IBM Plex Mono";font-weight:400;src:url(/fonts/ibm-plex-mono-latin-400-normal.woff2) format("woff2")}
:root{--ink:#14202b;--muted:#4d5b67;--rule:#c9d3db;--amber:#b86e00;--blue:#2e70ce;--paper:#fff;--wash:#f3f7fa}
*{box-sizing:border-box}
body{margin:0;background:#e9eef2;color:var(--ink);font:15px/1.5 "Source Sans 3",system-ui,sans-serif}
.page{max-width:820px;margin:24px auto;background:var(--paper);padding:48px 56px;box-shadow:0 1px 3px rgba(0,0,0,.12)}
h1,h2{font-family:"Source Serif 4",Georgia,serif;font-style:italic;font-weight:400;margin:0}
h1{font-size:34px;line-height:1.1}
h2{font-size:22px;margin:32px 0 10px;padding-top:14px;border-top:1px solid var(--rule)}
h3{font-size:15px;margin:14px 0 4px}
.mono,.src,th{font-family:"IBM Plex Mono",ui-monospace,monospace}
.top{display:flex;justify-content:space-between;align-items:flex-start;gap:24px}
.tag{font:12px "IBM Plex Mono",monospace;color:var(--muted);text-transform:uppercase;letter-spacing:.06em}
.notice{margin:20px 0;padding:12px 16px;border-left:3px solid var(--amber);background:#fff8ec;font-weight:600}
.meta{display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin-top:18px}
.meta div{background:var(--wash);padding:10px 12px;border-radius:6px}
.meta b{display:block;font:11px "IBM Plex Mono",monospace;color:var(--muted);text-transform:uppercase;letter-spacing:.05em}
table{width:100%;border-collapse:collapse;font-size:13.5px}
th{font-size:11px;text-align:left;color:var(--muted);text-transform:uppercase;letter-spacing:.04em;border-bottom:1px solid var(--ink);padding:6px 6px}
td{padding:7px 6px;border-bottom:1px solid var(--rule);vertical-align:top}
td.v{white-space:nowrap;font-weight:600}
.badge{display:inline-block;font:11px "IBM Plex Mono",monospace;padding:1px 6px;border-radius:9px;border:1px solid var(--rule);white-space:nowrap}
.badge.field{border-color:#2f7d4f;color:#1f5c38}
.src{font-size:12px;color:var(--muted)}
.key{color:var(--amber)}
ul{margin:4px 0 8px 18px;padding:0}
.cite{font-size:12px;color:var(--muted)}
.cite a{color:var(--blue)}
.strat{display:grid;grid-template-columns:repeat(3,1fr);gap:10px}
.strat>div{border:1px solid var(--rule);border-radius:6px;padding:10px 12px}
.strat .cur{border-color:var(--ink)}
.dots{font:12px "IBM Plex Mono",monospace;color:var(--muted)}
.actions{max-width:820px;margin:16px auto 0;display:flex;gap:10px;justify-content:flex-end}
.actions button,.actions a{font:600 14px "Source Sans 3",sans-serif;padding:10px 16px;border-radius:6px;border:1px solid var(--ink);background:var(--ink);color:#fff;cursor:pointer;text-decoration:none}
.actions a{background:#fff;color:var(--ink)}
footer{margin-top:28px;font-size:12px;color:var(--muted)}
@media (max-width:640px){.page{padding:24px 18px;margin:0}.meta{grid-template-columns:repeat(2,1fr)}.strat{grid-template-columns:1fr}table{font-size:12.5px}.hide-sm{display:none}}
@media print{body{background:#fff}.page{box-shadow:none;margin:0;max-width:none;padding:0}.actions{display:none}h2{break-after:avoid}tr,.strat>div{break-inside:avoid}@page{size:A4;margin:16mm}}
"#;

pub fn render(st: &AnalysisState, lang: &str, csv_url: &str) -> String {
    let es = lang == "es";
    let t = |en: &str, es_: &str| if es { es_.to_string() } else { en.to_string() };
    let ev = st.evidence.clone().unwrap_or(Value::Null);
    let crop = st.crop_current.as_ref().map(|c| format!("{} ({})", bi(&c["names"], lang), c["scientific"].as_str().unwrap_or(""))).unwrap_or_else(|| t("not stated", "no indicado"));
    let prev: Vec<String> = st.crop_history.iter().map(|c| bi(&c["names"], lang)).collect();
    let crop_type = st.crop_current.as_ref().and_then(|c| c["crop_type"].as_str()).unwrap_or("—");
    let generated = ev["generated_at"].as_str().unwrap_or("").chars().take(10).collect::<String>();

    let mut h = String::with_capacity(32_000);
    h.push_str(&format!("<!doctype html><html lang=\"{lang}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{} · Field Shift</title><style>{CSS}</style></head><body>",
        t("Technical sheet", "Ficha técnica")));
    h.push_str(&format!("<div class=\"actions\"><a href=\"{}\">{}</a><button onclick=\"print()\">{}</button></div>", esc(csv_url), t("Download CSV", "Descargar CSV"), t("Save as PDF / Print", "Guardar PDF / Imprimir")));
    h.push_str("<main class=\"page\">");
    h.push_str(&format!("<div class=\"top\"><div><div class=\"tag\">Field Shift · {}</div><h1>{}</h1></div><div class=\"tag\" style=\"text-align:right\">{}<br>{}</div></div>",
        t("technical sheet", "ficha técnica"), esc(&st.field.label), t("Generated", "Generado"), esc(&generated)));
    h.push_str(&format!("<p class=\"notice\">{}</p>", t(
        "PRE-DIAGNOSIS, NOT A DIAGNOSIS. Prepared for the farmer's agronomist or extension engineer, who makes the final decision. Regional satellite context is not a field measurement.",
        "PREDIAGNÓSTICO, NO UN DIAGNÓSTICO. Preparado para el agrónomo o ingeniero extensionista del agricultor, quien toma la decisión final. El contexto satelital regional no es una medición del campo.")));
    h.push_str(&format!("<div class=\"meta\"><div><b>{}</b>{:.4}, {:.4}</div><div><b>{}</b>{}</div><div><b>{}</b>{}</div><div><b>{}</b>{}</div></div>",
        t("Location", "Ubicación"), st.field.center.lat, st.field.center.lon,
        t("Area", "Área"), if st.field.area_ha > 0.0 { format!("{:.2} ha", st.field.area_ha) } else { t("point (no outline)", "punto (sin contorno)") },
        t("Crop now", "Cultivo actual"), esc(&format!("{crop} · {crop_type}")),
        t("Before", "Antes"), esc(&if prev.is_empty() { t("not stated", "no indicado") } else { prev.join(", ") })));

    // Readings
    h.push_str(&format!("<h2>{}</h2>", t("NASA and satellite readings", "Lecturas de la NASA y satelitales")));
    let readings = ev["readings"].as_array().cloned().unwrap_or_default();
    if readings.is_empty() {
        h.push_str(&format!("<p>{}</p>", t("The evidence package has not been built yet.", "El paquete de evidencia aún no se ha generado.")));
    } else {
        h.push_str(&format!("<table><thead><tr><th>{}</th><th>{}</th><th>{}</th><th class=\"hide-sm\">{}</th><th>{}</th></tr></thead><tbody>",
            t("Reading", "Lectura"), t("Value", "Valor"), t("Plain words", "En palabras"), t("Source · date · resolution", "Fuente · fecha · resolución"), t("Scale", "Escala")));
        for r in &readings {
            let badge = r["badge"].as_str().unwrap_or("");
            let cls = if badge == "field-level" { "badge field" } else { "badge" };
            let badge_txt = match (badge, es) { ("field-level", true) => "nivel de campo", ("regional context", true) => "contexto regional", (b, _) => b };
            h.push_str(&format!("<tr><td>{}</td><td class=\"v{}\">{} {}</td><td>{}</td><td class=\"src hide-sm\">{} · {} · {} · {}<br>{}</td><td><span class=\"{cls}\">{}</span></td></tr>",
                esc(&bi(&r["label"], lang)), if r["key"] == true { " key" } else { "" }, num(&r["value"]), esc(r["unit"].as_str().unwrap_or("")),
                esc(&bi(&r["verdict"], lang)), esc(r["source"].as_str().unwrap_or("")), esc(r["dataset"].as_str().unwrap_or("")),
                esc(r["date"].as_str().unwrap_or("")), esc(r["resolution"].as_str().unwrap_or("")), esc(r["license"].as_str().unwrap_or("")), esc(badge_txt)));
        }
        h.push_str("</tbody></table>");
    }
    if let Some(pests) = ev["pests"].as_array().filter(|p| !p.is_empty()) {
        h.push_str(&format!("<h3>{}</h3><table><thead><tr><th>{}</th><th>{}</th><th>{}</th><th class=\"hide-sm\">{}</th></tr></thead><tbody>",
            t("Pest generations from degree-days (NASA POWER temperature)", "Generaciones de plagas por grados-día (temperatura NASA POWER)"),
            t("Pest", "Plaga"), t("This season", "Esta campaña"), t("40-year normal", "Normal de 40 años"), t("Threshold source", "Fuente del umbral")));
        for p in pests {
            h.push_str(&format!("<tr><td>{} <i>{}</i></td><td class=\"v\">{}</td><td>{}</td><td class=\"cite hide-sm\">{}</td></tr>",
                esc(p["name"].as_str().unwrap_or("")), esc(p["scientific"].as_str().unwrap_or("")), num(&p["generations_now"]), num(&p["generations_normal"]), esc(p["citation"].as_str().unwrap_or(""))));
        }
        h.push_str("</tbody></table>");
    }
    if let Some(pj) = ev.get("projection").filter(|p| p.is_object()) {
        h.push_str(&format!("<p class=\"src\">{}: {} °C, {} % {} ({}, {}; {}). {}</p>",
            t("Climate projection", "Proyección climática"), num(&pj["d_temp_c"]), num(&pj["d_rain_pct"]), t("rain", "lluvia"),
            esc(pj["horizon"].as_str().unwrap_or("")), esc(pj["scenario"].as_str().unwrap_or("")),
            esc(&pj["models"].as_array().map(|m| m.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ")).unwrap_or_default()),
            t("A projection, not a forecast.", "Una proyección, no un pronóstico.")));
    }

    // Sections
    for s in st.sections.iter().filter(|s| s.key != "sheet") {
        let Some(c) = &s.content else { continue };
        h.push_str(&format!("<h2>{}</h2>", esc(&bi(&c["title"], lang))));
        let summary = bi(&c["summary"], lang);
        if !summary.is_empty() { h.push_str(&format!("<p>{}</p>", esc(&summary))); }
        if let Some(strats) = c["strategies"].as_array() {
            h.push_str("<div class=\"strat\">");
            for x in strats {
                let sc = &x["scores"];
                let dots = |k: &str| { let n = sc[k].as_u64().unwrap_or(0) as usize; format!("{}{}", "●".repeat(n.min(5)), "○".repeat(5 - n.min(5))) };
                h.push_str(&format!("<div class=\"{}\"><h3>{}</h3><div class=\"mono src\">{}</div><div class=\"dots\">{} {}<br>{} {}<br>{} {}<br>{} {}<br>{} {}</div><p><b>{}</b> {}</p><p><b>{}</b> {}</p><p><b>{}</b> {}</p><p><b>{}</b> {}</p></div>",
                    if x["current"] == true { "cur" } else { "" }, esc(&bi(&x["name"], lang)),
                    esc(&x["timeline"].as_array().map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" → ")).unwrap_or_default()),
                    t("Soil", "Suelo"), dots("soil"), t("Water", "Agua"), dots("water"), t("Pest break", "Corte de plagas"), dots("pest_break"), t("Income", "Ingreso"), dots("income"), t("Risk (5 = low)", "Riesgo (5 = bajo)"), dots("risk"),
                    t("Why:", "Por qué:"), esc(&bi(&x["why"], lang)), t("Evidence:", "Evidencia:"), esc(&bi(&x["evidence"], lang)), t("Gaps:", "Vacíos:"), esc(&bi(&x["gaps"], lang)), t("Risks:", "Riesgos:"), esc(&bi(&x["risks"], lang))));
            }
            h.push_str("</div>");
        }
        for it in c["items"].as_array().into_iter().flatten() {
            h.push_str(&format!("<h3>{}</h3><p>{}</p>", esc(&bi(&it["title"], lang)), esc(&bi(&it["body"], lang))));
            let cites: Vec<String> = it["citations"].as_array().into_iter().flatten().map(|c| {
                let url = c["url"].as_str().unwrap_or("");
                format!("<a href=\"{}\">{}</a>", esc(url), esc(c["title"].as_str().unwrap_or(url)))
            }).collect();
            if !cites.is_empty() { h.push_str(&format!("<p class=\"cite\">{} {}</p>", t("Sources:", "Fuentes:"), cites.join(" · "))); }
        }
    }

    // Gaps
    let mut gaps: Vec<String> = ev["gaps"].as_array().into_iter().flatten().map(|g| bi(&g["text"], lang)).collect();
    for s in &st.sections {
        for g in s.content.as_ref().and_then(|c| c["declared_gaps"].as_array()).into_iter().flatten() { gaps.push(bi(g, lang)); }
    }
    h.push_str(&format!("<h2>{}</h2>", t("Information gaps", "Vacíos de información")));
    if gaps.is_empty() { h.push_str(&format!("<p>{}</p>", t("None declared.", "Ninguno declarado."))); }
    else { h.push_str(&format!("<ul>{}</ul>", gaps.iter().map(|g| format!("<li>{}</li>", esc(g))).collect::<String>())); }

    h.push_str(&format!("<footer>{} {}<br>{}</footer>",
        t("Field Shift, team InnoHuando, NASA Space Apps Challenge 2026. Open source (Apache-2.0).", "Field Shift, equipo InnoHuando, NASA Space Apps Challenge 2026. Código abierto (Apache-2.0)."),
        if st.demo_mode { t("Parts of this analysis used the scripted demo mode.", "Partes de este análisis usaron el modo demo guionizado.") } else { String::new() },
        t("Advice text was assembled with AI assistance (Claude, Anthropic) from cited sources; every number comes from the listed datasets.",
          "Los textos se armaron con ayuda de IA (Claude, Anthropic) a partir de fuentes citadas; cada número proviene de los conjuntos de datos listados.")));
    h.push_str("</main></body></html>");
    h
}
