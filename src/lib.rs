#![allow(dead_code)]
mod data;

use std::{cmp::Ordering, fs};

use image::{
    buffer::ConvertBuffer,
    imageops::{self, FilterType},
    GrayImage, ImageBuffer, Luma, Rgb,
};
use imageproc::{
    drawing, map,
    rect::Rect,
    template_matching::{self, MatchTemplateMethod},
};
use itertools::Itertools;

use crate::data::*;

pub fn draw_line() {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let mut source = image::open("tests/mkt_drivers.jpg").unwrap().into_rgb8();
    let levels: Vec<_> = vec![
        (1, 0.3),
        (2, 0.3),
        (3, 0.3),
        (4, 0.3),
        (5, 0.3),
        (6, 0.3),
        (7, 0.3),
    ]
    .into_iter()
    .map(|(lvl, threshold)| {
        (
            lvl,
            image::open(format!("templates/levels/{}.png", lvl))
                .unwrap()
                .into_luma8(),
            threshold,
        )
    })
    .collect();

    let drivers: Vec<_> = fs::read_dir("templates/drivers")
        .unwrap()
        .into_iter()
        .map(|p| {
            let p = p.unwrap();
            let img = image::open(p.path()).unwrap().into_rgb8();
            (
                p.file_name().to_str().unwrap().to_owned(),
                map::red_channel(&img),
                map::green_channel(&img),
                map::blue_channel(&img),
                0.01,
            )
        })
        .collect();

    // The dimensions method returns the images width and height.
    // println!("dimensions {:?}", img.dimensions());
    let (_width, _height) = source.dimensions();

    // The color method returns the image's `ColorType`.
    // println!("{:?}", img.color());

    let mut img: GrayImage = source.convert();
    // let mut img = map::red_channel(&source);
    // img = imageops::resize(&img, 4000, 4000, FilterType::Triangle); // resizing is not worth it
    img.save("pics/post_1.png").unwrap();

    let h_lines = find_item_lines(&mut img);
    img = imageops::rotate90(&img);
    let v_lines = find_item_lines(&mut img);
    img = imageops::rotate270(&img);

    // let mut img = edges::canny(&img, 10.0, 20.0);
    img.save("pics/post_2.png").unwrap();
    v_lines
        .iter()
        .cartesian_product(h_lines.iter())
        .filter(|((x1, x2), (y1, y2))| (x2 - x1) * (y2 - y1) > 10_000)
        .for_each(|((x1, x2), (y1, y2))| {
            // crop 160 x 200
            let mut crop = imageops::crop(&mut source, *x1, *y1, x2 - x1, y2 - y1).to_image();
            crop = imageops::resize(&crop, 160, 200, FilterType::Triangle);
            crop.save(format!("pics/test_{}_{}.png", y1, x1)).unwrap();

            // item x: 10 - 150  y: 20 - 140
            let item = imageops::crop(&mut crop, 10, 20, 140, 120).to_image();
            item.save(format!("pics/test_{}_{}_char.png", y1, x1))
                .unwrap();
            let item = imageops::resize(&item, 14, 12, FilterType::Triangle);
            let item_r = map::red_channel(&item);
            let item_g = map::green_channel(&item);
            let item_b = map::blue_channel(&item);

            // for easily adding future items
            let item_template = imageops::crop(&mut crop, 30, 30, 100, 100).to_image();
            let item_template = imageops::resize(&item_template, 10, 10, FilterType::Triangle);
            item_template
                .save(format!("pics/test_{}_{}_char_template.png", y1, x1))
                .unwrap();

            // lvl x: 125 - 160  y: 130 - 170
            let lvl = map::green_channel(&imageops::crop(&mut crop, 125, 130, 35, 40).to_image());
            lvl.save(format!("pics/test_{}_{}_lvl.png", y1, x1))
                .unwrap();

            // template testing levels
            let lvl: _ = levels
                .iter()
                .map(|(l, template, t)| (*l, template_score(&lvl, &template), *t))
                .filter(|(_, score, t)| score < t)
                // .collect();
                .min_by(|(_, a, _), (_, b, _)| -> Ordering {
                    a.partial_cmp(b).unwrap_or(Ordering::Equal)
                });

            // template testing drivers
            let driver: Vec<_> = drivers
                .iter()
                .map(|(i, r, g, b, t)| {
                    (
                        i,
                        template_score(&item_r, &r),
                        template_score(&item_g, &g),
                        template_score(&item_b, &b),
                        *t,
                    )
                })
                .filter(|(_, sr, sg, sb, t)| sr < t && sg < t && sb < t)
                .collect();
            // .min_by(|(_, a, _), (_, b, _)| -> Ordering {
            //     a.partial_cmp(b).unwrap_or(Ordering::Equal)
            // });

            dbg!((y1, x1, lvl, driver));

            // debug rectangle position
            drawing::draw_hollow_rect_mut(
                &mut source,
                Rect::at(*x1 as i32, *y1 as i32).of_size(x2 - x1, y2 - y1),
                Rgb([255, 0, 0]),
            );
        });

    // Write the contents of this image to the Writer in PNG format.
    source.save("pics/source_rect_.png").unwrap();
}

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

fn template_score(image: &GrayImage, template: &GrayImage) -> f32 {
    *template_matching::match_template(
        image,
        template,
        MatchTemplateMethod::SumOfSquaredErrorsNormalized,
    )
    .iter()
    .min_by(|a, b| -> Ordering { a.partial_cmp(b).unwrap_or(Ordering::Equal) })
    .unwrap()
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
