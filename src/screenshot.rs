#![allow(dead_code)]

use crate::data::*;

use std::cmp::Ordering;

use image::{
    buffer::ConvertBuffer,
    imageops::{self, FilterType},
    DynamicImage, GenericImageView, GrayImage, Luma, RgbImage, RgbaImage,
};
use imageproc::{
    contrast::threshold_mut,
    map,
    rect::Rect,
    template_matching::{self, MatchTemplateMethod},
};
use itertools::Itertools;

const DEFAULT_ITEM_WIDTH: u32 = 160;
const DEFAULT_ITEM_HEIGHT: u32 = 200;
const LVL_THRESHOLD: f32 = 0.5;
const ITEM_THRESHOLD: f32 = 0.05;

const DEBUG_IMG: bool = true;

type LvlTemplates = Vec<(ItemLvl, GrayImage)>;
type ItemTemplates = Vec<(ItemId, GrayImage, GrayImage, GrayImage)>;

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
    const LUMA_SHIFT_DIFF: i16 = 5;
    const LUMA_SHIFT_RATIO: f32 = 0.15;

    let (width, height) = img.dimensions();
    let mut item_rows = vec![];
    let mut prev_is_item = None;
    let mut item_line = None;
    for (y, ps) in img.enumerate_rows() {
        let mut luma_shifts = 0;
        let mut prev_luma = None;
        for (_x, _y, p) in ps {
            let Luma([luma]) = *p;
            if let Some(prev_luma) = prev_luma {
                if (luma as i16 - prev_luma as i16).abs() > LUMA_SHIFT_DIFF {
                    luma_shifts += 1;
                }
            }
            // luma average
            prev_luma = Some(((luma as u16 + prev_luma.unwrap_or(luma) as u16) / 2) as u8);
        }
        let is_item = luma_shifts as f32 / width as f32 > LUMA_SHIFT_RATIO;
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
        FilterType::Gaussian,
    );
    if DEBUG_IMG {
        crop.save(format!("pics/test_{:?}_{}_{}.png", &crop.as_ptr(), y1, x1))
            .unwrap();
    }
    crop
}

fn item_level_from_image(img: &RgbImage, templates: &LvlTemplates) -> Option<ItemLvl> {
    // lvl x: 125 - 160  y: 130 - 170
    let mut img = map::green_channel(&imageops::crop_imm(img, 125, 130, 35, 40).to_image());
    threshold_mut(&mut img, 150);

    // template testing levels
    let lvl = templates
        .iter()
        .map(|(l, template)| (*l, template_score(&img, &template)))
        .inspect(|i| {
            println!("lvl points: {:#?}", i);
        })
        .filter(|(_, score)| *score < LVL_THRESHOLD)
        // .collect();
        .min_by(|(_, a), (_, b)| -> Ordering { a.partial_cmp(b).unwrap_or(Ordering::Equal) });

    if DEBUG_IMG {
        img.save(format!(
            "pics/test_{:?}_{:?}_lvl.png",
            &img.as_ptr(),
            &lvl.map(|l| l.0)
        ))
        .unwrap();
    }
    dbg!(lvl.map(|l| l.0))
}

fn item_id_from_image(img: &RgbImage, templates: &ItemTemplates) -> Option<ItemId> {
    // item x: 10 - 150  y: 20 - 140
    let item = imageops::crop_imm(img, 0, 0, 160, 150).to_image();
    if DEBUG_IMG {
        item.save(format!("pics/test_{:?}_item.png", &img.as_ptr()))
            .unwrap();
    }
    let item = imageops::resize(&item, 16, 15, FilterType::Gaussian);
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
        .map(|(i, r, g, b)| (i, r * g * b))
        .filter(|(_, p)| *p < ITEM_THRESHOLD * ITEM_THRESHOLD * ITEM_THRESHOLD)
        .inspect(|i| {
            println!("points: {:#?}", i);
        })
        .min_by(|(_, p1), (_, p2)| -> Ordering { p1.partial_cmp(&p2).unwrap_or(Ordering::Equal) });

    item.map(|i| i.0.clone())
}

