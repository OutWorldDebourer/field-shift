# 1. Challenge brief

**Event:** 2026 NASA Space Apps Challenge, November 14-15, 2026
**Challenge:** Field Shift: Adapting Farms with NASA Data
**Difficulty:** Intermediate / Advanced. **Subjects:** Earth Science, Software, Arts & Multimedia
**Team:** InnoHuando
**Official page:** <https://www.spaceappschallenge.org/2026/challenges/adapting-farms-with-nasa-data/>

## The ask, in one sentence

Build a decision-support tool that combines NASA Earth observations with local soil information, crop characteristics and farmer priorities, so that farmers can explore crop-rotation strategies that strengthen soil health and adapt the farm to changing weather and water conditions.

## What the statement actually asks for

Everything in the statement is phrased as "you may", so nothing below is a hard requirement. Read together, though, it describes what a strong answer looks like.

| # | Element | Source in the statement |
|---|---------|-------------------------|
| 1 | Use NASA Earth observations as a core input (SMAP, ECOSTRESS, GPM, Landsat, MODIS, NASA POWER, GRACE/GRACE-FO, SRTM) | Objectives |
| 2 | Analyze temperature, heat stress, rainfall, soil moisture, evaporation, frost risk, extreme precipitation, vegetation stress, long-term water trends | Objectives |
| 3 | Combine with local soil data, crop traits and farmer priorities (water use, forage value, nitrogen fixation, erosion reduction, organic matter, market fit) | Objectives |
| 4 | Show when fields are usually bare and when current crops look most vulnerable | Objectives |
| 5 | **Compare at least three rotation strategies, one of them the existing rotation** | Objectives |
| 6 | For each strategy explain: why it was suggested, the evidence, the benefits, the information gaps, the risks needing local review | Objectives |
| 7 | Stay a supportive tool: it never replaces farmer judgment or local expertise | Objectives |

Items 5, 6 and 7 are the heart of the challenge. A tool that only shows maps and charts does not answer it.

## Optional considerations grouped by the statement

- **Understanding conditions and data.** Explain terms such as evapotranspiration, soil-moisture anomaly, NDVI, frost risk and water balance in plain language. Explain how satellite observations differ from ground measurements.
- **Modeling and data fusion.** Compare statistical or machine-learning approaches, say how uncertainty is handled, and harmonize datasets of different resolutions onto one analysis grid.
- **Rotation strategy exploration.** Compare scenarios on nutrient cycling, erosion, organic matter, pest-cycle disruption, water use, forage and market. Map strategies to the farmer's priorities. Explain the role of legumes, cover crops, deep-rooting and drought-tolerant crops.
- **Geospatial mapping.** Interactive maps of long-term trends, field-level vulnerability zones, timelines, overlays, scenario sliders for rainfall or temperature.
- **Farmer-centered design.** Readable by people with little technical background, lets the farmer enter priorities and constraints (water, equipment, market), and keeps reminding the user where human judgment matters.

## A note on vocabulary

The Space Apps site writes "SI" (Super Intelligence) where most people write "AI". Its FAQ says this follows a U.S. Executive Order of September 29, 2026. We use the site's term on the submission form and "AI/ML" in our own technical notes.

## What the official Resources tab offers

The tab lists 24 resources. They fall into three groups, and the grouping matters for where we pilot.

**Global coverage (usable anywhere, including Peru)**

| Resource | What it gives | Link |
|----------|---------------|------|
| SMAP | Surface soil moisture and freeze/thaw every 2-3 days | <https://www.earthdata.nasa.gov/data/platforms/space-based-platforms/smap> |
| GPM / IMERG | Precipitation rate and accumulation | <https://gpm.nasa.gov/data> |
| Landsat | 30 m reflectance, NDVI, bare ground | <https://earthexplorer.usgs.gov/> |
| MODIS / VIIRS | 250-1000 m vegetation indices | <https://lpdaac.usgs.gov/products/> |
| NASA POWER | Agroclimatology: temperature, humidity, radiation, derived indicators | <https://power.larc.nasa.gov/> |
| SRTM | Elevation, slope, runoff, erosion risk | <https://www.earthdata.nasa.gov/data/instruments/srtm> |
| ECOSTRESS | Land-surface temperature and evapotranspiration at fine scale | <https://ecostress.jpl.nasa.gov/data> |
| GLDAS | Assimilated temperature, rain, soil moisture, ET proxies | <https://www.earthdata.nasa.gov/> |
| GEDI | Canopy height and biomass (best over tall vegetation) | <https://gedi.umd.edu/data> |
| FAO GAEZ | Crop suitability, soil constraints, yield potential | <https://gaez.fao.org/> |
| Copernicus Sentinel-2 (ESA) | 10-20 m multispectral imagery | <https://dataspace.copernicus.eu/> |
| ERA5 (ECMWF) | Hourly reanalysis climate variables | <https://cds.climate.copernicus.eu/datasets/reanalysis-era5-single-levels> |

**United States only**

USDA Ag Data Commons, Web Soil Survey, Cropland Data Layer (CropScape), PRISM, SSURGO/STATSGO2, NAIP, NLCD, NLDAS, NWIS, USDA AMS Market News, NASS QuickStats, OpenET (western states), LANDFIRE.

**North America only**

CoCoRaHS crowd-sourced rainfall.

### Why this matters

More than half of the listed resources cover only the United States. A pilot in Peru can use every NASA mission dataset, but has to replace the U.S. soil, crop-map, price and station sources with Peruvian or global equivalents. See [6. Open decisions](06-open-decisions.md).

## Sources

- Challenge statement and Resources tab, read on 2026-11-14: <https://www.spaceappschallenge.org/2026/challenges/adapting-farms-with-nasa-data/?tab=resources>
- Participant FAQs: <https://www.spaceappschallenge.org/resources/participant-faqs/>
