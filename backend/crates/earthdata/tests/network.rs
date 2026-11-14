//! Live reads for Huaral (lat -11.47, lon -77.21). Run with `cargo test -p earthdata -- --ignored --nocapture`.
//! Needs the `earthdata` token in env.txt; the token is never printed.
use earthdata::{ctx::Ctx, imerg, smap};

fn ctx() -> Ctx {
    let cfg = common::config::Config::load().expect("config");
    let token = cfg.secrets.earthdata.clone().expect("earthdata token in env.txt");
    let http = reqwest::Client::builder().user_agent("FieldShift/0.1 tests").timeout(std::time::Duration::from_secs(120)).build().unwrap();
    Ctx { http, token, db: None }
}

#[tokio::test]
#[ignore]
async fn smap_huaral() {
    let r = smap::report(&ctx(), -11.47, -77.21, 7).await.unwrap();
    println!("cell {:?}", r.cell);
    println!("latest {:?}", r.latest);
    for d in &r.daily { println!("daily {} {:?} {:?}", d.time, d.surface_m3m3, d.rootzone_m3m3); }
    println!("past {}: {:?}", r.same_date_past_years.len(), r.comparison);
    println!("gaps {:?}", r.gaps);
    let l = r.latest.expect("latest");
    assert!(l.surface_m3m3.unwrap() > 0.0 && l.surface_m3m3.unwrap() < 0.6);
    assert_eq!((r.cell.row, r.cell.col), (973, 1100));
}

#[tokio::test]
#[ignore]
async fn imerg_huaral() {
    let r = imerg::report(&ctx(), -11.47, -77.21, 30).await.unwrap();
    println!("cell {:?}", r.cell);
    println!("summary {:?}", r.summary);
    for d in r.daily.iter().rev().take(5) { println!("day {} {:?}", d.date, d.rain_mm); }
    println!("gaps {:?}", r.gaps);
    assert!(r.summary.days_with_data >= 25);
}
