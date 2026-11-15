//! Perennial crops keep their main crop: inter-row cover and green manures, intercropping while young,
//! residue management and biological control.
use super::*;
use crate::seed;

pub fn practices(c: &CropRecord, s: &FieldSignals) -> Vec<Practice> {
    let deficit = s.water_balance_mm.is_some_and(|w| w < 0.0);
    let mut cover_evidence = vec![guidance(
        "Cover crops between rows add organic matter, sustain mycorrhizal fungi and soil life; legumes in the mix add nitrogen.",
        "Las coberturas entre hileras aportan materia orgánica y mantienen hongos micorrícicos y vida del suelo; las leguminosas de la mezcla aportan nitrógeno.",
        refs::sare_cover_crops())];
    if s.bare_soil == Some(true) {
        cover_evidence.push(signal_field("Satellite NDVI shows bare-soil periods on this field".into(), "El NDVI satelital muestra periodos de suelo desnudo en este campo".into()));
    }
    if let Some(w) = s.water_balance_mm { cover_evidence.push(signal(format!("Water balance: {w:.0} mm"), format!("Balance hídrico: {w:.0} mm"))); }
    if let crate::model::RotationNotes::Note { en, es, citation } = &c.traits.rotation_notes {
        cover_evidence.push(Evidence { kind: EvidenceKind::Trait, text: L10n::new(en.clone(), es.clone()), citation: Some(citation.clone()) });
    }

    let mut inter_evidence = vec![];
    for k in ["common_bean", "pea"] {
        if let Some(l) = seed::get(k) {
            inter_evidence.extend(trait_evidence(l, "N fixation", "fijación de N", &l.traits.n_fixation.typical_range));
        }
    }
    inter_evidence.push(guidance("Legumes leave nitrogen in the soil for the following plants.", "Las leguminosas dejan nitrógeno en el suelo para las plantas siguientes.", refs::sare_rotation()));

    let mut bio_actions = vec![];
    let mut bio_evidence = vec![];
    for b in &c.traits.beneficials {
        bio_actions.push(L10n::new(format!("{} ({}) against: {}", b.name.en, b.kind, b.controls.en),
                                   format!("{} ({}) contra: {}", b.name.es, b.kind, b.controls.es)));
        bio_evidence.push(Evidence {
            kind: EvidenceKind::Trait,
            text: L10n::new(format!("{} controls {}", b.name.en, b.controls.en), format!("{} controla {}", b.name.es, b.controls.es)),
            citation: Some(b.citation.clone()),
        });
    }
    bio_evidence.push(guidance("SENASA produces and releases biological control agents in Peru.", "SENASA produce y libera controladores biológicos en el Perú.", refs::senasa_biocontrol()));
    let mut bio_gaps = vec![L10n::new("Release rates and timing must come from SENASA or your agronomist.", "Las dosis y momentos de liberación deben venir de SENASA o de su agrónomo.")];
    if c.traits.beneficials.is_empty() {
        bio_gaps.push(L10n::new("No cited biological control agents for this crop yet.", "Aún no hay controladores biológicos citados para este cultivo."));
    }

    vec![
        Practice {
            id: "inter-row-cover".into(),
            title: L10n::new("Inter-row cover crops and green manures", "Coberturas y abonos verdes entre hileras"),
            why: L10n::new(format!("{} stays in place for years; the space between rows is where soil can be rebuilt without touching the main crop.", c.name_en()),
                           format!("{} se queda por años; el espacio entre hileras es donde se puede recuperar el suelo sin tocar el cultivo principal.", c.name_es())),
            actions: vec![
                L10n::new("Sow a legume–cereal mix between rows; mow it and leave it as mulch or incorporate it at flowering.",
                          "Siembre una mezcla de leguminosa y cereal entre hileras; córtela y déjela como cobertura o incorpórela en floración."),
                L10n::new("Keep the cover away from the trunk/crown zone to avoid competing for water there.",
                          "Mantenga la cobertura lejos de la zona del tronco o corona para no competir por agua allí."),
            ],
            evidence: cover_evidence,
            gaps: vec![L10n::new("No soil analysis yet: the organic-matter gain cannot be measured.", "Aún no hay análisis de suelo: no se puede medir la ganancia de materia orgánica.")],
            risks: vec![if deficit {
                L10n::new("Water deficit this season: a cover uses irrigation water; prefer mulch or a short cover.", "Déficit hídrico esta campaña: la cobertura usa agua de riego; prefiera mulch o una cobertura corta.")
            } else {
                L10n::new("A cover uses some irrigation water; mow it before it competes.", "La cobertura usa algo de agua de riego; córtela antes de que compita.")
            }],
        },
        Practice {
            id: "intercropping".into(),
            title: L10n::new("Intercropping while the plantation is young", "Cultivos asociados mientras la plantación es joven"),
            why: L10n::new(format!("Short legumes between young {} rows give income and nitrogen; the main crop is never replaced.", c.name_en()),
                           format!("Leguminosas de ciclo corto entre hileras jóvenes de {} dan ingreso y nitrógeno; el cultivo principal nunca se reemplaza.", c.name_es())),
            actions: vec![L10n::new("Grow bean or pea between rows only while the canopy leaves light; stop when it closes.",
                                    "Siembre frijol o arveja entre hileras solo mientras la copa deje pasar luz; deje de hacerlo cuando se cierre.")],
            evidence: inter_evidence,
            gaps: vec![L10n::new("Spacing and canopy cover of this field are unknown.", "Se desconoce el distanciamiento y la cobertura de copa de este campo.")],
            risks: vec![L10n::new("Intercrops can host shared pests; choose a different family.", "Los cultivos asociados pueden hospedar plagas comunes; elija otra familia.")],
        },
        Practice {
            id: "residue-management".into(),
            title: L10n::new("Residue management", "Manejo de residuos"),
            why: L10n::new("Prunings, fallen fruit and crop residues left whole or piled at the field edge keep pests and diseases alive.",
                           "Las podas, frutos caídos y residuos dejados enteros o amontonados en el borde del campo mantienen vivas plagas y enfermedades."),
            actions: vec![
                L10n::new("Chop prunings and compost them (ideally with Trichoderma) or use them as mulch; collect fallen fruit.",
                          "Pique las podas y compóstelas (de preferencia con Trichoderma) o úselas como mulch; recoja los frutos caídos."),
                L10n::new("Avoid burning: it pollutes the air; avoid piles at the field edge: they become a source of infestation.",
                          "Evite quemar: contamina el aire; evite montones en el borde del campo: se vuelven foco de infestación."),
            ],
            evidence: vec![
                guidance("Intact residue lets disease organisms survive; decomposition by soil microbes reduces them.",
                         "Los residuos enteros permiten que sobrevivan los patógenos; la descomposición por microbios del suelo los reduce.", refs::residue_incorporation()),
                guidance("Trichoderma in compost protects roots against soil-borne diseases.", "Trichoderma en el compost protege las raíces contra enfermedades del suelo.", refs::trichoderma_review()),
            ],
            gaps: vec![L10n::new("Local rules on residue burning and fruit-fly sanitation should be checked with SENASA.",
                                 "Verifica con SENASA las normas locales sobre quema de residuos y saneamiento contra mosca de la fruta.")],
            risks: vec![L10n::new("Fresh residue incorporated near harvest can tie up nitrogen for a few weeks.",
                                  "Los residuos frescos incorporados cerca de la cosecha pueden inmovilizar nitrógeno unas semanas.")],
        },
        Practice {
            id: "biological-control".into(),
            title: L10n::new("Biological control", "Control biológico"),
            why: L10n::new("Beneficial insects and fungi lower pest populations and the number of pesticide sprays.",
                           "Los insectos y hongos benéficos reducen las poblaciones de plagas y el número de aplicaciones de plaguicidas."),
            actions: if bio_actions.is_empty() {
                vec![L10n::new("Ask SENASA which agents are available for your main pest.", "Consulte a SENASA qué controladores hay para su plaga principal.")]
            } else { bio_actions },
            evidence: bio_evidence,
            gaps: bio_gaps,
            risks: vec![L10n::new("Broad-spectrum insecticides kill the beneficials; coordinate sprays with releases.",
                                  "Los insecticidas de amplio espectro matan a los benéficos; coordine aplicaciones con liberaciones.")],
        },
    ]
}
