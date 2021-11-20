use std::path::PathBuf;

use itertools::Itertools;
use mkt_data::{ItemType, MktData, MktItemHashes};
use mkt_import::screenshot::{dist_hash, screenshots_to_bootstrap_hashes, HASH_ITEM_THRESHOLD};

pub fn get_test_hash_data() -> MktData {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    MktData::load(d.join("test_hash_mkt_data.json").to_str().unwrap()).unwrap()
}

pub fn get_test_screenshot(
    i_type: &str,
    screenshot_name: &str,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push(i_type);
    image::open(d.join(screenshot_name)).unwrap().into_rgb8()
}

#[test]
fn hash_distance() {
    let mut h = MktItemHashes::new();
    let mut fail = false;
    let data = get_test_hash_data();
    for (i, t, n) in [
        (ItemType::Driver, "drivers", 6),
        (ItemType::Kart, "karts", 8),
        (ItemType::Glider, "gliders", 6),
    ] {
        let list = (1..=n)
            .map(|i| get_test_screenshot(t, &format!("mkt_{}_{}.jpg", t, i)))
            .collect_vec();
        let hashes = screenshots_to_bootstrap_hashes(list, i, &data).unwrap();
        h.merge(hashes.clone());
        let hashes = hashes.hashes;
        for (e, (id1, hs1)) in hashes.iter().enumerate() {
            for (id2, hs2) in hashes.iter().skip(e) {
                if id1 != id2 {
                    for (h1, h2) in hs1.iter().cartesian_product(hs2.iter()) {
                        let d = dist_hash(h1, h2);
                        if d <= HASH_ITEM_THRESHOLD * 3 / 2 {
                            fail = true;
                            println!("{} == {}: {}", id1, id2, d);
                        }
                    }
                }
            }
        }
    }
    h.save("pics/mkt_hash.json").unwrap();
    if fail {
        panic!("some items have hashes that are too close");
    }
}
