#![allow(dead_code)]
mod data;

use image::{GenericImageView, ImageBuffer, Luma, Rgba, imageops::{self, FilterType}};
use imageproc::{drawing, edges, rect::Rect};
use itertools::Itertools;

use crate::data::*;

pub fn draw_line() {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let mut source = image::open("tests/mkt_drivers.jpg").unwrap();

    // The dimensions method returns the images width and height.
    // println!("dimensions {:?}", img.dimensions());
    let (_width, _height) = source.dimensions();

    // The color method returns the image's `ColorType`.
    // println!("{:?}", img.color());

    let mut img = source.clone().into_luma8();
    // img = imageops::resize(&img, 4000, 4000, FilterType::Triangle);
    img.save("pics/post_1.png").unwrap();

    fn find_item_lines(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<(u32, u32)> {
        let mut item_lines = vec![];
        let mut prev_is_item = None;
        let mut item_line = None;
        for (y, ps) in img.enumerate_rows() {
            let mut is_item = false;
            let mut min_luma_shifts = 100;
            let mut prev_luma = None;
            for (_x, _y, p) in ps {
                let Luma([luma]) = *p;
                if let Some(prev_luma) = prev_luma {
                    if (luma as i16 - prev_luma as i16).abs() > 5 {
                        if min_luma_shifts == 0 {
                            is_item = true;
                            break;
                        }
                        min_luma_shifts -= 1;
                    }
                }
                // luma average
                prev_luma = Some(((luma as u16 + prev_luma.unwrap_or(luma) as u16) / 2) as u8);
            }
            if prev_is_item != Some(is_item) {
                if is_item {
                    item_line = Some(y);
                } else if let Some(l) = item_line {
                    // remove small boxes
                    if y - l > 3 {
                        // merge small gaps
                        if let Some(il @ (y1, y2)) = item_lines.pop() {
                            if l - y2 > 3 {
                                item_lines.push(il);
                                item_lines.push((l, y));
                            } else {
                                item_lines.push((y1, y));
                            }
                        } else {
                            item_lines.push((l, y));
                        }
                    }
                    item_line = None;
                }
            }
            prev_is_item = Some(is_item);
        }
        if let Some(l) = item_line {
            item_lines.push((l, img.dimensions().1));
        }
        item_lines
    }

    let h_lines = find_item_lines(&mut img);
    img = imageops::rotate90(&img);
    let v_lines = find_item_lines(&mut img);
    img = imageops::rotate270(&img);

    let mut img = edges::canny(&img, 1.0, 10.0);
    v_lines
        .iter()
        .cartesian_product(h_lines.iter())
        // .filter(|((x1, x2), (y1, y2))| (x2 - x1) * (y2 - y1) > 10_000)
        .for_each(|((x1, x2), (y1, y2))| {
            // crop 160 x 200
            let mut crop = imageops::crop(&mut img, *x1, *y1, x2 - x1, y2 - y1).to_image();
            crop = imageops::resize(&crop, 160, 200, FilterType::Triangle);
            crop.save(format!("pics/test_{}_{}.png", y1, x1)).unwrap();
            // item x: 0 - 160  y: 20 - 135
            let item = imageops::crop(&mut crop, 0, 20, 160, 115).to_image();
            item.save(format!("pics/test_{}_{}_char.png", y1, x1))
                .unwrap();
            imageops::resize(
                &imageops::resize(&item, 16, 16, FilterType::Triangle),
                100,
                100,
                FilterType::Nearest,
            )
            .save(format!("pics/test_{}_{}_char_pix.png", y1, x1))
            .unwrap();
            // lvl x: 125 - 160  y: 133 - 165
            let lvl = imageops::crop(&mut crop, 125, 133, 35, 32).to_image();
            lvl.save(format!("pics/test_{}_{}_lvl.png", y1, x1))
                .unwrap();
            imageops::resize(
                &imageops::resize(&lvl, 8, 8, FilterType::Triangle),
                100,
                100,
                FilterType::Nearest,
            )
            .save(format!("pics/test_{}_{}_lvl_pix.png", y1, x1))
            .unwrap();
            drawing::draw_hollow_rect_mut(
                &mut source,
                Rect::at(*x1 as i32, *y1 as i32).of_size(x2 - x1, y2 - y1),
                Rgba([255, 0, 0, 255]),
            );
        });

    // Write the contents of this image to the Writer in PNG format.
    source.save("pics/source_rect_.png").unwrap();
}

fn update_mkt_data() {
    // TODO: get data (from Super Mario Wiki?)

    // TODO merge data

    // TODO: store data (pull request?)
}

fn import_screenshot(inv: &mut MktInventory, _img: Vec<u8>) {
    // TODO: maybe get the picture?

    // update inventory
    inv.update_inventory(screenshot_to_inventory(_img));

    // TODO: save inventory
}

fn screenshot_to_inventory(_img: Vec<u8>) -> MktInventory {
    // TODO: identify square

    // TODO: identify character/item

    // TODO: identify level

    unimplemented!();
}
