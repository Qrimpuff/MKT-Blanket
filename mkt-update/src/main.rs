use mkt_data::*;
use mkt_update::*;

fn main() {
    println!("MKT Database Update");

    let mut data = MktDatabase::load("tests/database.json").unwrap_or_else(|_| MktDatabase::new());
    let mut update = test_update_data();
    update.copy_hashes(data);
    data = update;
    // println!("{:#?}", data);
    data.load_hashes().unwrap();
    data.save("tests/database.json").unwrap();

    println!("Done");
}

fn test_update_data() -> MktDatabase {
    let mut data = MktDatabase::new();
    update_mkt_item_data(&mut data, ItemType::Driver);
    // update_mkt_item_data(&mut data, ItemType::Kart);
    // update_mkt_item_data(&mut data, ItemType::Glider);
    update_mkt_item_coverage_data(&mut data);
    data
}
