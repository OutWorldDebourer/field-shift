//! "What does it mean?": the technical words and acronyms the app and the sheet use, in plain English and Spanish.
//! One source for GET /api/glossary and the sheet's annex.
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Bi { pub en: &'static str, pub es: &'static str }

#[derive(Clone, Debug, Serialize)]
pub struct Term {
    pub key: &'static str,
    pub term: Bi,
    pub short: Bi,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<Bi>,
}

const fn b(en: &'static str, es: &'static str) -> Bi { Bi { en, es } }

pub fn terms() -> Vec<Term> {
    vec![
        Term { key: "ndvi", term: b("Crop vigor (NDVI)", "Vigor del cultivo (NDVI)"),
            short: b("How green and leafy the plot looks from space, measured by satellites (Sentinel-2, Landsat). It is an index, not a length.", "Qué tan verde y frondosa se ve la parcela desde el espacio, medido por satélites (Sentinel-2, Landsat). Es un índice, no una medida de longitud."),
            scale: Some(b("0 to 1: below 0.2 bare soil · 0.2–0.4 sparse or young crop · 0.4–0.6 moderate · above 0.6 dense and vigorous", "0 a 1: menos de 0.2 suelo desnudo · 0.2–0.4 cultivo ralo o joven · 0.4–0.6 moderado · más de 0.6 denso y vigoroso")) },
        Term { key: "et0", term: b("Reference evapotranspiration (ET₀)", "Evapotranspiración de referencia (ET₀)"),
            short: b("The water the air 'pulls' from a well-watered reference crop, from heat, sun, wind and humidity (FAO-56 method). More ET₀ means the crop needs more water.", "El agua que el aire 'jala' de un cultivo de referencia bien regado, según calor, sol, viento y humedad (método FAO-56). Más ET₀ significa que el cultivo necesita más agua."), scale: Some(b("millimetres of water; 1 mm = 10 m³ per hectare", "milímetros de agua; 1 mm = 10 m³ por hectárea")) },
        Term { key: "water_balance", term: b("Water balance", "Balance hídrico"),
            short: b("Rain minus ET₀. A negative number is the water irrigation has to supply (irrigation itself is not modelled).", "Lluvia menos ET₀. Un número negativo es el agua que debe aportar el riego (el riego en sí no se modela)."), scale: None },
        Term { key: "gdd", term: b("Growing degree-days", "Grados-día de crecimiento"),
            short: b("Accumulated warmth above a base temperature. Crops and insects develop according to it: more degree-days, faster development.", "Calor acumulado por encima de una temperatura base. Los cultivos y los insectos se desarrollan según él: más grados-día, desarrollo más rápido."), scale: Some(b("°C·day", "°C·día")) },
        Term { key: "pest_gen", term: b("Pest generations", "Generaciones de plaga"),
            short: b("How many life cycles a pest can complete with the season's warmth, from published lab thresholds. More generations than normal means more pressure.", "Cuántos ciclos de vida puede completar una plaga con el calor de la campaña, según umbrales publicados. Más generaciones que lo normal significa más presión."), scale: None },
        Term { key: "regional", term: b("Regional context vs field-level", "Contexto regional vs nivel de campo"),
            short: b("Field-level readings measure your plot (10–30 m pixels). Regional readings describe the surrounding area (9 to 300 km) and are never a measurement of your plot.", "Las lecturas de nivel de campo miden tu parcela (píxeles de 10–30 m). Las regionales describen la zona (9 a 300 km) y nunca son una medición de tu parcela."), scale: None },
        Term { key: "provisional", term: b("Provisional", "Provisional"),
            short: b("The latest weeks of NASA POWER come from near-real-time streams and can be revised later.", "Las últimas semanas de NASA POWER vienen de datos casi en tiempo real y pueden corregirse después."), scale: None },
        Term { key: "normal", term: b("40-year normal", "Normal de 40 años"),
            short: b("The average of the same dates in 1985–2024, used as the yardstick for 'warmer' or 'drier than normal'.", "El promedio de las mismas fechas en 1985–2024, la regla para decir 'más cálido' o 'más seco que lo normal'."), scale: None },
        Term { key: "power", term: b("NASA POWER", "NASA POWER"),
            short: b("NASA's daily weather and solar data for agriculture since 1981 (about 50 km cells).", "Datos diarios de clima y radiación solar de la NASA para agricultura desde 1981 (celdas de unos 50 km)."), scale: None },
        Term { key: "smap", term: b("SMAP", "SMAP"),
            short: b("NASA satellite mission for soil moisture; here its 9 km model product for the top 5 cm and the root zone.", "Misión satelital de la NASA para humedad del suelo; aquí su producto de 9 km para los 5 cm superiores y la zona de raíces."), scale: Some(b("m³ of water per m³ of soil (0.05 is very dry, 0.4 is wet)", "m³ de agua por m³ de suelo (0.05 muy seco, 0.4 húmedo)")) },
        Term { key: "imerg", term: b("GPM IMERG", "GPM IMERG"),
            short: b("NASA's satellite rain estimate every day for ~10 km cells; light coastal drizzle is often missed.", "Estimación satelital de lluvia diaria de la NASA en celdas de ~10 km; suele no captar la garúa costera."), scale: None },
        Term { key: "grace", term: b("GRACE / GRACE-FO", "GRACE / GRACE-FO"),
            short: b("Twin satellites that weigh the water stored in a region (soil, groundwater, snow) from tiny changes in gravity, over ~300 km.", "Satélites gemelos que 'pesan' el agua almacenada en una región (suelo, agua subterránea, nieve) por cambios mínimos de gravedad, en ~300 km."), scale: Some(b("centimetres of water above or below the 2004–2009 average", "centímetros de agua por encima o debajo del promedio 2004–2009")) },
        Term { key: "geosfp", term: b("NASA GEOS-FP forecast", "Pronóstico NASA GEOS-FP"),
            short: b("NASA's operational weather model (GMAO); we read its next 10 days for your area (~25 km). A forecast, so it can change.", "El modelo meteorológico operativo de la NASA (GMAO); leemos sus próximos 10 días para tu zona (~25 km). Es un pronóstico: puede cambiar."), scale: None },
        Term { key: "outlook", term: b("Season outlook", "Perspectiva de la campaña"),
            short: b("Our estimate for the coming months from the 40-year normals, with two scenarios: back to normal, or as warm as now. An estimate, not a forecast.", "Nuestra estimación para los próximos meses con las normales de 40 años, en dos escenarios: vuelve a lo normal, o sigue tan cálido como ahora. Una estimación, no un pronóstico."), scale: None },
        Term { key: "nexgddp", term: b("NEX-GDDP-CMIP6 projection", "Proyección NEX-GDDP-CMIP6"),
            short: b("NASA's downscaled climate projections from 13 global models for 2031–2050. A projection of the climate, not a weather forecast.", "Proyecciones climáticas de la NASA a partir de 13 modelos globales para 2031–2050. Una proyección del clima, no un pronóstico del tiempo."), scale: None },
        Term { key: "ssp", term: b("SSP2-4.5 / SSP5-8.5", "SSP2-4.5 / SSP5-8.5"),
            short: b("Emission scenarios: SSP2-4.5 is a middle road; SSP5-8.5 is very high emissions.", "Escenarios de emisiones: SSP2-4.5 es un camino intermedio; SSP5-8.5 son emisiones muy altas."), scale: None },
        Term { key: "sentinel2", term: b("Sentinel-2 / Landsat", "Sentinel-2 / Landsat"),
            short: b("Earth-observation satellites (ESA Copernicus and NASA/USGS) that photograph every field every few days at 10–30 m.", "Satélites de observación (Copernicus de la ESA y NASA/USGS) que fotografían cada campo cada pocos días a 10–30 m."), scale: None },
        Term { key: "nasadem", term: b("NASADEM", "NASADEM"),
            short: b("NASA's elevation model of the land surface (30 m), used for the slope of your plot.", "Modelo de elevación del terreno de la NASA (30 m), usado para la pendiente de tu parcela."), scale: None },
        Term { key: "prediagnosis", term: b("Pre-diagnosis", "Prediagnóstico"),
            short: b("A starting point built from data and your answers. Your agronomist confirms or corrects it before you act.", "Un punto de partida hecho con datos y tus respuestas. Tu agrónomo lo confirma o corrige antes de actuar."), scale: None },
    ]
}
