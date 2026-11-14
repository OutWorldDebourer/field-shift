# evidence

Analyst stage: one fixed, cited evidence package per analysis (the consultant may only cite its numbers).

| File | What it does |
|------|--------------|
| `src/lib.rs` | `build`: runs POWER, imagery (and Earthdata, climate) in parallel, stores the package, marks "Your field" |
| `src/reading.rs` | Reading shape of the frontend contract and plain-words verdicts |
| `src/from_power.rs` | POWER summary → temperature, rain, ET0, water balance, heat days, GDD, aphid generations; monthly chart rows; pest table |
| `src/from_imagery.rs` | NDVI (Sentinel-2/Landsat) and NASADEM slope → field-level readings; NDVI series |
| `src/advise.rs` | Crop records and advice into sections |
