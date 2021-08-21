#![allow(dead_code)]

use mkt_item_coverage::screenshot::*;
use mkt_item_coverage::*;

fn main() {
    println!("MKT Item Coverage");
    // test_update_data();
    // test_overlay();
    test_screenshot_import();
}

fn test_update_data() {
    let mut data = MktDatabase::new();
    update_mkt_item_data(&mut data, ItemType::Driver);
    // update_mkt_item_data(&mut data, ItemType::Kart);
    // update_mkt_item_data(&mut data, ItemType::Glider);
    update_mkt_item_coverage_data(&mut data);
    println!("{:#?}", data);
}

fn test_screenshot_import() {
    let data = get_database();

    let screenshot = image::open("tests/mkt_drivers.jpg").unwrap().into_rgb8();
    // let screenshot2 = image::open("tests/mkt_drivers2.jpg").unwrap().into_rgb8();
    // let screenshot3 = image::open("tests/mkt_drivers3.jpg").unwrap().into_rgb8();
    // let screenshot4 = image::open("tests/mkt_karts.jpg").unwrap().into_rgb8();

    let (inventory, missing) = screenshots_to_inventory(vec![screenshot], &data);

    dbg!(&inventory);
    dbg!(&inventory.drivers.len());
    dbg!(&inventory.karts.len());
    dbg!(&inventory.gliders.len());

    // dbg!(&missing);
    dbg!(&missing.len());
}
