#![allow(dead_code)]

use mkt_coverage::screenshot::*;
use mkt_coverage::*;

fn main() {
    println!("MKT Coverage");

    let data = get_database();

    let screenshot = image::open("tests/mkt_drivers.jpg").unwrap().into_rgb8();
    let screenshot2 = image::open("tests/mkt_drivers2.jpg").unwrap().into_rgb8();
    let screenshot3 = image::open("tests/mkt_drivers3.jpg").unwrap().into_rgb8();
    let screenshot4 = image::open("tests/mkt_karts.jpg").unwrap().into_rgb8();

    let (inventory, missing) = screenshots_to_inventory(
        vec![screenshot, screenshot2, screenshot3, screenshot4],
        &data,
    );

    dbg!(&inventory);
    dbg!(&inventory.drivers.len());
    dbg!(&inventory.karts.len());
    dbg!(&inventory.gliders.len());

    // dbg!(&missing);
    dbg!(&missing.len());
}
