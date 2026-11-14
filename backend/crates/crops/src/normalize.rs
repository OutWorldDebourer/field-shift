//! Free text (English or Spanish, any accents, plurals, diminutives, small typos) to a crop key. Suggestions, never a restriction.
use crate::model::CropRecord;
use serde::Serialize;

/// Lowercase, accents removed, only letters/digits and single spaces.
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.trim().chars().flat_map(char::to_lowercase) {
        let c = match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        };
        if c == ' ' && (out.is_empty() || out.ends_with(' ')) { continue; }
        out.push(c);
    }
    out.trim_end().to_string()
}

/// A loose stem per word: plural, then diminutive, then the final vowel removed ("arverjitas" → "arverj", "grapes" → "grap").
pub fn stem(folded: &str) -> String {
    folded.split(' ').map(stem_word).collect::<Vec<_>>().join(" ")
}

fn stem_word(w: &str) -> String {
    let mut w = w.to_string();
    if w.len() > 4 && w.ends_with("ies") {
        w.truncate(w.len() - 3);
        w.push('y');
    } else if w.len() > 3 && w.ends_with('s') && !w.ends_with("ss") {
        w.pop();
    }
    for suf in ["cita", "cito", "ita", "ito", "illa", "illo"] {
        if w.len() >= suf.len() + 3 && w.ends_with(suf) {
            w.truncate(w.len() - suf.len());
            break;
        }
    }
    if w.len() > 3 && w.ends_with(['a', 'e', 'i', 'o', 'u', 'y']) {
        w.pop();
    }
    w
}

/// Optimal string alignment distance (Levenshtein plus adjacent transpositions).
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() { row[0] = i; }
    for (j, cell) in d[0].iter_mut().enumerate() { *cell = j; }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[n][m]
}

/// Score of the closest name of a crop for a query: 100 exact, 90 same stem, 70 prefix (45 if shorter than 4 letters), 50-55 typo.
#[derive(Clone, Debug, Serialize)]
pub struct Match {
    pub key: String,
    pub matched: String,
    pub score: u8,
}

/// Minimum score for `resolve` to accept a match without research.
pub const RESOLVE_MIN: u8 = 50;

pub fn score(query: &str, name: &str) -> u8 {
    let (q, n) = (fold(query), fold(name));
    if q.is_empty() || n.is_empty() { return 0; }
    if q == n { return 100; }
    let (qs, ns) = (stem(&q), stem(&n));
    if qs == ns { return 90; }
    let qlen = q.chars().count();
    if qlen >= 2 && (n.starts_with(&q) || n.split(' ').any(|w| w.starts_with(&q))) {
        return if qlen >= 4 { 70 } else { 45 };
    }
    let len = qs.chars().count();
    if len >= 4 {
        let max = if len <= 6 { 1 } else { 2 };
        let d = edit_distance(&qs, &ns);
        if d <= max { return 60 - 5 * d as u8; }
    }
    0
}

/// Best match per crop, best first.
pub fn rank<'a>(query: &str, crops: impl IntoIterator<Item = &'a CropRecord>) -> Vec<Match> {
    let mut out: Vec<Match> = crops.into_iter().filter_map(|c| {
        c.all_names()
            .map(|n| (score(query, n), n))
            .max_by_key(|(s, _)| *s)
            .filter(|(s, _)| *s > 0)
            .map(|(s, n)| Match { key: c.key.clone(), matched: n.to_string(), score: s })
    }).collect();
    out.sort_by(|a, b| b.score.cmp(&a.score).then(a.key.cmp(&b.key)));
    out
}

/// Best match good enough for `resolve`.
pub fn best<'a>(query: &str, crops: impl IntoIterator<Item = &'a CropRecord>) -> Option<Match> {
    rank(query, crops).into_iter().next().filter(|m| m.score >= RESOLVE_MIN)
}

/// A storage key for a researched crop: folded, underscores ("Dragon fruit" → "dragon_fruit").
pub fn slug(s: &str) -> String {
    fold(s).replace(' ', "_").chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    fn key(q: &str) -> Option<String> { best(q, seed::all()).map(|m| m.key) }

    #[test]
    fn spanish_english_accents_plurals_diminutives() {
        for (q, k) in [
            ("Arverjita", "pea"), ("arveja", "pea"), ("peas", "pea"), ("ARVERJAS", "pea"), ("alverjitas", "pea"),
            ("uva", "grape"), ("Grapes", "grape"), ("uvas", "grape"),
            ("fresas", "strawberry"), ("frutilla", "strawberry"), ("strawberries", "strawberry"),
            ("piña", "pineapple"), ("pina", "pineapple"), ("Piñas", "pineapple"),
            ("camote", "sweet_potato"), ("palta", "avocado"), ("mandarinas", "mandarin"),
            ("maíz", "maize"), ("tomates", "tomato"), ("papas", "potato"), ("cebollas", "onion"),
            ("espárragos", "asparagus"), ("quinua", "quinoa"), ("algodón", "cotton"), ("Pisum sativum", "pea"),
        ] {
            assert_eq!(key(q).as_deref(), Some(k), "{q}");
        }
    }

    #[test]
    fn typos_are_forgiven() {
        assert_eq!(key("tomatoe").as_deref(), Some("tomato"));
        assert_eq!(key("avocaod").as_deref(), Some("avocado"));
        assert_eq!(key("lechuja").as_deref(), Some("lettuce"));
    }

    #[test]
    fn unknown_crops_do_not_match() {
        assert_eq!(key("pitahaya"), None);
        assert_eq!(key("dragon fruit"), None);
        assert_eq!(key("x"), None);
    }

    #[test]
    fn type_ahead_offers_prefixes() {
        let r = rank("pa", seed::all());
        assert!(r.iter().any(|m| m.key == "avocado"), "palta");
        assert!(r.iter().any(|m| m.key == "potato"), "papa");
    }

    #[test]
    fn folding() {
        assert_eq!(fold("  Piña  Golden! "), "pina golden");
        assert_eq!(stem("arverjitas"), "arverj");
        assert_eq!(slug("Dragon Fruit"), "dragon_fruit");
        assert_eq!(edit_distance("abc", "acb"), 1);
    }
}
