mod drivers;
mod gliders;
mod hash;
mod karts;

use std::fs;
use std::path::PathBuf;

use mkt_data::MktData;
use mkt_import::screenshot::screenshots_to_inventory;

use pretty_assertions::assert_eq;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(PartialEq, Eq)]
pub struct PrettyPrint<T: Display + Eq>(T);

impl<T: Display + Eq> Debug for PrettyPrint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Display + Eq> Display for PrettyPrint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn compare_screenshot(i_type: &str, screenshot_name: &str, json_name: &str) {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push(i_type);
    println!("{:?}", d);
    let data = MktData::load(d.join("../../../data/mkt_data.json").to_str().unwrap()).unwrap();
    let screenshot = image::open(d.join(screenshot_name)).unwrap().into_rgb8();
    let list = vec![screenshot];
    let (mut inventory, _) = screenshots_to_inventory(list, &data, None);
    inventory.clear_dates();
    let json_1 = inventory.to_json().unwrap();
    let json_2 = fs::read_to_string(d.join(json_name)).unwrap();

    assert_eq!(PrettyPrint(json_1), PrettyPrint(json_2));
}