fn item_image_to_template(item: &Item, i: u32, img: &RgbImage) -> RgbImage {
    let item_template = imageops::crop_imm(img, 30, 30, 100, 100).to_image();
    let item_template = imageops::resize(&item_template, 10, 10, FilterType::Gaussian);
    if DEBUG_IMG {
        item_template
            .save(format!("templates/{}s/{}_{}.png", item.i_type, item.id, i))
            .unwrap();
    }
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

    println!("-------");
    let lvl = item_level_from_image(img, lvl_templates);
    if let Some(lvl) = lvl {
        let id = item_id_from_image(img, item_templates);
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

// pub fn test_overlay() {
//     let mut img1 = image::open("tests/39px-MKT_Icon_Normal.png")
//         .unwrap()
//         .resize_exact(160, 200, FilterType::Gaussian);
//     let img1_w = dbg!(&img1.dimensions()).0;
//     let img2 = image::open("tests/44px-BabyMarioSluggers.png")
//         .unwrap()
//         .resize(160, 200, FilterType::Gaussian);
//     let img2_w = dbg!(&img2.dimensions()).0;
//     imageops::overlay(&mut img1, &img2, dbg!(&(img1_w - img2_w)) / 2, 10);
//     img1.save("tests/test_overlay.png").unwrap();
//     item_image_to_template(&img1.into_rgb8());
//     item_image_to_template(
//         &image::open("tests/test_0x1c37b87d900_215_46.png")
//             .unwrap()
//             .into_rgb8(),
//     );
// }

pub fn create_template(item: &Item, img: RgbaImage) {
    for i in (70..=160).step_by(45) {
        let mut bg = match item.rarity {
            Rarity::Normal => image::open("tests/39px-MKT_Icon_Normal.png"),
            Rarity::Super => image::open("tests/39px-MKT_Icon_Rare.png"),
            Rarity::HighEnd => image::open("tests/39px-MKT_Icon_HighEnd.png"),
        }
        .unwrap()
        .resize_exact(
            DEFAULT_ITEM_WIDTH,
            DEFAULT_ITEM_HEIGHT,
            FilterType::Gaussian,
        );
        let bg_w = bg.dimensions().0;

        let (og_width, og_height) = img.dimensions();
        // find a better center for the image
        let (center_x, center_y) = find_center_of_mass(&img);
        let ratio = DEFAULT_ITEM_HEIGHT as f32 / center_y as f32 * i as f32 / 100.0;
        let mut img = DynamicImage::ImageRgba8(img.clone()).resize_exact(
            (og_width as f32 * ratio) as u32,
            (og_height as f32 * ratio) as u32,
            FilterType::Gaussian,
        );
        let mut img_x: i32 = bg_w as i32 / 2 - (center_x as f32 * ratio) as i32;
        // the image is too big
        if img_x < 0 {
            img = img.crop_imm((-img_x) as u32, 0, DEFAULT_ITEM_WIDTH, DEFAULT_ITEM_HEIGHT);
            img_x = 0;
        }

        imageops::overlay(&mut bg, &img, img_x as u32, 20);
        bg.save(format!("pics/{}_{}.png", item.id, i)).unwrap();
        item_image_to_template(item, i, &bg.into_rgb8());
    }
}

pub fn find_center_of_mass(img: &RgbaImage) -> (u32, u32) {
    let total_pixel_count = img.pixels().filter(|p| p.0[3] > 0).count();
    let mut center_y = 0;
    let mut pixels = 0;
    for (y, ps) in img.enumerate_rows() {
        pixels += ps.filter(|(_, _, p)| p.0[3] > 0).count();
        if pixels as f32 >= total_pixel_count as f32 * 0.9 {
            center_y = y;
            break;
        }
    }
    let img = imageops::rotate90(img);
    let mut center_x = 0;
    let mut pixels = 0;
    for (x, ps) in img.enumerate_rows() {
        pixels += ps.filter(|(_, _, p)| p.0[3] > 0).count();
        if pixels as f32 >= total_pixel_count as f32 * 0.5 {
            center_x = x;
            break;
        }
    }
    (center_x, center_y)
}
