#![allow(dead_code)]
mod data;

use std::{cmp::Ordering, fs};

use image::{
    buffer::ConvertBuffer,
    imageops::{self, FilterType},
    GrayImage, Luma, Rgb,
};
use imageproc::{
    drawing, map,
    rect::Rect,
    template_matching::{self, MatchTemplateMethod},
};
use itertools::Itertools;

use crate::data::*;

const DEFAULT_ITEM_WIDTH: u32 = 160;
const DEFAULT_ITEM_HEIGHT: u32 = 200;
const LVL_THRESHOLD: f32 = 0.3;
const ITEM_THRESHOLD: f32 = 0.05;

const DEBUG_IMG: bool = true;

pub fn draw_line() {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let mut source = image::open("tests/mkt_drivers.jpg").unwrap().into_rgb8();
    let levels_templates: Vec<_> = vec![
        (1, LVL_THRESHOLD),
        (2, LVL_THRESHOLD),
        (3, LVL_THRESHOLD),
        (4, LVL_THRESHOLD),
        (5, LVL_THRESHOLD),
        (6, LVL_THRESHOLD),
        (7, LVL_THRESHOLD),
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

    let items_templates: Vec<_> = ["drivers", "karts", "gliders"]
        .iter()
        .flat_map(|ty| fs::read_dir(format!("templates/{}", ty)).unwrap())
        .map(|p| {
            let p = p.unwrap();
            let img = image::open(p.path()).unwrap().into_rgb8();
            (
                format!(
                    "{}_{}",
                    p.path()
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    p.file_name().to_str().unwrap()
                ),
                map::red_channel(&img),
                map::green_channel(&img),
                map::blue_channel(&img),
                ITEM_THRESHOLD,
            )
        })
        .collect();

    let img: GrayImage = source.convert();
    if DEBUG_IMG {
        img.save("pics/post_1.png").unwrap();
    }

    let item_areas = find_item_areas(&img);

    item_areas.iter().for_each(|ItemArea { x1, x2, y1, y2 }| {
        // crop 160 x 200
        let mut crop = imageops::crop(&mut source, *x1, *y1, x2 - x1, y2 - y1).to_image();
        crop = imageops::resize(
            &crop,
            DEFAULT_ITEM_WIDTH,
            DEFAULT_ITEM_HEIGHT,
            FilterType::Triangle,
        );
        if DEBUG_IMG {
            crop.save(format!("pics/test_{}_{}.png", y1, x1)).unwrap();
        }

        // item x: 10 - 150  y: 20 - 140
        let item = imageops::crop(&mut crop, 10, 20, 140, 120).to_image();
        if DEBUG_IMG {
            item.save(format!("pics/test_{}_{}_item.png", y1, x1))
                .unwrap();
        }
        let item = imageops::resize(&item, 14, 12, FilterType::Triangle);
        let item_r = map::red_channel(&item);
        let item_g = map::green_channel(&item);
        let item_b = map::blue_channel(&item);

        // lvl x: 125 - 160  y: 130 - 170
        let lvl = map::green_channel(&imageops::crop(&mut crop, 125, 130, 35, 40).to_image());
        if DEBUG_IMG {
            lvl.save(format!("pics/test_{}_{}_lvl.png", y1, x1))
                .unwrap();
        }

        // template testing levels
        let lvl = levels_templates
            .iter()
            .map(|(l, template, t)| (*l, template_score(&lvl, &template), *t))
            .filter(|(_, score, t)| score < t)
            // .collect();
            .min_by(|(_, a, _), (_, b, _)| -> Ordering {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            });

        // template testing drivers
        let item = items_templates
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
            // .collect();
            .min_by(|(_, ar, ab, ag, _), (_, br, bb, bg, _)| -> Ordering {
                (ar, ab, ag)
                    .partial_cmp(&(br, bb, bg))
                    .unwrap_or(Ordering::Equal)
            });

        dbg!((y1, x1, lvl, item));

        // for easily adding future items
        if lvl.is_some() || DEBUG_IMG {
            let item_template = imageops::crop(&mut crop, 30, 30, 100, 100).to_image();
            let item_template = imageops::resize(&item_template, 10, 10, FilterType::Triangle);
            item_template
                .save(format!("pics/test_{}_{}_item_template.png", y1, x1))
                .unwrap();
        } else {
            crop.save(format!("pics/test_{}_{}.png", y1, x1)).unwrap();
        }

        if DEBUG_IMG {
            // debug rectangle position
            drawing::draw_hollow_rect_mut(
                &mut source,
                Rect::at(*x1 as i32, *y1 as i32).of_size(x2 - x1, y2 - y1),
                Rgb([255, 0, 0]),
            );
        }
    });

    if DEBUG_IMG {
        // Write the contents of this image to the Writer in PNG format.
        source.save("pics/source_rect_.png").unwrap();
    }
}

#[derive(Debug)]
struct ItemArea {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
}

impl ItemArea {
    fn from_rect(rect: Rect) -> Self {
        ItemArea {
            x1: rect.left() as u32,
            y1: rect.top() as u32,
            x2: rect.left() as u32 + rect.width(),
            y2: rect.top() as u32 + rect.height(),
        }
    }
    fn to_rect(&self) -> Rect {
        Rect::at(self.x1 as i32, self.y1 as i32).of_size(self.x2 - self.x1, self.y2 - self.y1)
    }
    fn intersect(&self, other: &ItemArea) -> Option<ItemArea> {
        self.to_rect()
            .intersect(other.to_rect())
            .map(ItemArea::from_rect)
    }
    fn size(&self) -> u32 {
        (self.x2 - self.x1) * (self.y2 - self.y1)
    }
    fn swap_x_y(&mut self) {
        std::mem::swap(&mut self.x1, &mut self.y1);
        std::mem::swap(&mut self.x2, &mut self.y2);
    }
}

fn find_item_rows(img: &GrayImage) -> Vec<ItemArea> {
    let (width, height) = img.dimensions();
    let mut item_rows = vec![];
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
                    if let Some(ir @ ItemArea { y1, y2, .. }) = item_rows.pop() {
                        if l - y2 > 3 {
                            item_rows.push(ir);
                            item_rows.push(ItemArea {
                                x1: 0,
                                y1: l,
                                x2: width,
                                y2: y,
                            });
                        } else {
                            item_rows.push(ItemArea {
                                x1: 0,
                                y1: y1,
                                x2: width,
                                y2: y,
                            });
                        }
                    } else {
                        item_rows.push(ItemArea {
                            x1: 0,
                            y1: l,
                            x2: width,
                            y2: y,
                        });
                    }
                }
                item_line = None;
            }
        }
        prev_is_item = Some(is_item);
    }
    if let Some(l) = item_line {
        item_rows.push(ItemArea {
            x1: 0,
            y1: l,
            x2: width,
            y2: height,
        });
    }
    item_rows
}

fn find_item_areas(img: &GrayImage) -> Vec<ItemArea> {
    let rows = find_item_rows(img);
    dbg!(&rows);
    let img = &imageops::rotate90(img);
    let mut columns = find_item_rows(img);
    columns.iter_mut().for_each(ItemArea::swap_x_y);
    dbg!(&columns);

    columns
        .iter()
        .cartesian_product(rows.iter())
        .flat_map(|(c, r)| c.intersect(r))
        .filter(|a| a.size() >= 10_000)
        .collect()
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
    inv.update_inventory(screenshot_to_inventory(_img, todo!()));

    // TODO: save inventory
}

fn screenshot_to_inventory(_img: Vec<u8>, database: &MktDatabase) -> MktInventory {
    // TODO: identify square

    // TODO: identify character/item

    // TODO: identify level

    unimplemented!();
}
