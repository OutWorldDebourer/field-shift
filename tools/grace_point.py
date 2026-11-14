"""Extract a GRACE/GRACE-FO mascon time series at one point and write it as JSON.

Usage (from the project root, with the tools venv):
  tools/.venv/Scripts/python.exe -I tools/grace_point.py <grace.nc> <lat> <lon> <out.json>

Input: JPL GRACE/GRACE-FO mascon CRI v4 netCDF (TELLUS_GRAC-GRFO_MASCON_CRI_GRID_RL06.3_V4),
downloaded with the owner's Earthdata token. Values are liquid water equivalent thickness (cm),
relative to the product's baseline. Regional context only: mascons are about 300 km wide.
"""
import json
import sys
from datetime import date, timedelta

import h5py
import numpy as np


def main(path: str, lat: float, lon: float, out: str) -> None:
    with h5py.File(path, "r") as f:
        lats = f["lat"][:]
        lons = f["lon"][:]
        days = f["time"][:]
        units = f["time"].attrs.get("units", b"days since 2002-01-01T00:00:00Z")
        units = units.decode() if isinstance(units, bytes) else str(units)
        lon360 = lon % 360.0
        i = int(np.abs(lats - lat).argmin())
        j = int(np.abs(lons - lon360).argmin())
        lwe = f["lwe_thickness"][:, i, j]
        fill = f["lwe_thickness"].attrs.get("_FillValue")

    base = date.fromisoformat(units.split("since")[1].strip()[:10])
    series = []
    for d, v in zip(days, lwe):
        value = None if (fill is not None and v == fill) or not np.isfinite(v) else round(float(v), 3)
        series.append({"date": (base + timedelta(days=float(d))).isoformat(), "lwe_cm": value})

    doc = {
        "source": "NASA JPL GRACE/GRACE-FO Mascon CRI RL06.3 v4 (PO.DAAC TELLUS_GRAC-GRFO_MASCON_CRI_GRID_RL06.3_V4)",
        "requested": {"lat": lat, "lon": lon},
        "cell_center": {"lat": float(lats[i]), "lon": float(lons[j])},
        "units": "cm liquid water equivalent",
        "note": "Regional context only (mascons about 300 km).",
        "series": series,
    }
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(doc, fh, indent=1)
    valid = [s for s in series if s["lwe_cm"] is not None]
    print(f"cell {doc['cell_center']}, {len(valid)} months, {valid[0]['date']} .. {valid[-1]['date']}")
    print("first", valid[0], "last", valid[-1])


if __name__ == "__main__":
    if len(sys.argv) != 5:
        sys.exit(__doc__)
    main(sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), sys.argv[4])
