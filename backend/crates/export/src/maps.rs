//! "Data maps of your field" on the technical sheet: the same four layers as the dashboard map.
//! Vigor (NDVI per 10 m pixel) and slope (classes per 30 m cell) are images with the field outline drawn on top;
//! soil moisture (SMAP 9 km) and temperature (NASA POWER ~50 km) are their cells coloured by value, with the field
//! marked inside, so the sheet says plainly that those two describe the area, not the plot.
use crate::html::esc;
use analyses::AnalysisState;
use common::AppState;
use imagery::layer::LayerMeta;
use serde_json::Value;

/// Layer images available for this sheet (fetched before rendering; missing ones are left out).
#[derive(Default, Clone)]
pub struct Maps { pub ndvi: Option<LayerMeta>, pub slope: Option<LayerMeta> }

pub const NDVI_SIZE_M: f64 = 1500.0;
pub const SLOPE_SIZE_M: f64 = 5000.0;

/// Fetch (or read from the 24 h cache) both layer images, in parallel, waiting at most 25 s.
pub async fn fetch(state: &AppState, st: &AnalysisState) -> (Maps, Vec<(String, Vec<u8>)>) {
    let (lat, lon) = (st.field.center.lat, st.field.center.lon);
    let get = |kind: &'static str, size: f64| async move {
        tokio::time::timeout(std::time::Duration::from_secs(25), imagery::layer::layer(&state.http, &state.config.root, kind, lat, lon, size)).await.ok().and_then(|r| r.ok())
    };
    let (n, s) = tokio::join!(get("ndvi", NDVI_SIZE_M), get("slope", SLOPE_SIZE_M));
    let mut files = vec![];
    let mut maps = Maps::default();
    if let Some((png, m)) = n { files.push(("map-vigor.png".to_string(), png)); maps.ndvi = Some(m); }
    if let Some((png, m)) = s { files.push(("map-slope.png".to_string(), png)); maps.slope = Some(m); }
    (maps, files)
}

fn reading<'a>(ev: &'a Value, id: &str) -> Option<&'a Value> {
    ev["readings"].as_array()?.iter().find(|r| r["id"].as_str().map(|x| x.starts_with(id)).unwrap_or(false))
}

type Stop = (f64, [u8; 3]);
const NDVI: [Stop; 5] = [(0.0, [165, 0, 38]), (0.2, [244, 109, 67]), (0.4, [254, 224, 139]), (0.6, [166, 217, 106]), (0.8, [26, 152, 80])];
const SOIL: [Stop; 4] = [(0.0, [150, 98, 52]), (0.15, [214, 176, 112]), (0.3, [110, 178, 214]), (0.45, [36, 92, 170])];
const TEMP: [Stop; 4] = [(-4.0, [70, 130, 210]), (0.0, [236, 236, 228]), (3.0, [244, 170, 80]), (6.0, [206, 52, 46])];
const SLOPE: [([u8; 3], &str, &str); 5] = [
    ([126, 200, 140], "Flat 0–3 %", "Plano 0–3 %"), ([236, 216, 110], "Gentle 3–8 %", "Suave 3–8 %"),
    ([244, 166, 72], "Moderate 8–15 %", "Moderada 8–15 %"), ([226, 92, 58], "Strong 15–30 %", "Fuerte 15–30 %"),
    ([160, 36, 64], "Very steep > 30 %", "Muy fuerte > 30 %"),
];

fn ramp(stops: &[Stop], v: f64) -> String {
    let x = v.clamp(stops[0].0, stops[stops.len() - 1].0);
    for w in stops.windows(2) {
        let ((a, ca), (b, cb)) = (w[0], w[1]);
        if x <= b {
            let f = if b > a { (x - a) / (b - a) } else { 0.0 };
            let c: Vec<String> = (0..3).map(|i| format!("{}", (ca[i] as f64 + (cb[i] as f64 - ca[i] as f64) * f).round())).collect();
            return format!("rgb({})", c.join(","));
        }
    }
    let c = stops[stops.len() - 1].1;
    format!("rgb({},{},{})", c[0], c[1], c[2])
}

fn gradient(stops: &[Stop]) -> String {
    let (lo, hi) = (stops[0].0, stops[stops.len() - 1].0);
    let parts: Vec<String> = stops.iter().map(|(v, c)| format!("rgb({},{},{}) {:.0}%", c[0], c[1], c[2], (v - lo) / (hi - lo) * 100.0)).collect();
    format!("linear-gradient(90deg,{})", parts.join(","))
}

/// Colour scale with the field's value marked on it.
fn scale(stops: &[Stop], v: Option<f64>, lo: &str, hi: &str, label: &str) -> String {
    let (a, b) = (stops[0].0, stops[stops.len() - 1].0);
    let mark = v.map(|v| format!("<i style=\"left:{:.1}%\"><b>{}</b></i>", ((v - a) / (b - a) * 100.0).clamp(0.0, 100.0), esc(label))).unwrap_or_default();
    format!("<div class=\"mscale\"><span class=\"mscale__bar\" style=\"background:{}\">{mark}</span><span class=\"mscale__ends\"><span>{}</span><span>{}</span></span></div>", gradient(stops), esc(lo), esc(hi))
}

