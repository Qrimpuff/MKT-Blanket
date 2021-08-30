#![allow(dead_code)]

use crate::data::*;

use std::{cmp::Ordering, fs};

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
use img_hash::{HasherConfig, ImageHash};
use itertools::Itertools;

const DEFAULT_ITEM_WIDTH: u32 = 160;
const DEFAULT_ITEM_HEIGHT: u32 = 200;
const DEFAULT_ITEM_RATIO: f32 = DEFAULT_ITEM_WIDTH as f32 / DEFAULT_ITEM_HEIGHT as f32;
const ITEM_RATIO_THRESHOLD: f32 = 0.1;

const LVL_THRESHOLD: f32 = 0.6;
const ITEM_THRESHOLD: f32 = 0.05;
const ITEM_HASH_THRESHOLD: u32 = 600;

const DEBUG_IMG: bool = true;

type LvlTemplates = Vec<(ItemLvl, GrayImage)>;
type ItemTemplates = Vec<(ItemId, GrayImage, GrayImage, GrayImage)>;
type ItemHashes = Vec<(ItemId, String)>;

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

#[derive(Debug, Copy, Clone)]
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
    fn ratio(&self) -> f32 {
        (self.x2 - self.x1) as f32 / (self.y2 - self.y1) as f32
    }
    fn swap_x_y(mut self) -> Self {
        std::mem::swap(&mut self.x1, &mut self.y1);
        std::mem::swap(&mut self.x2, &mut self.y2);
        self
    }
}

fn find_item_rows(img: &GrayImage) -> Vec<ItemArea> {
    const LUMA_SHIFT_DIFF: i16 = 5;
    const LUMA_SHIFT_RATIO: f32 = 0.10;

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

    rows.cartesian_product(columns)
        .flat_map(|(c, r)| c.intersect(&r))
        .filter(|a| (a.ratio() - DEFAULT_ITEM_RATIO).abs() < ITEM_RATIO_THRESHOLD)
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
        crop.save(format!("pics/test_{}_{}.png", y1, x1)).unwrap();
    }
    crop
}

fn item_level_from_image(
    ItemArea { x1, y1, .. }: ItemArea,
    img: &RgbImage,
    templates: &LvlTemplates,
) -> Option<ItemLvl> {
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
            "pics/test_{}_{}_{:?}_lvl.png",
            y1,
            x1,
            &lvl.map(|l| l.0)
        ))
        .unwrap();
    }
    println!("best lvl: {:?}", lvl);

    lvl.map(|l| l.0)
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
    println!("best item: {:?}", item);

    item.map(|i| i.0.clone())
}

fn item_id_from_image_h(
    ItemArea { x1, y1, .. }: ItemArea,
    img: &RgbImage,
    hashes: &ItemHashes,
) -> Option<ItemId> {
    // item x: 10 - 150  y: 20 - 140
    let item_img = imageops::crop_imm(img, 10, 20, 140, 120).to_image();

    let hash = to_image_hash(&item_img);

    // template testing drivers
    let item = hashes
        .iter()
        .map(|(i, h)| (i, dist_hash(&hash, h)))
        .filter(|(_, p)| *p < ITEM_HASH_THRESHOLD)
        .inspect(|i| {
            println!("points h: {:#?}", i);
        })
        .min_by_key(|i| i.1);

    if DEBUG_IMG {
        item_img
            .save(format!(
                "pics/test_{}_{}_{}_item.png",
                y1,
                x1,
                item.unwrap_or((&"none".to_string(), 0)).0
            ))
            .unwrap();
        fs::write(
            format!(
                "pics/test_{}_{}_{}_item.txt",
                y1,
                x1,
                item.unwrap_or((&"none".to_string(), 0)).0
            ),
            &hash,
        )
        .unwrap();
    }
    println!("best item: {:?}", item);

    item.map(|i| i.0.clone())
}

