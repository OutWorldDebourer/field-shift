"""Climate projection at one point from NASA NEX-GDDP-CMIP6 (v2.0 files on NCCS THREDDS), written as JSON.

Usage (from the project root, standard library only):
  tools/.venv/Scripts/python.exe -I tools/nex_gddp_point.py <site_id> <site_name> <lat> <lon> <out.json>
  e.g. ... huaral "Huaral valley, Peru" -11.47 -77.21 backend/data/projection_huaral.json

Method: for each model and experiment, one NCSS point request per variable and year returns the daily values of
the 0.25 degree cell containing the point (about 15 kB each; no full-globe files are downloaded). Annual means of
tas (deg C) and annual totals of pr (mm) are averaged over the baseline 1995-2014 (historical) and the window
2031-2050 (SSP2-4.5, SSP5-8.5). Change = window - baseline per model; the ensemble gives median, min and max.

License: NASA's NEX-GDDP-CMIP6 tech note (v1, rev. 2022) states a blanket CC0 for all outputs; the v2.0 files carry
the global attribute cmip6_license = "CC-BY-SA 4.0", which the v2 tech note says governs. We record the attribute
read from each model's file and follow the stricter one (attribution + share-alike for derived numbers).
"""
import csv
import io
import json
import re
import socket
import statistics
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor

BASE = "https://ds.nccs.nasa.gov/thredds"
ROOT = "AMES/NEX/GDDP-CMIP6"
MODELS = {  # model -> variant (NCCS tech note Table 1); a spread of institutions, all with SSP2-4.5 and SSP5-8.5
    "ACCESS-CM2": "r1i1p1f1", "CanESM5": "r1i1p1f1", "CMCC-ESM2": "r1i1p1f1", "EC-Earth3": "r1i1p1f1",
    "GFDL-ESM4": "r1i1p1f1", "GISS-E2-1-G": "r1i1p1f2", "INM-CM5-0": "r1i1p1f1", "IPSL-CM6A-LR": "r1i1p1f1",
    "MIROC6": "r1i1p1f1", "MPI-ESM1-2-HR": "r1i1p1f1", "MRI-ESM2-0": "r1i1p1f1", "NorESM2-MM": "r1i1p1f1",
    "UKESM1-0-LL": "r1i1p1f2",
}
UPSTREAM_LICENSE = {m: "CC BY 4.0" for m in MODELS} | {"GISS-E2-1-G": "CC0 1.0"}
PERIODS = {"historical": range(1995, 2015), "ssp245": range(2031, 2051), "ssp585": range(2031, 2051)}
VARS = ("tas", "pr")
UA = {"User-Agent": "FieldShift-tools/0.1"}

# NCCS answers slowly over IPv6 from some networks (~20 s connect fallback); resolve IPv4 only.
_getaddrinfo = socket.getaddrinfo
socket.getaddrinfo = lambda host, port, family=0, *a, **k: _getaddrinfo(host, port, socket.AF_INET, *a, **k)


def get(url: str, tries: int = 4) -> str:
    for i in range(tries):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=120) as r:
                return r.read().decode("utf-8")
        except Exception:
            if i == tries - 1:
                raise
            time.sleep(2 * (i + 1))
    return ""


def files_by_year(model: str, variant: str, exp: str, var: str) -> dict:
    xml = get(f"{BASE}/catalog/{ROOT}/{model}/{exp}/{variant}/{var}/catalog.xml")
    out = {}
    for name in re.findall(r'name="([^"]+\.nc)"', xml):
        m = re.search(r"_(\d{4})(_v[\d.]+)?\.nc$", name)
        if m:
            year = int(m.group(1))
            out[year] = max(out.get(year, ""), name)  # highest version suffix wins
    return out


def point_year(model, variant, exp, var, fname, lat, lon):
    url = (f"{BASE}/ncss/grid/{ROOT}/{model}/{exp}/{variant}/{var}/{fname}"
           f"?var={var}&latitude={lat}&longitude={lon}&temporal=all&accept=csv")
    rows = list(csv.reader(io.StringIO(get(url))))[1:]
    vals = [float(r[4]) for r in rows if len(r) >= 5 and r[4] not in ("", "NaN")]
    cell = (float(rows[0][2]), float(rows[0][3])) if rows else None
    if var == "tas":
        return statistics.fmean(vals) - 273.15, cell, len(vals)
    return sum(vals) * 86400.0, cell, len(vals)  # kg m-2 s-1 daily means -> mm per year


def license_of(model, variant, fname):
    das = get(f"{BASE}/dodsC/{ROOT}/{model}/historical/{variant}/tas/{fname}.das")
    m = re.search(r'cmip6_license "([^"]+)"', das)
    v = re.search(r'String version "([^"]+)"', das)
    return (m.group(1) if m else None), (v.group(1) if v else None)


def summary(xs):
    xs = sorted(xs)
    return {"median": round(statistics.median(xs), 2), "min": round(xs[0], 2), "max": round(xs[-1], 2), "n": len(xs)}