/// The field outline as an SVG path over an image covering `bbox` ([w, s, e, n]).
fn outline(st: &AnalysisState, bbox: [f64; 4]) -> String {
    let [w, s, e, n] = bbox;
    let px = |lon: f64, lat: f64| ((lon - w) / (e - w) * 1000.0, (n - lat) / (n - s) * 1000.0);
    if st.field.ring.len() >= 3 {
        let d: Vec<String> = st.field.ring.iter().enumerate().map(|(i, [lon, lat])| { let (x, y) = px(*lon, *lat); format!("{}{x:.1} {y:.1}", if i == 0 { "M" } else { "L" }) }).collect();
        format!("<svg viewBox=\"0 0 1000 1000\" preserveAspectRatio=\"none\" aria-hidden=\"true\"><path d=\"{}Z\" fill=\"none\" stroke=\"#fff\" stroke-width=\"5\" vector-effect=\"non-scaling-stroke\"/><path d=\"{}Z\" fill=\"none\" stroke=\"#0f1d29\" stroke-width=\"1.5\" vector-effect=\"non-scaling-stroke\"/></svg>", d.join(""), d.join(""))
    } else {
        let (x, y) = px(st.field.center.lon, st.field.center.lat);
        format!("<svg viewBox=\"0 0 1000 1000\" preserveAspectRatio=\"none\" aria-hidden=\"true\"><circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"14\" fill=\"none\" stroke=\"#fff\" stroke-width=\"4\"/></svg>")
    }
}

/// A coarse cell drawn as a coloured square with the field marked at its true position inside it.
fn cell(km: f64, colour: &str, lang: &str) -> String {
    let es = lang == "es";
    format!("<div class=\"mcell\"><svg viewBox=\"0 0 200 200\" role=\"img\" aria-label=\"{0}\"><rect x=\"4\" y=\"4\" width=\"192\" height=\"192\" rx=\"6\" fill=\"{1}\" stroke=\"#0f1d29\" stroke-dasharray=\"6 5\" stroke-width=\"1.5\"/>\
<circle cx=\"100\" cy=\"100\" r=\"5\" fill=\"#fff\" stroke=\"#0f1d29\" stroke-width=\"2\"/><text x=\"112\" y=\"104\" font-size=\"12\" fill=\"#0f1d29\" font-family=\"sans-serif\">{2}</text>\
<text x=\"12\" y=\"190\" font-size=\"11\" fill=\"#0f1d29\" font-family=\"monospace\">{3}</text></svg></div>",
        if es { format!("Celda de {km:.0} km") } else { format!("{km:.0} km cell") }, colour,
        if es { "tu campo" } else { "your field" }, if es { format!("celda {km:.0} km") } else { format!("{km:.0} km cell") })
}

fn km_of(res: &str) -> Option<f64> {
    let l = res.to_lowercase();
    let i = l.find("km")?;
    l[..i].rsplit(|c: char| !(c.is_ascii_digit() || c == '.')).find(|s| !s.is_empty())?.parse().ok()
}

fn fmt_num(v: f64, d: usize) -> String { format!("{v:.d$}") }

