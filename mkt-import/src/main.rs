use mkt_data::*;
use mkt_import::screenshot::*;

fn main() {
    println!("MKT Test inventory Import");

    let data = MktData::load("data/mkt_data.json").unwrap_or_else(|_| MktData::new());
    test_screenshot_import(&data);

    println!("Done");
}

fn test_screenshot_import(data: &MktData) {
    // let data = get_data_hashes();

    let screenshot = image::open("tests/mkt_drivers6.jpg").unwrap().into_rgb8();
    // let screenshot = image::open("tests/mkt_drivers.jpg").unwrap().into_rgb8();
    // let screenshot2 = image::open("tests/mkt_drivers2.jpg").unwrap().into_rgb8();
    // let screenshot3 = image::open("tests/mkt_drivers3.jpg").unwrap().into_rgb8();
    // let screenshot4 = image::open("tests/mkt_karts.jpg").unwrap().into_rgb8();
    let list = vec![screenshot];

    // let reddit1 = image::open("tests/reddit1.png").unwrap().into_rgb8();
    // let reddit2 = image::open("tests/reddit2.png").unwrap().into_rgb8();
    // let reddit3 = image::open("tests/reddit3.png").unwrap().into_rgb8();
    // let reddit4 = image::open("tests/reddit4.png").unwrap().into_rgb8();
    // let reddit5 = image::open("tests/reddit5.png").unwrap().into_rgb8();
    // let reddit6 = image::open("tests/reddit6.png").unwrap().into_rgb8();
    // let list = vec![reddit1, reddit2, reddit3, reddit4, reddit5, reddit6];
    // let list = vec![reddit1];

    // let list = vec![screenshot];

    let (inventory, new_hashes) = screenshots_to_inventory(list, data, None);

    dbg!(&inventory);
    dbg!(&inventory.drivers.len());
    dbg!(&inventory.karts.len());
    dbg!(&inventory.gliders.len());

    dbg!(&new_hashes);
    dbg!(&new_hashes.hashes.len());
}
