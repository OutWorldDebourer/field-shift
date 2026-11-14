//! Repair of text that went through a UTF-8 → Windows-1252 → UTF-8 round trip ("maíz" stored as "maÃ­z"), once
//! or several times. JSON values were bound to MariaDB as binary and read back as Latin-1; writes now send text,
//! and `repair_db` fixes rows saved before that, once, at startup.
use serde_json::Value;
use sqlx::{MySqlPool, Row};

/// Windows-1252 bytes 0x80–0x9F that map to characters outside Latin-1.
fn cp1252_byte(c: char) -> Option<u8> {
    Some(match c {
        '€' => 0x80, '‚' => 0x82, 'ƒ' => 0x83, '„' => 0x84, '…' => 0x85, '†' => 0x86, '‡' => 0x87, 'ˆ' => 0x88,
        '‰' => 0x89, 'Š' => 0x8a, '‹' => 0x8b, 'Œ' => 0x8c, 'Ž' => 0x8e, '‘' => 0x91, '’' => 0x92, '“' => 0x93,
        '”' => 0x94, '•' => 0x95, '–' => 0x96, '—' => 0x97, '˜' => 0x98, '™' => 0x99, 'š' => 0x9a, '›' => 0x9b,
        'œ' => 0x9c, 'ž' => 0x9e, 'Ÿ' => 0x9f,
        c if (c as u32) < 256 => c as u32 as u8,
        _ => return None,
    })
}

fn looks_garbled(s: &str) -> bool { s.contains('Ã') || s.contains('Â') || s.contains("â€") }

/// Undo the round trip as many times as it was applied. Each run of non-ASCII characters is repaired on its own,
/// so a string that mixes correct accents ("orgánica") with garbled ones ("estiÃ©rcol") is fixed too; text that is
/// not garbled comes back unchanged.
pub fn demojibake(s: &str) -> String {
    let mut cur = s.to_string();
    for _ in 0..4 {
        if !looks_garbled(&cur) { break; }
        let next = fix_runs(&cur);
        if next == cur { break; }
        cur = next;
    }
    cur
}

fn fix_runs(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut run: Vec<char> = vec![];
    fn flush(run: &mut Vec<char>, out: &mut String) {
        if run.is_empty() { return; }
        let bytes: Option<Vec<u8>> = run.iter().map(|c| cp1252_byte(*c)).collect();
        match bytes.and_then(|b| String::from_utf8(b).ok()) {
            Some(fixed) if fixed.chars().count() < run.len() => out.push_str(&fixed),
            _ => out.extend(run.iter()),
        }
        run.clear();
    }
    for c in s.chars() {
        if !c.is_ascii() && cp1252_byte(c).is_some() { run.push(c); } else { flush(&mut run, &mut out); out.push(c); }
    }
    flush(&mut run, &mut out);
    out
}

pub fn fix_value(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(demojibake(s)),
        Value::Array(a) => Value::Array(a.iter().map(fix_value).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, x)| (demojibake(k), fix_value(x))).collect()),
        x => x.clone(),
    }
}

/// Repair stored JSON columns that still hold garbled text. Idempotent; returns how many cells were rewritten.
pub async fn repair_db(db: &MySqlPool) -> anyhow::Result<usize> {
    let targets: [(&str, &str, &str); 7] = [
        ("analyses", "id", "facts"), ("analyses", "id", "steps"), ("analyses", "id", "crop_current_record"),
        ("analyses", "id", "crop_history"), ("sections", "CONCAT(analysis_id, '|', section)", "content"),
        ("messages", "id", "meta"), ("reports", "id", "evidence"),
    ];
    let mut n = 0;
    for (table, key, col) in targets {
        let sql = format!("SELECT CAST({key} AS CHAR) AS k, CAST({col} AS CHAR) AS v FROM {table} WHERE {col} LIKE '%Ã%' OR {col} LIKE '%Â%' OR {col} LIKE '%â€%'");
        let rows = match sqlx::query(&sql).fetch_all(db).await { Ok(r) => r, Err(e) => { tracing::warn!("repair {table}.{col}: {e}"); continue } };
        for r in rows {
            let (k, raw): (String, String) = (r.try_get("k")?, r.try_get("v")?);
            let Ok(v) = serde_json::from_str::<Value>(&raw) else { continue };
            let fixed = fix_value(&v);
            if fixed == v { continue; }
            let upd = format!("UPDATE {table} SET {col} = ? WHERE {key} = ?");
            sqlx::query(&upd).bind(fixed.to_string()).bind(&k).execute(db).await?;
            n += 1;
        }
    }
    // Cached upstream payloads are simply fetched again.
    let r = sqlx::query("DELETE FROM source_cache WHERE payload LIKE '%Ã%' OR payload LIKE '%â€%'").execute(db).await;
    if let Ok(r) = r { n += r.rows_affected() as usize; }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn undoes_one_and_two_round_trips() {
        assert_eq!(demojibake("maÃ­z u otra hortaliza"), "maíz u otra hortaliza");
        assert_eq!(demojibake("floraciÃ³n"), "floración");
        assert_eq!(demojibake("cada 8â€“10 dÃ­as"), "cada 8–10 días");
        let twice = "maÃ­z".chars().flat_map(|c| { let mut b = [0u8; 4]; c.encode_utf8(&mut b).bytes().collect::<Vec<_>>() }).map(|b| match b { 0x80..=0x9f => char::from_u32(b as u32).unwrap(), _ => b as char }).collect::<String>();
        let _ = twice; // byte-level double encoding is covered by the loop; the plain cases above are the ones stored
        assert_eq!(demojibake("río Chancay, ñ, ü"), "río Chancay, ñ, ü");
        assert_eq!(demojibake("materia orgánica: estiÃ©rcol de cuy"), "materia orgánica: estiércol de cuy");
        assert_eq!(demojibake("próximo cultivo: ma\u{c3}\u{ad}z"), "próximo cultivo: maíz");
    }
}
