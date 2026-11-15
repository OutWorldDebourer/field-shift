# 7. Referencias y licencias

Todas las fuentes que usa el producto, con su licencia. El código propio de Field Shift es Apache-2.0 (`LICENSE`).

## Datos de la NASA (dominio público o sin restricciones; citar el conjunto de datos)

| Conjunto de datos | Cita / DOI | Licencia |
|-------------------|------------|----------|
| NASA POWER diario, comunidad AG | NASA Langley Research Center POWER Project, https://power.larc.nasa.gov | Datos abiertos de la NASA, sin restricciones; citar NASA POWER |
| SMAP L4 global trihorario 9 km (SPL4SMGP) v8 | Reichle et al., NSIDC DAAC (DOI en cada lectura) | Datos de ciencias de la Tierra de la NASA, sin restricciones |
| GPM IMERG Late L3 diario 0,1° v07 (GPM_3IMERGDL) | Huffman et al., GES DISC, DOI 10.5067/GPM/IMERGDL/DAY/07 | Sin restricciones |
| GRACE/GRACE-FO mascon JPL CRI RL06.3 v4 | Watkins et al. 2015; Wiese et al. 2016; PO.DAAC | Sin restricciones |
| Pronóstico NASA GEOS-FP (GMAO), tavg1_2d_slv_Nx y tavg1_2d_flx_Nx | NASA Global Modeling and Assimilation Office, vía OPeNDAP de NCCS | Datos de la NASA, sin restricciones; citar NASA GMAO |
| NASADEM | NASA JPL, vía Microsoft Planetary Computer | Dominio público |
| NEX-GDDP-CMIP6 | Thrasher et al. 2022, Scientific Data; NASA NCCS | CC BY-SA 4.0 según cada archivo de modelo v2.0 (la más estricta de las dos declaraciones de la NASA; la nota técnica v1 dice CC0). 13 modelos, uno por institución; DOI del conjunto 10.7917/OFSG3345, DOI del artículo 10.1038/s41597-022-01393-4. Los números derivados se comparten igual |
| Blue Marble Next Generation, VIIRS Black Marble, reflectancia corregida VIIRS | NASA GIBS / ESDIS | Dominio público, crédito NASA |

## Otras imágenes satelitales (abiertas, con atribución)

| Fuente | Crédito mostrado | Licencia |
|--------|------------------|----------|
| Copernicus Sentinel-2 L2A (Planetary Computer) | "Contains modified Copernicus Sentinel data 2026, processed by Microsoft Planetary Computer" | Aviso legal de datos Sentinel de Copernicus (libre, completo, abierto) |
| Landsat 8/9 Colección 2 L2 (USGS) | "Landsat courtesy of the U.S. Geological Survey" | Dominio público |
| Sentinel-2 cloudless 2016 de EOX IT Services GmbH (capa base del mapa) | "Sentinel-2 cloudless 2016 by EOX IT Services GmbH (Contains modified Copernicus Sentinel data 2016)" | CC BY 4.0 (solo la edición 2016) |
| OpenStreetMap Nominatim (búsqueda de lugares) | "© OpenStreetMap contributors" | ODbL 1.0; se respeta la política de uso (búsqueda solo con Enter, ≤1 solicitud/s, con caché) |
| Teselas vectoriales de OpenStreetMap vía OpenFreeMap (vista "Híbrido": caminos, ríos y nombres de pueblos sobre el satélite) | "© OpenStreetMap contributors · OpenFreeMap" | Datos ODbL 1.0; OpenFreeMap es gratuito, sin clave ni cuenta; dibujado con MapLibre GL JS (BSD-3-Clause) |
| Teselas estándar de OpenStreetMap (vista "Mapa" en la selección del campo) | "© OpenStreetMap contributors (ODbL)" | Datos ODbL 1.0; uso interactivo ligero según la política de teselas de OSM |

## Métodos y umbrales

- Allen, R.G., Pereira, L.S., Raes, D., Smith, M. (1998). *Evapotranspiración del cultivo*, Estudio FAO Riego y Drenaje 56 (ET0 de referencia, valores Kc). Términos FAO: reutilización no comercial con reconocimiento.
- Zalom, F.G. et al. (1983). Degree-days: the calculation and use of heat units in pest management. UC DANR Leaflet 21373 (método seno simple).
- Whalon, M.E. y Smilowitz, Z. (1979). Modelo del pulgón verde del duraznero. *The Canadian Entomologist* 111:1025–1032. DOI 10.4039/Ent1111025-9.
- Bosco, D. y Caciagli, P. (1998). Bionomía y ecología de *Bemisia tabaci* en Italia. *European Journal of Entomology* 95(4):519–527.
- du Plessis, H., Schlemmer, M.-L. y Van den Berg, J. (2020). Efecto de la temperatura en el desarrollo de *Spodoptera frugiperda*. *Insects* 11(4):228. DOI 10.3390/insects11040228 (CC BY 4.0).
- Base de conocimiento de cultivos: 103 fuentes (FAO-56, FAO Ecocrop, UC IPM, SARE, INIA, SENASA, UNALM y otras), cada una con su licencia en `backend/crates/crops/data/README.md`. La iteración 4 añade 41 fuentes para las condiciones de plagas, micronutrientes y bioestimulantes de los cinco cultivos de Huaral (investigadas por IA, verificar), listadas en el mismo archivo.
- Pronóstico NASA GMAO GEOS-FP (usado por los semáforos de Prevención): https://gmao.gsfc.nasa.gov/GMAO_products/NRT_products.php (datos abiertos de la NASA).
- Entrevista a la agrónoma Ing. Cenia (2026), `Entrevista/Ing. Cenia.txt` (temas de suelo, sanidad y nutrición).

## Software, tipografías y referencias de diseño

- Crates de Rust (axum, tokio, sqlx, reqwest, serde; MIT/Apache-2.0), MariaDB 11.4 (GPL-2.0, como servicio aparte), Vite y TypeScript (MIT/Apache-2.0), three.js (MIT), Leaflet (BSD-2-Clause).
- Tipografías: Source Serif 4, Source Sans 3, IBM Plex Mono (SIL Open Font License 1.1), alojadas localmente desde Fontsource.
- Técnicas del globo adaptadas del motor de demostración Satellite Briefing de Lemo-Opuscar, © 2026 Lemomo/LemoLab, MIT (`frontend/src/globe/LICENSE-lemolab-engine.txt`). Las guías de estilo de Lemo-Opuscar (CC BY-NC 4.0) se leyeron solo como referencia de diseño; ningún texto suyo está en el producto.
