//! Cited open sources behind the generic guidance (soil, residues, Trichoderma, biological control, rotation).
use crate::model::Citation;

fn c(title: &str, author: &str, year: Option<i32>, url: &str) -> Citation {
    Citation { title: title.into(), author: author.into(), year, url: url.into() }
}

/// Cover crops and green manures: soil organic matter, soil life, legume N to the next crop.
pub fn sare_cover_crops() -> Citation {
    c("Building Soils for Better Crops, Ch. 10: Cover Crops", "Magdoff, F., van Es, H. / SARE Outreach", Some(2021),
      "https://www.sare.org/publications/building-soils-for-better-crops/cover-crops")
}

/// Crop rotation effects on soil fertility, nutrition and pest/disease cycles.
pub fn sare_rotation() -> Citation {
    c("Crop Rotation on Organic Farms: Crop Rotation Effects on Soil Fertility and Plant Nutrition",
      "Rangarajan, A., in Mohler, C.L., Johnson, S.E. (eds.) / SARE", Some(2009),
      "https://www.sare.org/publications/crop-rotation-on-organic-farms/physical-and-biological-processes-in-crop-production/crop-rotation-effects-on-soil-fertility-and-plant-nutrition/")
}

/// Incorporate residues promptly after harvest: intact residue keeps disease organisms alive.
pub fn residue_incorporation() -> Citation {
    c("End of Season Practices to Reduce Vegetable Diseases Next Year", "Ernest, E. / University of Delaware Cooperative Extension",
      Some(2023), "https://sites.udel.edu/weeklycropupdate/?p=23433")
}

/// Trichoderma as biocontrol of root/soil-borne diseases and growth promoter (review).
pub fn trichoderma_review() -> Citation {
    c("Trichoderma Species: Our Best Fungal Allies in the Biocontrol of Plant Diseases—A Review. Plants 12(3):432",
      "Plants (MDPI); author list at the DOI", Some(2023), "https://doi.org/10.3390/plants12030432")
}

/// SENASA biological control programme in Peru (Trichoderma, Beauveria, Chrysoperla, parasitoids).
pub fn senasa_biocontrol() -> Citation {
    c("Memoria Anual 2016, Subdirección de Control Biológico", "SENASA, Dirección de Sanidad Vegetal", Some(2016),
      "https://repositorio.senasa.gob.pe/bitstreams/294c749d-a012-4828-924c-e888585c3592/download")
}

/// Pest development speeds up with temperature (temperature-dependent phenology model, CIP).
pub fn temperature_pest_development() -> Citation {
    c("A temperature-dependent phenology model for Liriomyza huidobrensis (Diptera: Agromyzidae). J. Econ. Entomol. 110(3):1333-1344",
      "Mujica, N., Sporleder, M., Carhuapoma, P., Kroschel, J.", Some(2017), "https://pmc.ncbi.nlm.nih.gov/articles/PMC5444682/")
}

/// Legume N fixation ranges by crop.
pub fn nmsu_legume_n() -> Citation {
    c("Nitrogen Fixation by Legumes, Guide A-129", "Flynn, R., Idowu, J. / New Mexico State University", Some(2015),
      "https://pubs.nmsu.edu/_a/A129/index.html")
}

/// The agronomist interview that defines the guidance themes (local project document, not a published source).
pub const INTERVIEW: &str = "Field Shift interview with agronomist Ing. Cenia (2026)";

#[cfg(test)]
pub fn all() -> Vec<Citation> {
    vec![sare_cover_crops(), sare_rotation(), residue_incorporation(), trichoderma_review(), senasa_biocontrol(),
         temperature_pest_development(), nmsu_legume_n()]
}
