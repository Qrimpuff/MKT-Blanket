#![allow(dead_code)]

use mkt_item_coverage::screenshot::*;
use mkt_item_coverage::*;

static TEST_UPDATE: bool = false;

fn main() {
    println!("MKT Item Coverage");

    let mut data = MktDatabase::load("tests/database.json").unwrap_or(MktDatabase::new());
    if TEST_UPDATE {
        let mut update = test_update_data();
        update.copy_hashes(data);
        data = update;
    }
    // println!("{:#?}", data);
    data.save("tests/database.json").unwrap();

    // test_overlay();
    test_screenshot_import(&data);
    // test_img_hash();
}

fn test_update_data() -> MktDatabase {
    let mut data = MktDatabase::new();
    update_mkt_item_data(&mut data, ItemType::Driver);
    // update_mkt_item_data(&mut data, ItemType::Kart);
    // update_mkt_item_data(&mut data, ItemType::Glider);
    update_mkt_item_coverage_data( &mut data);
    data
}

fn test_screenshot_import(data: &MktDatabase) {
    // let data = get_database_hashes();

    // let screenshot = image::open("tests/MKT_character_screen.png").unwrap().into_rgb8();
    // let screenshot = image::open("tests/mkt_drivers.jpg").unwrap().into_rgb8();
    // let screenshot2 = image::open("tests/mkt_drivers2.jpg").unwrap().into_rgb8();
    let screenshot3 = image::open("tests/mkt_drivers3.jpg").unwrap().into_rgb8();
    // let screenshot4 = image::open("tests/mkt_karts.jpg").unwrap().into_rgb8();
    // let list = vec![screenshot];

    // let reddit1 = image::open("tests/reddit1.png").unwrap().into_rgb8();
    // let reddit2 = image::open("tests/reddit2.png").unwrap().into_rgb8();
    // let reddit3 = image::open("tests/reddit3.png").unwrap().into_rgb8();
    let reddit4 = image::open("tests/reddit4.png").unwrap().into_rgb8();
    // let reddit5 = image::open("tests/reddit5.png").unwrap().into_rgb8();
    // let reddit6 = image::open("tests/reddit6.png").unwrap().into_rgb8();
    // let list = vec![reddit1, reddit2, reddit3, reddit4, reddit5, reddit6];
    // let list = vec![reddit1];

    let list = vec![screenshot3, reddit4];

    let (inventory, missing) = screenshots_to_inventory(list, &data);

    dbg!(&inventory);
    dbg!(&inventory.drivers.len());
    dbg!(&inventory.karts.len());
    dbg!(&inventory.gliders.len());

    // dbg!(&missing);
    dbg!(&missing.len());
}
