//! Development tool (paid: one web-search research conversation per crop):
//! `cargo run --release -p api --bin crop_extras pea lettuce …` researches the phase-D data of seed crops (the
//! weather that favours each pest, critical micronutrients, biostimulant categories), validates every citation
//! against the pages actually retrieved, prints it for review and writes `backend/crates/crops/data/extras/<key>.json`
//! as unreviewed ("AI-researched, verify"); the curated seed file stays untouched. Usage goes to `api_usage`.
use common::{config::Config, AppState};
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("warn").init();
    let keys: Vec<String> = std::env::args().skip(1).collect();
    anyhow::ensure!(!keys.is_empty(), "name the seed crops to research, e.g. pea lettuce");
    let config = Config::load()?;
    let key = config.secrets.anthropic.clone().ok_or_else(|| anyhow::anyhow!("no anthropic key"))?;
    let data_dir = config.root.join("backend").join("crates").join("crops").join("data");
    let db = MySqlPoolOptions::new().max_connections(2).connect(config.database_url()).await?;
    let state = AppState::new(db, config);
    for k in keys {
        let rec = crops::seed::get(&k).ok_or_else(|| anyhow::anyhow!("{k} is not a seed crop"))?;
        println!("== {k}: researching…");
        let x = match crops::research_extras::research(&state.http, &key, rec, Some(&state.db)).await {
            Ok(x) => x,
            Err(e) => { println!("   failed: {e}"); continue; }
        };
        for (sci, c) in &x.conditions {
            println!("   pest {sci}: {} [{}–{} °C, {}] <{}>", c.favourable.en, c.temp_min_c.map(|v| v.to_string()).unwrap_or("-".into()), c.temp_max_c.map(|v| v.to_string()).unwrap_or("-".into()), c.humidity, c.citation.url);
        }
        for m in &x.micronutrients { println!("   micronutrient {}: {} | {} <{}>", m.element, m.why.en, m.deficiency.en, m.citation.url); }
        for b in &x.biostimulants { println!("   biostimulant {}: {} <{}>", b.category, b.evidence.en, b.citation.url); }
        for g in &x.gaps { println!("   gap: {g}"); }
        let dir = data_dir.join("extras");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{k}.json"));
        std::fs::write(&path, serde_json::to_string_pretty(&x)? + "\n")?;
        println!("   written to {} (add it to EXTRAS in crops/src/seed.rs)", path.display());
    }
    Ok(())
}