pub fn section(st: &AnalysisState, ev: &Value, lang: &str, zip: bool, maps: &Maps) -> String {
    let es = lang == "es";
    let t = |en: &str, es_: &str| if es { es_.to_string() } else { en.to_string() };
    let mut cards: Vec<String> = vec![];
    let src = |kind: &str, m: &LayerMeta| if zip { format!("map-{kind}.png") } else { m.image_url.clone() };

    // Vigor
    if let Some(m) = &maps.ndvi {
        let v = reading(ev, "ndvi").and_then(|r| r["value"].as_f64());
        cards.push(format!("<figure class=\"mcard\"><h3>{}</h3><div class=\"mimg\"><img src=\"{}\" alt=\"{}\">{}</div>{}<figcaption class=\"cite\">{} · Sentinel-2 L2A · {} · 10 m · {}</figcaption></figure>",
            t("Crop vigor (NDVI)", "Vigor del cultivo (NDVI)"), esc(&src("vigor", m)), t("NDVI per 10 m pixel around the field", "NDVI por píxel de 10 m alrededor del campo"),
            outline(st, m.bbox), scale(&NDVI, v, &t("0 bare soil", "0 suelo desnudo"), &t("0.8 dense crop", "0.8 cultivo denso"), &v.map(|v| fmt_num(v, 2)).unwrap_or_default()),
            t("Each pixel coloured by its NDVI; white line = your field", "Cada píxel pintado según su NDVI; línea blanca = tu campo"),
            esc(m.date.as_deref().unwrap_or("")), t("field level", "a nivel de campo")));
    }
    // Soil moisture
    if let Some(r) = reading(ev, "soil_moisture") {
        if let Some(v) = r["value"].as_f64() {
            let km = km_of(r["resolution"].as_str().unwrap_or("")).unwrap_or(9.0);
            cards.push(format!("<figure class=\"mcard\"><h3>{}</h3>{}{}<figcaption class=\"cite\">{} · {} · {} · {}</figcaption></figure>",
                t("Soil moisture (area)", "Humedad del suelo (zona)"), cell(km, &ramp(&SOIL, v), lang),
                scale(&SOIL, Some(v), &t("dry", "seco"), &t("wet", "húmedo"), &format!("{} m³/m³", fmt_num(v, 2))),
                t("The coloured square is the whole cell this value describes: the area, not your plot", "El cuadro de color es toda la celda que describe este valor: la zona, no tu parcela"),
                esc(r["source"].as_str().unwrap_or("SMAP L4")), esc(r["date"].as_str().unwrap_or("")), t("regional context", "contexto regional")));
        }
    }
    // Temperature
    if let Some(r) = reading(ev, "temp_anomaly") {
        if let Some(v) = r["value"].as_f64() {
            let km = km_of(r["resolution"].as_str().unwrap_or("")).unwrap_or(50.0);
            let sign = if v > 0.0 { "+" } else { "" };
            cards.push(format!("<figure class=\"mcard\"><h3>{}</h3>{}{}<figcaption class=\"cite\">{} · {} · {} · {}</figcaption></figure>",
                t("Temperature vs 40-year normal (area)", "Temperatura vs. normal de 40 años (zona)"), cell(km, &ramp(&TEMP, v), lang),
                scale(&TEMP, Some(v), "−4 °C", "+6 °C", &format!("{sign}{} °C", fmt_num(v, 1))),
                t("The coloured square is the whole cell this value describes: the area, not your plot", "El cuadro de color es toda la celda que describe este valor: la zona, no tu parcela"),
                esc(r["source"].as_str().unwrap_or("NASA POWER")), esc(r["date"].as_str().unwrap_or("")), t("regional context", "contexto regional")));
        }
    }
    // Slope
    if let Some(m) = &maps.slope {
        let v = reading(ev, "slope").and_then(|r| r["value"].as_f64());
        let legend: String = SLOPE.iter().map(|(c, en, es_)| format!("<li><i style=\"background:rgb({},{},{})\"></i>{}</li>", c[0], c[1], c[2], esc(if es { es_ } else { en }))).collect();
        cards.push(format!("<figure class=\"mcard\"><h3>{}</h3><div class=\"mimg mimg--px\"><img src=\"{}\" alt=\"{}\">{}</div><ul class=\"mclass\">{}</ul><figcaption class=\"cite\">{}{} · NASADEM (NASA JPL) · 30 m · {}</figcaption></figure>",
            t("Slope", "Pendiente"), esc(&src("slope", m)), t("Slope classes per 30 m cell over about 5 km", "Clases de pendiente por celda de 30 m en unos 5 km"),
            outline(st, m.bbox), legend,
            v.map(|v| format!("{} {}° · ", t("Your field:", "Tu campo:"), fmt_num(v, 1))).unwrap_or_default(),
            t("classes grouped from FAO (2006); white line = your field", "clases agrupadas de FAO (2006); línea blanca = tu campo"), t("field level", "a nivel de campo")));
    }
    if cards.is_empty() { return String::new(); }
    format!("<h2>{}</h2><p class=\"cite\">{}</p><div class=\"maps\">{}</div>",
        t("Data maps of your field", "Mapas de datos de tu campo"),
        t("The four layers of the dashboard map. Field-level maps show your plot; the coarse cells show the area around it.", "Las cuatro capas del mapa del panel. Los mapas a nivel de campo muestran tu parcela; las celdas gruesas, la zona alrededor."),
        cards.join(""))
}

pub const CSS: &str = ".maps{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px}.mcard{margin:0;padding:12px;border:1px solid var(--rule,#cfd8df);border-radius:8px;break-inside:avoid;display:grid;gap:8px;align-content:start}.mcard h3{margin:0;font-size:15px}\
.mimg{position:relative;aspect-ratio:1/1;border-radius:6px;overflow:hidden;background:#e8eef2}.mimg img{position:absolute;inset:0;width:100%;height:100%;object-fit:fill;display:block}.mimg svg{position:absolute;inset:0;width:100%;height:100%}\
.mcell svg{width:100%;max-width:260px;display:block;margin:0 auto}\
.mscale{display:grid;gap:3px;padding-top:20px}.mscale__bar{position:relative;display:block;height:10px;border-radius:99px;border:1px solid #9aa7b1}.mscale__bar i{position:absolute;top:-4px;width:2px;height:16px;margin-left:-1px;background:#0f1d29}\
.mscale__bar b{position:absolute;bottom:17px;left:50%;transform:translateX(-50%);white-space:nowrap;font:600 11px/1.2 monospace;background:#0f1d29;color:#fff;padding:1px 5px;border-radius:4px}.mscale__ends{display:flex;justify-content:space-between;font:11px monospace;color:#556}\
.mclass{list-style:none;margin:0;padding:0;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:3px 10px;font-size:12px}.mclass li{display:flex;align-items:center;gap:6px}.mclass i{width:12px;height:12px;border-radius:2px;border:1px solid #9aa7b1;flex:none}\
@media (max-width:640px){.maps{grid-template-columns:1fr}}";
