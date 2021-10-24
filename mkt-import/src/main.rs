use std::fs;

use itertools::Itertools;
use mkt_data::*;
use mkt_import::screenshot::*;

fn main() {
    println!("MKT Test inventory Import");

    let data = MktData::load("data/mkt_data.json").unwrap_or_else(|_| MktData::new());
    // _test_screenshot_import(&data);

    // _test_combine();

    _test_screenshots_to_bootstrap_hashes(&data);

    // _test_img_hash();

    println!("Done");
}

fn _test_screenshot_import(data: &MktData) {
    let name = "Screenshot_20211022-180319_Mario Kart";

    let screenshot = image::open(format!("tmp/{}.jpg", name))
        .unwrap()
        .into_rgb8();
    let list = vec![screenshot];

    let (mut inventory, new_hashes) = screenshots_to_inventory(list, data, None);

    println!("{}", inventory.to_json().unwrap());
    dbg!(&inventory.drivers.len());
    dbg!(&inventory.karts.len());
    dbg!(&inventory.gliders.len());

    println!("{}", new_hashes.to_json().unwrap());
    dbg!(&new_hashes.hashes.len());

    inventory.clear_dates();
    fs::write(format!("tmp/{}.json", name), inventory.to_json().unwrap()).unwrap();
}

fn _test_combine() {
    let list = (1..=6)
        .map(|i| {
            image::open(format!("tests/mkt_drivers{}.jpg", i))
                .unwrap()
                .into_rgb8()
        })
        .collect();
    combine_screenshots(list).save("pics/big_out.png").unwrap();
}

fn _test_screenshots_to_bootstrap_hashes(data: &MktData) {
    // let list_d = (1..=6)
    //     .map(|i| {
    //         image::open(format!("tmp/mkt drivers ({}).jpg", i))
    //             .unwrap()
    //             .into_rgb8()
    //     })
    //     .collect_vec();
    // let list_k = (1..=8)
    //     .map(|i| {
    //         image::open(format!("tmp/mkt karts ({}).jpg", i))
    //             .unwrap()
    //             .into_rgb8()
    //     })
    //     .collect_vec();
    let list_g = (1..=6)
        .map(|i| {
            image::open(format!("tmp/mkt gliders ({}).jpg", i))
                .unwrap()
                .into_rgb8()
        })
        .collect_vec();
    let mut hashes = MktItemHashes::new();
    // hashes.merge(screenshots_to_bootstrap_hashes(list_d, ItemType::Driver, data).unwrap());
    // hashes.merge(screenshots_to_bootstrap_hashes(list_k, ItemType::Kart, data).unwrap());
    hashes.merge(screenshots_to_bootstrap_hashes(list_g, ItemType::Glider, data).unwrap());
    println!("{:?}", hashes);

    fs::write("data/mkt_hash.json", hashes.to_json().unwrap()).unwrap();
}
