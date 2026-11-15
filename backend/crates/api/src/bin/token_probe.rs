//! Development tool: `cargo run --release -p api --bin token_probe [en|es]` creates a fresh Huaral analysis (pea
//! now, lettuce before), replays one scripted consultant turn and prints the input tokens of every tool-loop request,
//! counted with the free `count_tokens` endpoint. No paid model call is made.
use common::{config::Config, AppState};
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("warn").init();
    let lang = std::env::args().nth(1).unwrap_or_else(|| "en".into());
    let config = Config::load()?;
    let db = MySqlPoolOptions::new().max_connections(4).connect(config.database_url()).await?;
    let state = AppState::new(db, config);
    let site = analyses::examples::get("huaral").ok_or_else(|| anyhow::anyhow!("no huaral example"))?;
    let n: analyses::NewAnalysis = serde_json::from_value(serde_json::json!({
        "field": site.field, "location_mode": "example", "example_site": "huaral", "lang": lang, "label": "token probe" }))?;
    let id = analyses::store::create(&state.db, &n, None).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    analyses::store::add_greeting(&state.db, &id, &lang).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let text = if lang == "es" { "Arveja ahora. Lechuga antes. Riego por surcos." } else { "Peas now. Lettuce before. Furrow irrigation." };
    let (rounds, sizes) = consultant::probe::run(&state, &id, text, &lang).await?;
    println!("analysis {id} ({lang})");
    for s in &sizes { println!("tool {:<14} {:>7} chars", s.name, s.chars); }
    let mut total = 0;
    for r in &rounds { total += r.input_tokens; println!("{:<24} {:>7} input tokens ({} bytes)", r.label, r.input_tokens, r.bytes); }
    println!("turn total               {total:>7} input tokens over {} requests", rounds.len());
    // Billing-equivalent input tokens (Opus 5.5: cache write 1.25x, cache read 0.05x of the input price).
    let p = consultant::probe::prefix_tokens(&state).await?;
    let n: Vec<f64> = rounds.iter().map(|r| r.input_tokens as f64).collect();
    let system_only: f64 = n.iter().map(|x| 0.05 * p as f64 + (x - p as f64)).sum::<f64>() + 0.25 * p as f64;
    let mut tail = 0.0; let mut prev = p as f64;
    for x in &n { tail += 0.05 * prev + 1.25 * (x - prev); prev = *x; }
    println!("cached prefix (tools + system)   {p} tokens");
    println!("billing-equivalent input, system-only cache: {system_only:.0}; with tail cache: {tail:.0}");
    Ok(())
}
