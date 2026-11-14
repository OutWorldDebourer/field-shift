//! The verified seed knowledge base: one cited JSON file per crop in `data/`, embedded at build time.
use crate::model::CropRecord;
use std::sync::OnceLock;

const FILES: &[(&str, &str)] = &[
    ("pea", include_str!("../data/pea.json")),
    ("common_bean", include_str!("../data/common_bean.json")),
    ("alfalfa", include_str!("../data/alfalfa.json")),
    ("quinoa", include_str!("../data/quinoa.json")),
    ("maize", include_str!("../data/maize.json")),
    ("cotton", include_str!("../data/cotton.json")),
    ("lettuce", include_str!("../data/lettuce.json")),
    ("potato", include_str!("../data/potato.json")),
    ("sweet_potato", include_str!("../data/sweet_potato.json")),
    ("tomato", include_str!("../data/tomato.json")),
    ("onion", include_str!("../data/onion.json")),
    ("asparagus", include_str!("../data/asparagus.json")),
    ("mandarin", include_str!("../data/mandarin.json")),
    ("avocado", include_str!("../data/avocado.json")),
    ("grape", include_str!("../data/grape.json")),
    ("mango", include_str!("../data/mango.json")),
    ("strawberry", include_str!("../data/strawberry.json")),
    ("pineapple", include_str!("../data/pineapple.json")),
];

/// All seed records (parsed once; a malformed file is a build-time bug caught by the tests).
pub fn all() -> &'static [CropRecord] {
    static SEED: OnceLock<Vec<CropRecord>> = OnceLock::new();
    SEED.get_or_init(|| {
        FILES.iter().map(|(name, text)| {
            serde_json::from_str::<CropRecord>(text).unwrap_or_else(|e| panic!("seed file {name}.json: {e}"))
        }).collect()
    })
}

pub fn get(key: &str) -> Option<&'static CropRecord> {
    all().iter().find(|c| c.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Origin, RotationNotes, Trait};

    #[test]
    fn all_seed_files_parse_and_keys_match() {
        for ((name, _), rec) in FILES.iter().zip(all()) {
            assert_eq!(*name, rec.key);
            assert_eq!(rec.origin, Origin::Seed);
            assert!(!rec.names.en.is_empty() && !rec.names.es.is_empty(), "{name} needs en and es names");
        }
        assert!(all().len() >= 18);
    }

    #[test]
    fn every_seed_value_has_a_citation_url() {
        for rec in all() {
            for (path, t) in rec.trait_leaves() {
                match t {
                    Trait::Value { citation, unit, .. } => {
                        assert!(citation.url.starts_with("https://") || citation.url.starts_with("http://"),
                            "{}.{path}: citation without URL", rec.key);
                        assert!(!citation.title.trim().is_empty(), "{}.{path}: citation without title", rec.key);
                        assert!(!unit.contains('F') || unit.contains("N/"), "{}.{path}: non-metric unit {unit}", rec.key);
                    }
                    Trait::Gap { gap } => assert!(!gap.trim().is_empty(), "{}.{path}: empty gap reason", rec.key),
                }
            }
            for c in rec.citations() {
                assert!(c.url.starts_with("http"), "{}: citation '{}' without URL", rec.key, c.title);
            }
            if let RotationNotes::Note { en, es, .. } = &rec.traits.rotation_notes {
                assert!(!en.is_empty() && !es.is_empty());
            }
        }
    }

    #[test]
    fn crop_types_are_as_agreed() {
        use crate::model::CropType::*;
        for (k, t) in [("pea", Annual), ("grape", Perennial), ("mango", Perennial), ("avocado", Perennial),
                       ("mandarin", Perennial), ("pineapple", SemiPerennial), ("strawberry", SemiPerennial)] {
            assert_eq!(get(k).unwrap().crop_type, t, "{k}");
        }
        assert!(get("pea").unwrap().is_legume());
    }
}
