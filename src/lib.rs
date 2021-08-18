#![allow(dead_code)]
mod data;

use data::*;

use std::{cmp::Ordering, collections::HashMap, fs};

use image::{
    buffer::ConvertBuffer,
    imageops::{self, FilterType},
    GrayImage, Luma, Rgb, RgbImage,
};
use imageproc::{
    drawing, map,
    rect::Rect,
    template_matching::{self, MatchTemplateMethod},
};
use itertools::Itertools;

const DEFAULT_ITEM_WIDTH: u32 = 160;
const DEFAULT_ITEM_HEIGHT: u32 = 200;
const LVL_THRESHOLD: f32 = 0.3;
const ITEM_THRESHOLD: f32 = 0.05;

const DEBUG_IMG: bool = true;

type LvlTemplates = Vec<(ItemLvl, GrayImage)>;
type ItemTemplates = Vec<(ItemId, GrayImage, GrayImage, GrayImage)>;

pub fn get_database() -> MktDatabase {
    let courses = HashMap::new();
    let mut drivers = HashMap::new();
    for (id, template) in get_item_templates("drivers") {
        drivers.insert(
            id.clone(),
            Item::with_id_and_template(id, ItemType::Driver, template),
        );
    }
    let mut karts = HashMap::new();
    for (id, template) in get_item_templates("karts") {
        karts.insert(
            id.clone(),
            Item::with_id_and_template(id, ItemType::Kart, template),
        );
    }
    let mut gliders = HashMap::new();
    for (id, template) in get_item_templates("gliders") {
        gliders.insert(
            id.clone(),
            Item::with_id_and_template(id, ItemType::Glider, template),
        );
    }
    MktDatabase {
        courses,
        drivers,
        karts,
        gliders,
    }
}

fn get_lvl_templates() -> LvlTemplates {
    let levels_templates: Vec<_> = (1..=7)
        .into_iter()
        .map(|lvl| {
            (
                lvl,
                image::open(format!("templates/levels/{}.png", lvl))
                    .unwrap()
                    .into_luma8(),
            )
        })
        .collect();
    levels_templates
}

fn get_item_templates(i_type: &str) -> Vec<(String, RgbImage)> {
    let items_templates: Vec<_> = Some(i_type)
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
                img,
            )
        })
        .collect();
    items_templates
}

// TODO: remove once I figure out debugging again
fn draw_line() {

    // item_areas.for_each(|ItemArea { x1, x2, y1, y2 }| {
    //     dbg!((y1, x1, lvl, item));

    //     // for easily adding future items
    //     if lvl.is_some() || DEBUG_IMG {
    //         let item_template = imageops::crop(&mut crop, 30, 30, 100, 100).to_image();
    //         let item_template = imageops::resize(&item_template, 10, 10, FilterType::Triangle);
    //         item_template
    //             .save(format!("pics/test_{}_{}_item_template.png", y1, x1))
    //             .unwrap();
    //     } else {
    //         crop.save(format!("pics/test_{}_{}.png", y1, x1)).unwrap();
    //     }

    //     if DEBUG_IMG {
    //         // debug rectangle position
    //         drawing::draw_hollow_rect_mut(
    //             &mut source,
    //             Rect::at(*x1 as i32, *y1 as i32).of_size(x2 - x1, y2 - y1),
    //             Rgb([255, 0, 0]),
    //         );
    //     }
    // });

    // if DEBUG_IMG {
    //     // Write the contents of this image to the Writer in PNG format.
    //     source.save("pics/source_rect_.png").unwrap();
    // }
}

