//! Pest table with published developmental thresholds and degree-days per generation (each cited), and the estimated
//! number of generations this season versus the same calendar window in the 1985-2024 normal.
use crate::{days::{slice, Day}, gdd, meta::Meta, normals, stats::{round, NormalStat}};
use chrono::NaiveDate;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Citation {
    pub authors: &'static str,
    pub year: u16,
    pub title: &'static str,
    pub published_in: &'static str,
    pub url: &'static str,
    pub doi: Option<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pest {
    pub key: &'static str,
    pub common_name: &'static str,
    pub scientific_name: &'static str,
    /// Lower developmental threshold, °C.
    pub lower_c: f64,
    /// Upper developmental threshold (horizontal cutoff), °C; `None` when the source gives none.
    pub upper_c: Option<f64>,
    /// Degree-days (°C·day above `lower_c`) for one generation as defined by `generation_basis`.
    pub dd_per_generation: f64,
    pub generation_basis: &'static str,
    pub host_in_study: &'static str,
    pub citation: Citation,
    /// Known limits of using this threshold here.
    pub gaps: &'static [&'static str],
}

/// The pest table. Every value is taken from the cited publication; nothing is estimated.
pub fn table() -> Vec<Pest> {
    vec![
        Pest {
            key: "myzus_persicae",
            common_name: "Green peach aphid",
            scientific_name: "Myzus persicae (Sulzer)",
            lower_c: 4.0,
            upper_c: Some(30.0),
            dd_per_generation: 129.8,
            generation_basis: "Nymph to nymph (one generation), potato",
            host_in_study: "potato",
            citation: Citation {
                authors: "Whalon, M.E.; Smilowitz, Z.",
                year: 1979,
                title: "Temperature-dependent model for predicting field populations of green peach aphid, Myzus persicae (Homoptera: Aphididae)",
                published_in: "The Canadian Entomologist 111: 1025-1032 (thresholds as tabulated by UC IPM, https://ipm.ucanr.edu/weather/phenology-models-description/green-peach-aphid/)",
                url: "https://doi.org/10.4039/Ent1111025-9",
                doi: Some("10.4039/Ent1111025-9"),
            },
            gaps: &[
                "The source model uses the double-sine method; this service uses single sine, which differs slightly on days crossing a threshold.",
                "Aphid numbers depend on alate immigration, natural enemies and host quality, not only degree-days.",
            ],
        },
        Pest {
            key: "bemisia_tabaci",
            common_name: "Sweetpotato / silverleaf whitefly",
            scientific_name: "Bemisia tabaci (Gennadius), biotype B (MEAM1)",
            lower_c: 11.53,
            upper_c: None,
            dd_per_generation: 307.0,
            generation_basis: "Egg to adult (pre-oviposition period not included)",
            host_in_study: "bean (Italian biotype B colony)",
            citation: Citation {
                authors: "Bosco, D.; Caciagli, P.",
                year: 1998,
                title: "Bionomics and ecology of Bemisia tabaci (Sternorrhyncha: Aleyrodidae) in Italy",
                published_in: "European Journal of Entomology 95(4): 519-527",
                url: "https://www.eje.cz/artkey/eje-199804-0005_Bionomics_and_ecology_of_Bemisia_tabaci_Sternorrhyncha_Aleyrodidae_in_Italy.php",
                doi: None,
            },
            gaps: &[
                "No upper developmental threshold given in the source; none applied, so generations may be overestimated during spells above ~32 °C.",
                "Egg-to-adult only: adding the pre-oviposition period would lower the generation count slightly.",
                "Thresholds vary by biotype and host plant; this value comes from biotype B on bean.",
            ],
        },
        Pest {
            key: "spodoptera_frugiperda",
            common_name: "Fall armyworm",
            scientific_name: "Spodoptera frugiperda (J.E. Smith)",
            lower_c: 12.57,
            upper_c: None,
            dd_per_generation: 391.61,
            generation_basis: "Egg to adult (pre-oviposition period not included)",
            host_in_study: "sweetcorn kernels (South African population)",
            citation: Citation {
                authors: "du Plessis, H.; Schlemmer, M.-L.; Van den Berg, J.",
                year: 2020,
                title: "The Effect of Temperature on the Development of Spodoptera frugiperda (Lepidoptera: Noctuidae)",
                published_in: "Insects 11(4): 228",
                url: "https://doi.org/10.3390/insects11040228",
                doi: Some("10.3390/insects11040228"),
            },
            gaps: &[
                "No upper developmental threshold applied (linear range in the study 18-30 °C); generations may be overestimated above ~32 °C.",
                "Egg-to-adult only: adding the pre-oviposition period would lower the generation count slightly.",
            ],
        },
    ]
}

#[derive(Clone, Debug, Serialize)]
pub struct PestGenerations {
    pub pest: Pest,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub degree_days: Option<f64>,
    pub generations: Option<f64>,
    /// Generations over the same calendar dates in 1985-2024.
    pub normal_generations: Option<NormalStat>,
    /// generations - normal mean.
    pub difference: Option<f64>,
    pub meta: Meta,
}

fn dd_sum(s: &[Day], p: &Pest) -> Option<f64> {
    let v: Vec<f64> = s.iter().filter_map(|d| Some(gdd::dd_sine(d.tmin?, d.tmax?, p.lower_c, p.upper_c))).collect();
    (!v.is_empty()).then(|| v.iter().sum())
}

/// Estimated generations for every pest in the table over `from..=to`, with the normal for the same dates.
pub fn generations(all: &[Day], from: NaiveDate, to: NaiveDate) -> Vec<PestGenerations> {
    table().into_iter().map(|p| {
        let dd = dd_sum(slice(all, from, to), &p);
        let gens = dd.map(|x| round(x / p.dd_per_generation, 2));
        let years = normals::per_year(all, from, to, |w| vec![dd_sum(w, &p).map(|x| x / p.dd_per_generation)]);
        let normal = normals::stat_of(&years, 0);
        let difference = match (gens, &normal) { (Some(g), Some(n)) => Some(round(g - n.mean, 2)), _ => None };
        let method = format!(
            "{}; lower {} °C, upper {}; {} °C·day per generation ({}); generations = degree-days / DD per generation; normal = same calendar dates in {}",
            gdd::SINE_METHOD, p.lower_c, p.upper_c.map(|u| format!("{u} °C")).unwrap_or_else(|| "none".into()),
            p.dd_per_generation, p.generation_basis, normals::WINDOW_LABEL);
        PestGenerations { from, to, degree_days: dd.map(|x| round(x, 1)), generations: gens, normal_generations: normal, difference,
                          meta: Meta::new(format!("{from}..{to}"), method), pest: p }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normals::tests::synthetic;

    #[test]
    fn table_is_cited() {
        for p in table() {
            assert!(!p.citation.authors.is_empty() && p.citation.year > 1900 && p.citation.url.starts_with("https://"));
            assert!(p.lower_c > 0.0 && p.dd_per_generation > 0.0);
        }
    }

    #[test]
    fn generations_on_synthetic() {
        let d = synthetic();
        let from = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
        let g = generations(&d, from, to);
        assert_eq!(g.len(), 3);
        for x in &g {
            assert!(x.generations.unwrap() > 0.0);
            assert!(x.difference.unwrap().abs() < 1e-6, "same climate every year");
        }
    }
}