def main(site_id, site_name, lat, lon, out):
    jobs, catalogs = [], {}
    for model, variant in MODELS.items():
        for exp, years in PERIODS.items():
            for var in VARS:
                fy = files_by_year(model, variant, exp, var)
                catalogs[(model, exp, var)] = fy
                for y in years:
                    if y in fy:
                        jobs.append((model, variant, exp, var, y, fy[y]))
    print(f"{len(jobs)} point requests", flush=True)
    results, cell = {}, None
    with ThreadPoolExecutor(max_workers=4) as pool:
        futs = {pool.submit(point_year, m, v, e, var, f, lat, lon): (m, e, var, y) for m, v, e, var, y, f in jobs}
        for i, (fut, key) in enumerate(futs.items()):
            try:
                val, c, n = fut.result()
                if n >= 360:
                    results[key] = val
                    cell = cell or c
            except Exception as exc:
                print("failed", key, type(exc).__name__, flush=True)
            if i % 100 == 0:
                print(f"{i}/{len(jobs)}", flush=True)

    models_out, deltas = [], {s: {"tas": [], "pr_mm": [], "pr_pct": []} for s in ("ssp245", "ssp585")}
    for model, variant in MODELS.items():
        def mean_of(exp, var):
            vals = [results[(model, exp, var, y)] for y in PERIODS[exp] if (model, exp, var, y) in results]
            return (statistics.fmean(vals), len(vals)) if len(vals) >= 15 else (None, len(vals))
        hist_t, nt = mean_of("historical", "tas")
        hist_p, np_ = mean_of("historical", "pr")
        hist_file = catalogs[(model, "historical", "tas")].get(2014)
        lic, ver = license_of(model, variant, hist_file) if hist_file else (None, None)
        entry = {"model": model, "variant": variant, "file_license": lic, "file_version": ver,
                 "upstream_cmip6_license": UPSTREAM_LICENSE[model],
                 "baseline": {"tas_c": round(hist_t, 2) if hist_t is not None else None,
                              "pr_mm_year": round(hist_p, 1) if hist_p is not None else None, "years": [nt, np_]},
                 "scenarios": {}}
        for s in ("ssp245", "ssp585"):
            ft, _ = mean_of(s, "tas")
            fp, _ = mean_of(s, "pr")
            if None in (hist_t, hist_p, ft, fp):
                continue
            d = {"tas_change_c": round(ft - hist_t, 2), "pr_change_mm_year": round(fp - hist_p, 1),
                 "pr_change_pct": round(100 * (fp - hist_p) / hist_p, 1) if hist_p > 0 else None,
                 "tas_c": round(ft, 2), "pr_mm_year": round(fp, 1)}
            entry["scenarios"][s] = d
            deltas[s]["tas"].append(d["tas_change_c"])
            deltas[s]["pr_mm"].append(d["pr_change_mm_year"])
            if d["pr_change_pct"] is not None:
                deltas[s]["pr_pct"].append(d["pr_change_pct"])
        models_out.append(entry)

    base_t = [m["baseline"]["tas_c"] for m in models_out if m["baseline"]["tas_c"] is not None]
    base_p = [m["baseline"]["pr_mm_year"] for m in models_out if m["baseline"]["pr_mm_year"] is not None]
    doc = {
        "site": {"id": site_id, "name": site_name, "lat": lat, "lon": lon},
        "cell_center": {"lat": cell[0], "lon": cell[1]} if cell else None,
        "dataset": "NASA Earth Exchange Global Daily Downscaled Projections (NEX-GDDP-CMIP6)",
        "dataset_version": "2.0 (NCCS THREDDS files *_v2.0.nc)",
        "doi": "10.7917/OFSG3345",
        "paper_doi": "10.1038/s41597-022-01393-4",
        "resolution": "0.25 degree (~25 km), daily, BCSD downscaled",
        "badge": "regional context",
        "label": "projection, not a forecast",
        "baseline_period": "1995-2014 (historical experiment)",
        "future_period": "2031-2050 (centred on ~2040)",
        "variables": {"tas": "annual mean near-surface air temperature, deg C",
                      "pr": "annual total precipitation, mm/year"},
        "ensemble": {
            "baseline": {"tas_c": summary(base_t), "pr_mm_year": summary(base_p)},
            **{s: {"tas_change_c": summary(v["tas"]), "pr_change_mm_year": summary(v["pr_mm"]),
                   "pr_change_pct": summary(v["pr_pct"]) if v["pr_pct"] else None}
               for s, v in deltas.items() if v["tas"]},
        },
        "models": models_out,
        "license": {
            "nasa_statement": "NCCS NEX-GDDP-CMIP6 tech note: as of Sept 2022 all outputs under a blanket CC0 license",
            "nasa_statement_url": "https://www.nccs.nasa.gov/wp-content/uploads/2024/03/NEX-GDDP-CMIP6-Tech_Note_4.pdf",
            "file_attribute": "v2.0 files carry cmip6_license = 'CC-BY-SA 4.0'; the v2 tech note defers to this attribute",
            "file_attribute_url": "https://www.nccs.nasa.gov/wp-content/uploads/2025/06/NEX-GDDP-CMIP6-v2-Tech_Note.pdf",
            "upstream_url": "https://wcrp-cmip.github.io/CMIP6_CVs/docs/CMIP6_source_id_licenses.html",
            "applied": "CC BY-SA 4.0 (strictest stated; attribution and share-alike for these derived numbers)",
        },
        "citation": "Thrasher, B., Wang, W., Michaelis, A., Nemani, R. (2021). NEX-GDDP-CMIP6. NASA Center for Climate "
                    "Simulation. https://doi.org/10.7917/OFSG3345",
        "acknowledgement": "Climate scenarios used were from the NEX-GDDP-CMIP6 dataset, prepared by the Climate Analytics "
                           "Group and NASA Ames Research Center using the NASA Earth Exchange and distributed by the "
                           "NASA Center for Climate Simulation (NCCS).",
        "access": "NCCS THREDDS NetcdfSubset point requests, https://ds.nccs.nasa.gov/thredds/catalog/AMES/NEX/GDDP-CMIP6/",
        "generated": time.strftime("%Y-%m-%d"),
        "generator": "tools/nex_gddp_point.py",
    }
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(doc, fh, indent=1)
    print(json.dumps(doc["ensemble"], indent=1))


if __name__ == "__main__":
    if len(sys.argv) != 6:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4]), sys.argv[5])
