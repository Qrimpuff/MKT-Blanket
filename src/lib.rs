#![allow(dead_code)]
pub mod data;
pub mod screenshot;

pub use data::*;
use screenshot::*;

use image::RgbImage;

fn update_mkt_data() {
    // TODO: get data (from Super Mario Wiki?)

    // TODO merge data

    // TODO: store data (pull request?)
}

fn import_screenshot(inv: &mut MktInventory, img: RgbImage, data: &MktDatabase) {
    // TODO: might be a client side function

    // TODO: get the picture

    // update inventory
    inv.update_inventory(screenshots_to_inventory(vec![img], data).0);

    // TODO: save inventory
}