#[derive(Debug, Clone)]
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
    fn swap_x_y(mut self) -> Self {
        std::mem::swap(&mut self.x1, &mut self.y1);
        std::mem::swap(&mut self.x2, &mut self.y2);
        self
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

fn find_item_areas(img: &GrayImage) -> impl Iterator<Item = ItemArea> {
    if DEBUG_IMG {
        img.save("pics/find_item_areas.png").unwrap();
    }

    let rows = find_item_rows(img).into_iter();
    let img = &imageops::rotate90(img);
    let columns = find_item_rows(img).into_iter().map(ItemArea::swap_x_y);

    columns
        .cartesian_product(rows)
        .flat_map(|(c, r)| c.intersect(&r))
        .filter(|a| a.size() >= 10_000)
}

fn item_area_to_image(ItemArea { x1, x2, y1, y2 }: ItemArea, screenshot: &RgbImage) -> RgbImage {
    // crop 160 x 200
    let crop = imageops::crop_imm(screenshot, x1, y1, x2 - x1, y2 - y1).to_image();
    let crop = imageops::resize(
        &crop,
        DEFAULT_ITEM_WIDTH,
        DEFAULT_ITEM_HEIGHT,
        FilterType::Triangle,
    );
    if DEBUG_IMG {
        crop.save(format!("pics/test_{}_{}.png", y1, x1)).unwrap();
    }
    crop
}

fn item_level_from_image(img: &RgbImage, templates: &LvlTemplates) -> Option<ItemLvl> {
    // lvl x: 125 - 160  y: 130 - 170
    let lvl = map::green_channel(&imageops::crop_imm(img, 125, 130, 35, 40).to_image());
    // if DEBUG_IMG {
    //     lvl.save(format!("pics/test_{}_{}_lvl.png", y1, x1))
    //         .unwrap();
    // }

    // template testing levels
    let lvl = templates
        .iter()
        .map(|(l, template)| (*l, template_score(&lvl, &template)))
        .filter(|(_, score)| *score < LVL_THRESHOLD)
        // .collect();
        .min_by(|(_, a), (_, b)| -> Ordering { a.partial_cmp(b).unwrap_or(Ordering::Equal) });

    lvl.map(|l| l.0)
}

fn item_id_from_image(img: &RgbImage, templates: &ItemTemplates) -> Option<ItemId> {
    // item x: 10 - 150  y: 20 - 140
    let item = imageops::crop_imm(img, 10, 20, 140, 120).to_image();
    // if DEBUG_IMG {
    //     item.save(format!("pics/test_{}_{}_item.png", y1, x1))
    //         .unwrap();
    // }
    let item = imageops::resize(&item, 14, 12, FilterType::Triangle);
    let item_r = map::red_channel(&item);
    let item_g = map::green_channel(&item);
    let item_b = map::blue_channel(&item);

    // template testing drivers
    let item = templates
        .iter()
        .map(|(i, r, g, b)| {
            (
                i,
                template_score(&item_r, &r),
                template_score(&item_g, &g),
                template_score(&item_b, &b),
            )
        })
        .filter(|(_, sr, sg, sb)| {
            *sr < ITEM_THRESHOLD && *sg < ITEM_THRESHOLD && *sb < ITEM_THRESHOLD
        })
        // .collect();
        .min_by(|(_, ar, ab, ag), (_, br, bb, bg)| -> Ordering {
            (ar, ab, ag)
                .partial_cmp(&(br, bb, bg))
                .unwrap_or(Ordering::Equal)
        });

    item.map(|i| i.0.clone())
}

fn item_image_to_template(img: &RgbImage) -> RgbImage {
    let item_template = imageops::crop_imm(img, 30, 30, 100, 100).to_image();
    let item_template = imageops::resize(&item_template, 10, 10, FilterType::Triangle);
    // item_template
    //     .save(format!("pics/test_{}_{}_item_template.png", y1, x1))
    //     .unwrap();
    item_template
}

enum OwnedItemResult {
    Found(OwnedItem),
    NotFound(RgbImage, ItemLvl),
    Invalid,
}

fn item_image_to_owned_item(
    img: &RgbImage,
    lvl_templates: &LvlTemplates,
    item_templates: &ItemTemplates,
) -> OwnedItemResult {
    use OwnedItemResult::*;

    let id = item_id_from_image(img, item_templates);
    let lvl = item_level_from_image(img, lvl_templates);

    if let Some(lvl) = lvl {
        if let Some(id) = id {
            Found(OwnedItem {
                id: id,
                lvl: lvl,
                points: 0,
            })
        } else {
            NotFound(img.clone(), lvl)
        }
    } else {
        Invalid
    }
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

fn import_screenshot(inv: &mut MktInventory, img: RgbImage, data: &MktDatabase) {
    // TODO: might be a client side function

    // TODO: get the picture

    // update inventory
    inv.update_inventory(screenshots_to_inventory(vec![img], data).0);

    // TODO: save inventory
}

pub fn screenshots_to_inventory(
    screenshots: Vec<RgbImage>,
    data: &MktDatabase,
) -> (MktInventory, Vec<(RgbImage, ItemLvl)>) {
    let lvl_templates = get_lvl_templates();
    let item_templates = data
        .drivers
        .iter()
        .chain(data.karts.iter())
        .chain(data.gliders.iter())
        .flat_map(|(id, i)| i.templates.iter().map(move |t| (id, t)))
        .map(|(id, t)| {
            (
                id.clone(),
                map::red_channel(t),
                map::green_channel(t),
                map::blue_channel(t),
            )
        })
        .collect_vec();

    let mut inv = MktInventory::new();
    let mut missing = vec![];

    for screenshot in screenshots {
        // identify square
        let item_areas = find_item_areas(&screenshot.convert());

        // identify character/item and levels
        let mut items = vec![];
        for img in item_areas.map(|area| item_area_to_image(area, &screenshot)) {
            let r = item_image_to_owned_item(&img, &lvl_templates, &item_templates);
            match r {
                OwnedItemResult::Found(item) => items.push(item),
                OwnedItemResult::NotFound(img, lvl) => missing.push((img, lvl)),
                OwnedItemResult::Invalid => {}
            }
        }

        inv.update_inventory(MktInventory::from_items(items, data));
    }

    (inv, missing)
}
