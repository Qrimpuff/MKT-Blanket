use mkt_data::*;
use mkt_update::*;

fn main() {
    println!("MKT Data Update");

    let mut data = test_update_data();

    // don't overwrite with bad data
    if !data.courses.is_empty()
        && !data.drivers.is_empty()
        && !data.karts.is_empty()
        && !data.gliders.is_empty()
    {
        let hash =
            MktItemHashes::load("data/mkt_hash.json").unwrap_or_else(|_| MktItemHashes::new());
        dbg!(&hash);
        data.merge_hashes(&hash);

        data.save("data/mkt_data.json").unwrap();
    }

    println!("Done");
}

fn test_update_data() -> MktData {
    let mut data = MktData::new();
    update_mkt_item_data(&mut data, ItemType::Driver);
    update_mkt_item_data(&mut data, ItemType::Kart);
    update_mkt_item_data(&mut data, ItemType::Glider);
    update_mkt_item_coverage_data(&mut data);
    data
}