fn maybe_item_image(img: &RgbImage) -> bool {
    let mut img = map::blue_channel(img);
    threshold_mut(&mut img, 200);
    let mut pixels = img.pixels().map(|x| x.0[0]).counts();
    let blue_percent = pixels.remove(&255).unwrap_or(0) as f32 / img.pixels().count() as f32;
    if DEBUG_IMG {
        img.save(format!("pics/test_{:?}_blue_item.png", blue_percent))
            .unwrap();
    }
    blue_percent < 0.9
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
    area: ItemArea,
    img: &RgbImage,
    lvl_templates: &LvlTemplates,
    item_hashes: &ItemHashes,
) -> OwnedItemResult {
    use OwnedItemResult::*;

    println!("area: {:?}", area);
    let lvl = item_level_from_image(area, img, lvl_templates);
    let id = item_id_from_image_h(area, img, item_hashes);
    if let Some(id) = id {
        Found(OwnedItem {
            id: id,
            lvl: lvl.unwrap_or(0),
            points: 0,
        })
    } else {
        if maybe_item_image(img) {
            NotFound(img.clone(), lvl.unwrap_or(0))
        } else {
            Invalid
        }
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
) -> (
    MktInventory,
    Vec<(RgbImage, ItemLvl)>,
    Vec<(RgbImage, ItemLvl)>,
) {
    let lvl_templates = get_lvl_templates();
    let item_hashes = data
        .drivers
        .iter()
        .chain(data.karts.iter())
        .chain(data.gliders.iter())
        .flat_map(|(id, i)| i.hashes.iter().map(move |h| (id.clone(), h.clone())))
        .collect_vec();

    let mut inv = MktInventory::new();
    let mut owned_missing = vec![];
    let mut not_owned_missing = vec![];

    for screenshot in screenshots {
        // identify square
        let item_areas = find_item_areas(&screenshot.convert());

        // identify character/item and levels
        let mut items = vec![];
        // try to identify items in order
        let mut item_offset = 0;
        let mut last_found_item: Option<(usize, OwnedItem)> = None;
        let mut try_items: Vec<(usize, OwnedItem, RgbImage)> = vec![];
        let mut potential_item_ids = None;
        // the end of the list
        let the_end_id = "<the_end>";
        for (i, (area, item_result)) in item_areas
            .map(|area| (area, item_area_to_image(area, &screenshot)))
            .map(|(area, img)| {
                (
                    area,
                    item_image_to_owned_item(area, &img, &lvl_templates, &item_hashes),
                )
            })
            .filter(|(_, item)| !matches!(item, OwnedItemResult::Invalid))
            .chain(Some((
                ItemArea {
                    x1: 0,
                    y1: 0,
                    x2: 0,
                    y2: 0,
                },
                OwnedItemResult::Found(OwnedItem {
                    id: the_end_id.into(),
                    lvl: 0,
                    points: 0,
                }),
            )))
            .enumerate()
        {
            println!("{} - x:{} y:{}", i, area.x1, area.y1);
            match item_result {
                OwnedItemResult::Found(item) => {
                    // assumes that there is only one item type per screenshot
                    if potential_item_ids.is_none() {
                        let potential_items = if item.id.starts_with("d_") {
                            &data.drivers
                        } else if item.id.starts_with("k_") {
                            &data.karts
                        } else {
                            &data.gliders
                        };
                        potential_item_ids = Some(
                            potential_items
                                .values()
                                .map(|i| i.id.clone())
                                .chain(Some(the_end_id.into()))
                                .collect_vec(),
                        );
                    }
                    let potential_item_ids = potential_item_ids.as_ref().unwrap();
                    // find known items
                    if let Some(last_found_item) = last_found_item.as_ref() {
                        if !try_items.is_empty() {
                            // check if expected item
                            if let Some(expected_item_id) =
                                potential_item_ids.get(i - last_found_item.0 + item_offset)
                            {
                                dbg!((&expected_item_id, &item.id));
                                if *expected_item_id == item.id {
                                    for mut t_item in try_items {
                                        let item_id = &potential_item_ids
                                            [t_item.0 - last_found_item.0 + item_offset];
                                        t_item.1.id = item_id.clone();
                                        items.push(t_item.1);
                                        // save_image_hash(item_id, &t_item.2);
                                    }
                                }
                            }
                            try_items = vec![];
                        }
                    }
                    item_offset = potential_item_ids
                        .into_iter()
                        .find_position(|id| **id == item.id)
                        .map(|i| i.0)
                        .unwrap_or(0);
                    last_found_item = Some((i, item.clone()));
                    if item.lvl > 0 {
                        items.push(item);
                    }
                }
                OwnedItemResult::NotFound(img, lvl) => {
                    if last_found_item.is_some() && lvl > 0 {
                        let missing_item = OwnedItem {
                            id: "not_found".into(),
                            lvl,
                            points: 0,
                        };
                        try_items.push((i, missing_item, img.clone()));
                    }
                    if lvl > 0 {
                        owned_missing.push((img, lvl));
                    } else {
                        not_owned_missing.push((img, 0));
                    }
                }
                OwnedItemResult::Invalid => { /* filtered out */ }
            }

            println!("-------");
        }

        inv.update_inventory(MktInventory::from_items(items, data));
    }

    (inv, owned_missing, not_owned_missing)
}

pub fn create_template(item: &Item, img: RgbaImage) {
    for i in (50..=170).step_by(20) {
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
        // bg.save(format!("pics/{}_{}.png", item.id, i)).unwrap();
        let _bg = bg.into_rgb8();
        // item_image_to_template(item, i, &bg);
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

pub fn test_img_hash() {
    let img1 = "tests/test_0x2416cd99f60_193_183.png";
    let img2 = [
        "tests/test_0x2416cda4f20_667_228.png",
        "tests/d_yoshi_70.png",
        "tests/d_yoshi_115.png",
        "tests/d_yoshi_160.png",
        "tests/d_blue_yoshi_70.png",
        "tests/d_blue_yoshi_115.png",
        "tests/d_blue_yoshi_160.png",
        "tests/d_red_yoshi_70.png",
        "tests/d_red_yoshi_115.png",
        "tests/d_red_yoshi_160.png",
        "tests/mkt_driver_single.jpg",
    ];

    println!("img 1: {}", img1);

    for img2 in img2 {
        println!("------");
        println!("img 2: {}", img2);
        let image1 = image::open(img1).unwrap();
        let image2 = image::open(img2).unwrap();

        let image1 = image1.crop_imm(0, 0, 160, 150);
        let image2 = image2.crop_imm(0, 0, 160, 150);

        let h1 = to_image_hash(&image1.into_rgb8());
        let h2 = to_image_hash(&image2.into_rgb8());

        let dist = dist_hash(&h1, &h2);

        println!("Image1 hash: {}", h1);
        println!("Image2 hash: {}", h2);

        println!("Distance: {}", dist);
    }
}

fn to_image_hash(img: &RgbImage) -> String {
    let hasher = HasherConfig::new()
        .preproc_dct()
        .hash_alg(img_hash::HashAlg::DoubleGradient)
        .to_hasher();

    let r = hasher.hash_image(&map::red_channel(img));
    let g = hasher.hash_image(&map::green_channel(img));
    let b = hasher.hash_image(&map::blue_channel(img));

    format!("{}|{}|{}", r.to_base64(), g.to_base64(), b.to_base64())
}

fn dist_hash(h1: &String, h2: &String) -> u32 {
    let dist: Option<_> = try {
        let (r1, g1, b1) = h1.split("|").next_tuple()?;
        let (r2, g2, b2) = h2.split("|").next_tuple()?;

        [
            ImageHash::<Box<[u8]>>::from_base64(r1)
                .ok()?
                .dist(&ImageHash::from_base64(r2).ok()?),
            ImageHash::<Box<[u8]>>::from_base64(g1)
                .ok()?
                .dist(&ImageHash::from_base64(g2).ok()?),
            ImageHash::<Box<[u8]>>::from_base64(b1)
                .ok()?
                .dist(&ImageHash::from_base64(b2).ok()?),
        ]
        .iter()
        .map(|d| d + 1)
        .fold(1, |d1, d2| d1 * d2)
    };
    dist.unwrap_or(u32::MAX)
}

fn save_image_hash(item: &Item, img: &RgbImage) {
    // item x: 10 - 150  y: 20 - 140
    let item_img = imageops::crop_imm(img, 10, 20, 140, 120).to_image();
    let h = to_image_hash(&item_img);
    fs::create_dir_all(format!("templates/{}s", item.i_type)).unwrap();
    fs::write(format!("templates/{}s/{}.txt", item.i_type, item.id), h).unwrap();
}

pub fn save_missing_image_hash(i: usize, img: &RgbImage) {
    fs::create_dir_all("missing").unwrap();
    img.save(format!("missing/missing_item_{}.png", i)).unwrap();
    // item x: 10 - 150  y: 20 - 140
    let item_img = imageops::crop_imm(img, 10, 20, 140, 120).to_image();
    let h = to_image_hash(&item_img);
    fs::write(format!("missing/missing_item_{}.txt", i), h).unwrap();
}
