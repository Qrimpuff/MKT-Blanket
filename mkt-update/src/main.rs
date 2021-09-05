use mkt_data::*;
use mkt_update::*;

fn main() {
    println!("MKT Database Update");

    let mut data = MktDatabase::load("tests/database.json").unwrap_or_else(|_| MktDatabase::new());
    let update = test_update_data();
    data.merge(update);
    data.save("tests/database.json").unwrap();

    println!("Done");
}

fn test_update_data() -> MktDatabase {
    let mut data = MktDatabase::new();
    update_mkt_item_data(&mut data, ItemType::Driver);
    update_mkt_item_data(&mut data, ItemType::Kart);
    update_mkt_item_data(&mut data, ItemType::Glider);
    update_mkt_item_coverage_data(&mut data);
    data.load_hashes().unwrap();
    // println!("{:?}", data);
    data
}
