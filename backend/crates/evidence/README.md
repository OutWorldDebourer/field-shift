# evidence

Analyst stage: one fixed, cited evidence package per analysis (the consultant may only cite its numbers).

| File | What it does |
|------|--------------|
| `src/lib.rs` | `build`: runs POWER, imagery (and Earthdata, climate) in parallel, stores the package, marks "Your field" |
| `src/reading.rs` | Reading shape of the frontend contract and plain-words verdicts |
| `src/from_power.rs` | POWER summary → temperature, rain, ET0, water balance, heat days, GDD readings; monthly chart rows |
| `src/crop_pests.rs` | Generations of one crop's pests from their published thresholds (single sine, POWER temperature, 1985–2024 normal) |
| `src/pests.rs` | Pests of the field's own crops (current, previous, planned) with generations where a threshold is published; `pest_gen` reading named after the leading pest; reference pests only while no crop is known |
| `src/from_imagery.rs` | NDVI (Sentinel-2/Landsat) and NASADEM slope → field-level readings; NDVI series |
| `src/advise.rs` | Any crop → crop records (crops crate); cited advice → Soil, Pest care, Nutrition, Rotation / perennial system sections |
| `src/intake.rs` | "Load information": structured answers complete the topics they cover, build evidence and advice (`POST /api/analyses/{id}/intake`) |
| `src/organic.rs` | Organic section: cited natural enemies and beneficial microorganisms (this crop + matches for reported pests across the knowledge base), practices, timing from NASA data, what harms them |
