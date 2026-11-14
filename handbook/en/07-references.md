# 7. References and licences

Every source used by the product, with its licence. Field Shift's own code is Apache-2.0 (`LICENSE`).

## NASA data (public domain or no restrictions; cite the dataset)

| Dataset | Citation / DOI | Licence |
|---------|----------------|---------|
| NASA POWER daily, AG community | NASA Langley Research Center POWER Project, https://power.larc.nasa.gov | NASA open data, no restrictions; cite NASA POWER |
| SMAP L4 Global 3-hourly 9 km Soil Moisture (SPL4SMGP) v8 | Reichle et al., NSIDC DAAC (dataset DOI shown with each reading) | NASA Earth science data, no restrictions |
| GPM IMERG Late Precipitation L3 1 day 0.1° v07 (GPM_3IMERGDL) | Huffman et al., GES DISC, DOI 10.5067/GPM/IMERGDL/DAY/07 | NASA Earth science data, no restrictions |
| GRACE/GRACE-FO JPL Mascon CRI RL06.3 v4 | Watkins et al. 2015; Wiese et al. 2016; PO.DAAC | NASA Earth science data, no restrictions |
| NASADEM | NASA JPL, via Microsoft Planetary Computer | Public domain |
| NEX-GDDP-CMIP6 | Thrasher et al. 2022, Scientific Data; NASA NCCS | CC BY-SA 4.0 as stated in each v2.0 model file (the stricter of NASA's two statements; the v1 tech note says CC0). 13 models, one per institution; dataset DOI 10.7917/OFSG3345, paper DOI 10.1038/s41597-022-01393-4. Derived numbers are shared alike |
| Blue Marble Next Generation, VIIRS Black Marble, VIIRS corrected reflectance | NASA GIBS / ESDIS ("We acknowledge the use of imagery provided by services from NASA's Global Imagery Browse Services (GIBS), part of NASA's ESDIS") | Public domain, credit NASA |

## Other satellite imagery (open, attribution)

| Source | Credit shown | Licence |
|--------|--------------|---------|
| Copernicus Sentinel-2 L2A (Planetary Computer) | "Contains modified Copernicus Sentinel data 2026, processed by Microsoft Planetary Computer" | Copernicus Sentinel data legal notice (free, full, open) |
| Landsat 8/9 Collection 2 L2 (USGS) | "Landsat courtesy of the U.S. Geological Survey" | Public domain |
| Sentinel-2 cloudless 2016 by EOX IT Services GmbH (map base layer) | "Sentinel-2 cloudless 2016 by EOX IT Services GmbH (Contains modified Copernicus Sentinel data 2016)" | CC BY 4.0 (2016 edition only) |
| OpenStreetMap Nominatim (place search) | "© OpenStreetMap contributors" | ODbL 1.0; usage policy respected (≤1 request/s, debounced) |

## Methods and thresholds

- Allen, R.G., Pereira, L.S., Raes, D., Smith, M. (1998). *Crop evapotranspiration*, FAO Irrigation and Drainage Paper 56 (reference ET0, Kc values). FAO terms: non-commercial reuse with acknowledgement.
- Zalom, F.G. et al. (1983). Degree-days: the calculation and use of heat units in pest management. UC DANR Leaflet 21373 (single-sine method).
- Whalon, M.E. & Smilowitz, Z. (1979). Temperature-dependent model for predicting field populations of green peach aphid. *The Canadian Entomologist* 111:1025–1032. DOI 10.4039/Ent1111025-9.
- Bosco, D. & Caciagli, P. (1998). Bionomics and ecology of *Bemisia tabaci* in Italy. *European Journal of Entomology* 95(4):519–527.
- du Plessis, H., Schlemmer, M.-L. & Van den Berg, J. (2020). The effect of temperature on the development of *Spodoptera frugiperda*. *Insects* 11(4):228. DOI 10.3390/insects11040228 (CC BY 4.0).
- Crop knowledge base: 103 sources (FAO-56, FAO Ecocrop, UC IPM, SARE, INIA, SENASA, UNALM and others), each listed with its licence in `backend/crates/crops/data/README.md`.
- Agronomist interview: Ing. Cenia (2026), `Entrevista/Ing. Cenia.txt` (themes of the soil, pest care and nutrition advice).

## Software, fonts and design references

- Rust crates (axum, tokio, sqlx, reqwest, serde; MIT/Apache-2.0), MariaDB 11.4 (GPL-2.0, as a separate service), Vite and TypeScript (MIT/Apache-2.0), three.js (MIT), Leaflet (BSD-2-Clause).
- Fonts: Source Serif 4, Source Sans 3, IBM Plex Mono (SIL Open Font License 1.1), self-hosted from Fontsource.
- Globe engine techniques adapted from the Lemo-Opuscar Satellite Briefing demo engine, © 2026 Lemomo/LemoLab, MIT (`frontend/src/globe/LICENSE-lemolab-engine.txt`). The Lemo-Opuscar style guides (CC BY-NC 4.0) were read as design references only; no text from them is in the product.
