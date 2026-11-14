# 4. Datos de la NASA y satelitales en el producto

Cada lectura de Field Shift lleva su fuente, fecha, resolución, licencia y una etiqueta de escala. Las lecturas de **nivel de campo** miden la parcela dibujada; las de **contexto regional** describen la zona y nunca se presentan como una medición de la parcela.

| Lectura | Fuente y conjunto de datos | Acceso | Resolución · etiqueta | Lo que ve el agricultor |
|---------|----------------------------|--------|-----------------------|--------------------------|
| Temperatura vs normal de 40 años, lluvia, evapotranspiración de referencia (FAO-56), balance hídrico, días de estrés por calor, grados-día | API diaria de NASA POWER, comunidad AG (MERRA-2, CERES SYN1deg; últimas semanas de flujos casi en tiempo real GEOS-IT / FLASHFlux, marcadas como provisionales) | Sin cuenta | ~0,5° × 0,625° (~50 km) · regional | "Mucho más cálido que lo normal (+5,0 °C)", "el riego debe cubrir este déficit" |
| Generaciones de plagas (grados-día) | Temperatura NASA POWER + umbrales publicados (plagas del propio cultivo según la base de conocimiento; plagas de referencia: *Myzus persicae*, *Bemisia tabaci*, *Spodoptera frugiperda*) | Sin cuenta | ~50 km · regional | "Pulgón de la arveja: 13,3 vs 10,3 generaciones esta campaña" |
| Serie de vigor (NDVI), periodos de suelo desnudo, ciclos | Sentinel-2 L2A (10 m) y Landsat 8/9 Colección 2 L2 (30 m), media sin nubes dentro del contorno | Microsoft Planetary Computer, sin cuenta | 10–30 m · nivel de campo | "Vigor alto (0,78)", suelo desnudo nov–dic 2025 |
| Imagen en color real del campo | Sentinel-2 L2A, activo `visual`, escena despejada más reciente | Planetary Computer | 10 m · nivel de campo | El campo bajo la interfaz de vidrio |
| Pendiente y altitud | NASADEM (SRTM reprocesado), pendiente de Horn en el contorno | Planetary Computer | 30 m · nivel de campo | "Casi plano (~3 %)" |
| Humedad del suelo (5 cm y zona radicular) vs la misma fecha en años anteriores | SMAP L4 SPL4SMGP v8 (NSIDC), OPeNDAP en la nube | Earthdata Login | 9 km · regional | "Similar a la misma fecha en años anteriores" |
| Lluvia satelital, últimos 30 días | GPM IMERG Late diario v07 (GES DISC), OPeNDAP | Earthdata Login | 0,1° (~10 km) · regional | "2 días con lluvia, 4,4 mm" |
| Agua almacenada a largo plazo | GRACE/GRACE-FO mascon JPL CRI RL06.3 v4 (PO.DAAC), extraída por sitio de ejemplo | Earthdata Login | ~300 km · regional | "Se humedece lentamente desde 2002" |
| Proyección climática ~2040 | NEX-GDDP-CMIP6 (NASA), solo modelos con licencia abierta | Planetary Computer | 0,25° · regional | "Proyección, no pronóstico" (solo donde se calculó) |
| Globo | NASA Blue Marble Next Generation, VIIRS Black Marble, VIIRS color real (GIBS); placas Sentinel-2 | GIBS, Planetary Computer | — | El descenso desde la órbita hasta el campo |

## Reglas de honestidad en el código

- Un valor sin fuente es un vacío de información, nunca una suposición (base de conocimiento de cultivos y paquete de evidencia).
- El consultor solo puede citar números del paquete de evidencia; cada número se muestra con fuente y fecha.
- Los días recientes de POWER se marcan como provisionales; un contraste con ERA5 confirmó el 2026 cálido (ver `research/01_api_tests.md`).
- Las generaciones de plagas solo se calculan donde existe un umbral de grados-día publicado; si no, la guía es cualitativa.
