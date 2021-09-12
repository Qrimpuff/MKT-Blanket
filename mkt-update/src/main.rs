use mkt_data::*;
use mkt_update::*;

fn main() {
    println!("MKT Data Update");

    let mut data = MktData::load("data/mkt_data.json").unwrap_or_else(|_| MktData::new());
    let update = test_update_data();
    data.merge(update);
    data.save("data/mkt_data.json").unwrap();

    println!("Done");
}

fn test_update_data() -> MktData {
    let mut data = MktData::new();
    update_mkt_item_data(&mut data, ItemType::Driver);
    update_mkt_item_data(&mut data, ItemType::Kart);
    update_mkt_item_data(&mut data, ItemType::Glider);
    update_mkt_item_coverage_data(&mut data);
    data.load_hashes().unwrap();
    // println!("{:?}", data);
    data
}
