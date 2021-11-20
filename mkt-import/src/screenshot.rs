#![allow(dead_code)]

use mkt_data::*;
use palette::{GetHue, Hsv, IntoColor, Pixel, Srgb};

use std::{cmp::Ordering, collections::HashMap, fs};

use image::{
    imageops::{self, FilterType},
    GrayImage, Luma, RgbImage,
};
use imageproc::{
    contrast,
    distance_transform::Norm,
    drawing, map, morphology,
    rect::Rect,
    template_matching::{self, MatchTemplateMethod},
};
use img_hash::{HasherConfig, ImageHash};
use itertools::Itertools;

const DEBUG: bool = true;
const DEBUG_IMG: bool = false;

const DEFAULT_ITEM_WIDTH: u32 = 160;
const DEFAULT_ITEM_HEIGHT: u32 = 200;
const DEFAULT_ITEM_RATIO: f32 = DEFAULT_ITEM_WIDTH as f32 / DEFAULT_ITEM_HEIGHT as f32;
const ITEM_RATIO_THRESHOLD: f32 = 0.1;

const TEMPLATE_LVL_X: u32 = 125;
const TEMPLATE_LVL_Y: u32 = 125;
const TEMPLATE_LVL_WIDTH: u32 = 32;
const TEMPLATE_LVL_HEIGHT: u32 = 35;
const TEMPLATE_LVL_THRESHOLD: f32 = 0.6;

const TEMPLATE_POINTS_X: u32 = 125;
const TEMPLATE_POINTS_X_OFFSET: u32 = 22;
const TEMPLATE_POINTS_Y: u32 = 155;
const TEMPLATE_POINTS_WIDTH: u32 = 32;
const TEMPLATE_POINTS_HEIGHT: u32 = 35;
const TEMPLATE_POINTS_NUMBERS_COUNT: u32 = 4;
const TEMPLATE_POINTS_THRESHOLD: f32 = 0.6;

const HASH_ITEM_X: u32 = 20;
const HASH_ITEM_Y: u32 = 30;
const HASH_ITEM_WIDTH: u32 = 120;
const HASH_ITEM_HEIGHT: u32 = 100;
pub const HASH_ITEM_THRESHOLD: u32 = 4000;

struct LvlTemplate(ItemLvl, GrayImage);
struct PointsTemplate(ItemPoints, GrayImage);

struct ItemHash(ItemId, String);

static TEMPLATES_LVL: &[(ItemLvl, &[u8])] = &[
    (1, include_bytes!("../templates/levels/1.png")),
    (2, include_bytes!("../templates/levels/2.png")),
    (3, include_bytes!("../templates/levels/3.png")),
    (4, include_bytes!("../templates/levels/4.png")),
    (5, include_bytes!("../templates/levels/5.png")),
    (6, include_bytes!("../templates/levels/6.png")),
    (7, include_bytes!("../templates/levels/7.png")),
];

fn get_lvl_templates() -> Vec<LvlTemplate> {
    let levels_templates: Vec<_> = TEMPLATES_LVL
        .iter()
        .map(|(lvl, bytes)| LvlTemplate(*lvl, image::load_from_memory(bytes).unwrap().into_luma8()))
        .collect();
    levels_templates
}

static TEMPLATES_POINTS: &[(ItemPoints, &[u8])] = &[
    (0, include_bytes!("../templates/points/0.png")),
    (1, include_bytes!("../templates/points/1.png")),
    (2, include_bytes!("../templates/points/2.png")),
    (3, include_bytes!("../templates/points/3.png")),
    (4, include_bytes!("../templates/points/4.png")),
    (5, include_bytes!("../templates/points/5.png")),
    (6, include_bytes!("../templates/points/6.png")),
    (7, include_bytes!("../templates/points/7.png")),
    (8, include_bytes!("../templates/points/8.png")),
    (9, include_bytes!("../templates/points/9.png")),
];

fn get_points_templates() -> Vec<PointsTemplate> {
    let levels_templates: Vec<_> = TEMPLATES_POINTS
        .iter()
        .map(|(points, bytes)| {
            PointsTemplate(
                *points,
                image::load_from_memory(bytes).unwrap().into_luma8(),
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
    fn to_rect(self) -> Rect {
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
    fn area(&self) -> u32 {
        (self.x2 - self.x1) * (self.y2 - self.y1)
    }
    fn swap_x_y(mut self) -> Self {
        std::mem::swap(&mut self.x1, &mut self.y1);
        std::mem::swap(&mut self.x2, &mut self.y2);
        self
    }
}

fn find_item_rows(img: &GrayImage, min_width: u32) -> (Vec<ItemArea>, u32) {
    let (width, height) = img.dimensions();
    let mut item_rows = vec![];
    let mut prev_is_item = None;
    let mut item_line = None;

    let item_width = {
        let mut streaks: HashMap<_, u32> = HashMap::new();
        for (_y, ps) in img.enumerate_rows() {
            let mut current_w_streak = 0;
            for (_x, _y, p) in ps {
                let Luma([luma]) = *p;
                if luma > 0 {
                    current_w_streak += 1;
                } else {
                    if current_w_streak > 0 && current_w_streak >= min_width {
                        *streaks.entry(current_w_streak).or_default() += 1;
                    }
                    current_w_streak = 0;
                }
            }
        }
        if DEBUG {
            println!("{:?}", streaks);
        }
        streaks
            .iter()
            .max_by_key(|(_, v)| *v)
            .map_or(0, |(k, _)| *k)
    };
    if DEBUG {
        dbg!(&item_width);
    }

    let mut item_count = 0;
    let mut is_in_item = false;
    for (y, ps) in img.enumerate_rows() {
        let mut streak_count = 0;
        let mut in_streaks = false;
        let mut is_item_line = false;
        let mut current_w_streak = 0;
        let mut current_b_streak = 0;
        let mut long_current_w_streak = 0;
        for (_x, _y, p) in ps {
            let Luma([luma]) = *p;
            if luma > 0 {
                if long_current_w_streak > 0 {
                    long_current_w_streak += current_b_streak;
                }
                long_current_w_streak += 1;
                current_w_streak += 1;
                current_b_streak = 0;
            } else {
                if current_b_streak > item_width {
                    long_current_w_streak = 0;
                }
                current_b_streak += 1;
                current_w_streak = 0;
            }
            let is_streak = !is_in_item && current_w_streak >= item_width * 9 / 10;
            if in_streaks != is_streak {
                if is_streak {
                    streak_count += 1;
                }
                in_streaks = is_streak;
            }
            if streak_count > 0
                || is_in_item && long_current_w_streak >= item_count * item_width * 9 / 10
            {
                is_item_line = true;
                if is_in_item {
                    break;
                }
            }
        }
        is_in_item = is_item_line;
        if prev_is_item != Some(is_item_line) {
            if is_item_line {
                if DEBUG {
                    dbg!(streak_count);
                }
                item_count = streak_count;
                item_line = Some(y);
            } else if let Some(l) = item_line {
                item_count = 0;
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
                                y1,
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
        prev_is_item = Some(is_item_line);
    }
    if let Some(l) = item_line {
        item_rows.push(ItemArea {
            x1: 0,
            y1: l,
            x2: width,
            y2: height,
        });
    }

    if DEBUG {
        dbg!(item_rows.len());
        println!("{:?},{:?}", item_rows, item_width);
    }
    (item_rows, item_width)
}

fn hsv(ps: [u8; 3]) -> (f32, f32, f32) {
    let rgb = Srgb::from_raw(&ps).into_format();
    let hsv: Hsv = rgb.into_color();
    let h = hsv.hue.to_positive_degrees();
    let s = hsv.saturation;
    let v = hsv.value;
    (h, s, v)
}

fn hue(ps: [u8; 3]) -> Option<palette::RgbHue> {
    let rgb = Srgb::from_raw(&ps).into_format();
    let hsv: Hsv = rgb.into_color();
    hsv.get_hue()
}

fn find_item_areas(i: usize, img: &RgbImage) -> impl Iterator<Item = ItemArea> {
    // used for find_item_rows
    let img = GrayImage::from_raw(
        img.width(),
        img.height(),
        img.pixels()
            .flat_map(|p| {
                let (h, s, v) = hsv(p.0);
                if (190.0..=225.0).contains(&h) && s >= 0.5 && v >= 0.5 || s <= 0.1 && v <= 0.1 {
                    [0]
                } else {
                    [255]
                }
            })
            .collect(),
    )
    .unwrap();

    if DEBUG_IMG {
        img.save(format!("pics/test_mask_{}.png", i)).unwrap();
    }
    let (rows, item_width) = find_item_rows(&img, 50);
    let rows = rows.into_iter();
    let img = &imageops::rotate90(&img);
    let (columns, _item_width) = find_item_rows(img, item_width);
    let columns = columns.into_iter().map(ItemArea::swap_x_y);

    rows.cartesian_product(columns)
        .flat_map(|(c, r)| c.intersect(&r))
        .filter(|a| a.area() > 100)
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
    templates: &[LvlTemplate],
) -> Option<ItemLvl> {
    let lvl_img = imageops::crop_imm(
        img,
        TEMPLATE_LVL_X,
        TEMPLATE_LVL_Y,
        TEMPLATE_LVL_WIDTH,
        TEMPLATE_LVL_HEIGHT,
    )
    .to_image();

    let mut img = GrayImage::from_raw(
        lvl_img.width(),
        lvl_img.height(),
        lvl_img
            .pixels()
            .flat_map(|p| {
                let (h, s, v) = hsv(p.0);
                if (65.0..=95.0).contains(&h) && s >= 0.2 && v >= 0.5 {
                    [255]
                } else {
                    [0]
                }
            })
            .collect(),
    )
    .unwrap();
    morphology::erode_mut(&mut img, Norm::LInf, 1);

    // template testing levels
    let lvl = templates
        .iter()
        .map(|LvlTemplate(l, template)| (*l, template_score(&img, template)))
        .inspect(|i| {
            if DEBUG {
                println!("lvl points: {:#?}", i);
            }
        })
        .filter(|(_, score)| *score < TEMPLATE_LVL_THRESHOLD)
        .min_by(|(_, a), (_, b)| -> Ordering { a.partial_cmp(b).unwrap_or(Ordering::Equal) });

    if DEBUG_IMG {
        img.save(format!(
            "pics/test_{}_{}_lvl_{:?}.png",
            y1,
            x1,
            &lvl.map(|l| l.0)
        ))
        .unwrap();
    }
    if DEBUG {
        println!("best lvl: {:?}", lvl);
    }

    lvl.map(|l| l.0)
}

fn item_points_from_image(
    ItemArea { x1, y1, .. }: ItemArea,
    img: &RgbImage,
    templates: &[PointsTemplate],
) -> Option<ItemPoints> {
    let mut points = None;

    for num in 0..TEMPLATE_POINTS_NUMBERS_COUNT {
        let points_img = imageops::crop_imm(
            img,
            TEMPLATE_POINTS_X - (num * TEMPLATE_POINTS_X_OFFSET),
            TEMPLATE_POINTS_Y,
            TEMPLATE_POINTS_WIDTH,
            TEMPLATE_POINTS_HEIGHT,
        )
        .to_image();

        let mut img = GrayImage::from_raw(
            points_img.width(),
            points_img.height(),
            points_img
                .pixels()
                .flat_map(|p| {
                    let (h, s, v) = hsv(p.0);
                    if (25.0..=50.0).contains(&h) && s >= 0.2 && v >= 0.6 {
                        [255]
                    } else {
                        [0]
                    }
                })
                .collect(),
        )
        .unwrap();
        morphology::erode_mut(&mut img, Norm::LInf, 1);

        // template testing points
        let point = templates
            .iter()
            .map(|PointsTemplate(l, template)| (*l, template_score(&img, template)))
            .inspect(|i| {
                if DEBUG {
                    println!("points points: {:#?}", i);
                }
            })
            .filter(|(_, score)| *score < TEMPLATE_POINTS_THRESHOLD)
            .min_by(|(_, a), (_, b)| -> Ordering { a.partial_cmp(b).unwrap_or(Ordering::Equal) });

        if DEBUG_IMG {
            img.save(format!(
                "pics/test_{}_{}_points_{}_{:?}.png",
                y1,
                x1,
                num,
                &point.map(|l| l.0)
            ))
            .unwrap();
        }
        if DEBUG {
            println!("best points: {:?}", point);
        }

        if let Some(p) = point {
            let p = p.0 * 10_u16.pow(num);
            if p < 2000 {
                points = Some(p + points.unwrap_or(0));
            }
        }
    }

    points
}

fn item_id_from_image(
    ItemArea { x1, y1, .. }: ItemArea,
    img: &RgbImage,
    hashes: &[ItemHash],
) -> (String, Option<ItemId>) {
    let item_img = imageops::crop_imm(
        img,
        HASH_ITEM_X,
        HASH_ITEM_Y,
        HASH_ITEM_WIDTH,
        HASH_ITEM_HEIGHT,
    )
    .to_image();

    let hash = to_image_hash(&item_img);

    // template testing drivers
    let item = hashes
        .iter()
        .map(|ItemHash(i, h)| (i, dist_hash(&hash, h)))
        .filter(|(_, p)| *p < HASH_ITEM_THRESHOLD)
        .inspect(|i| {
            if DEBUG {
                println!("points h: {:#?}", i);
            }
        })
        .min_by_key(|i| i.1);

    if DEBUG_IMG {
        item_img
            .save(format!(
                "pics/test_{}_{}_item_{}.png",
                y1,
                x1,
                item.unwrap_or((&"none".to_string(), 0)).0
            ))
            .unwrap();
        fs::write(
            format!(
                "pics/test_{}_{}_item_{}.txt",
                y1,
                x1,
                item.unwrap_or((&"none".to_string(), 0)).0
            ),
            &hash,
        )
        .unwrap();
    }
    if DEBUG {
        println!("best item: {:?}", item);
    }

    (hash, item.map(|i| i.0.clone()))
}

fn maybe_item_image(img: &RgbImage) -> bool {
    let mut img = map::blue_channel(img);
    contrast::threshold_mut(&mut img, 200);
    let mut pixels = img.pixels().map(|x| x.0[0]).counts();
    let blue_percent = pixels.remove(&255).unwrap_or(0) as f32 / img.pixels().count() as f32;
    if DEBUG_IMG {
        img.save(format!("pics/test_{:?}_blue_item.png", blue_percent))
            .unwrap();
    }
    blue_percent < 0.9
}

#[derive(Debug)]
pub struct OwnedItemResult {
    pub id: Option<ItemId>,
    pub i_type: Option<ItemType>,
    pub lvl: Option<ItemLvl>,
    pub points: Option<ItemPoints>,
    pub hash: String,
    pub img: Option<RgbImage>,
}

fn result_owned_item(
    OwnedItemResult {
        id, lvl, points, ..
    }: OwnedItemResult,
) -> Option<OwnedItem> {
    id.map(|id| OwnedItem::new(id, lvl.unwrap_or(0), points.unwrap_or(0)))
}

fn item_image_to_owned_item(
    area: ItemArea,
    img: &RgbImage,
    lvl_templates: &[LvlTemplate],
    points_templates: &[PointsTemplate],
    item_hashes: &[ItemHash],
) -> Option<OwnedItemResult> {
    if DEBUG {
        println!("area: {:?}", area);
    }
    let lvl = item_level_from_image(area, img, lvl_templates);
    let points = item_points_from_image(area, img, points_templates);
    let (hash, id) = item_id_from_image(area, img, item_hashes);
    if id.is_some() {
        Some(OwnedItemResult {
            i_type: item_type_from_id(id.as_ref().unwrap()),
            id,
            lvl,
            points,
            hash,
            img: None,
        })
    } else if maybe_item_image(img) {
        Some(OwnedItemResult {
            id: None,
            i_type: None,
            lvl,
            points,
            hash,
            img: Some(img.clone()),
        })
    } else {
        None
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

pub fn image_bytes_to_inventory(
    bytes: Vec<u8>,
    data: &MktData,
    hashes: Option<&MktItemHashes>,
) -> (MktInventory, MktItemHashes) {
    images_bytes_to_inventory(vec![bytes], data, hashes)
}
pub fn images_bytes_to_inventory(
    bytes: Vec<Vec<u8>>,
    data: &MktData,
    hashes: Option<&MktItemHashes>,
) -> (MktInventory, MktItemHashes) {
    let list = bytes
        .into_iter()
        .map(|bytes| image::load_from_memory(&bytes).unwrap().into_rgb8())
        .collect();
    screenshots_to_inventory(list, data, hashes)
}

pub fn images_bytes_to_bootstrap_hashes(
    bytes: Vec<Vec<u8>>,
    i_type: ItemType,
    data: &MktData,
) -> Result<MktItemHashes, BootstrapError> {
    let list = bytes
        .into_iter()
        .map(|bytes| image::load_from_memory(&bytes).unwrap().into_rgb8())
        .collect();
    screenshots_to_bootstrap_hashes(list, i_type, data)
}

#[derive(Debug)]
pub enum BootstrapError {
    WrongLength(usize, usize),
    MissingId,
}

pub fn screenshots_to_bootstrap_hashes(
    screenshots: Vec<RgbImage>,
    i_type: ItemType,
    data: &MktData,
) -> Result<MktItemHashes, BootstrapError> {
    let mut id_list = match i_type {
        ItemType::Driver => &data.drivers,
        ItemType::Kart => &data.karts,
        ItemType::Glider => &data.gliders,
    }
    .values()
    .map(|i| (i.sort, i.id.clone()))
    .collect_vec();
    id_list.sort();
    let id_list = id_list.into_iter().map(|(_, id)| id).collect_vec();

    let items = screenshots_to_owned_items(screenshots, None);

    // remove duplicate rows
    let mut new_hashes: Vec<String> = vec![];
    dbg!(&items.len());
    let mut items = items
        .into_iter()
        .chunks(4)
        .into_iter()
        .flat_map(|c| {
            let mut chunk = c.collect_vec();
            if DEBUG {
                chunk.iter_mut().for_each(|i| i.img = None);
                println!("chunk {:#?}", chunk);
            }
            if chunk.iter().all(|i| {
                println!("item {:#?}", i);
                new_hashes
                    .iter()
                    .any(|h| dist_hash(&i.hash, h) < HASH_ITEM_THRESHOLD)
            }) {
                if DEBUG {
                    println!("bad row x{}", chunk.len());
                }
                None
            } else {
                chunk.iter().for_each(|i| new_hashes.push(i.hash.clone()));
                Some(chunk.into_iter())
            }
        })
        .flatten()
        .collect_vec();

    // verify the length of the list
    if items.len() != id_list.len() {
        return Err(BootstrapError::WrongLength(items.len(), id_list.len()));
    }

    // set first and last item
    let first_id = id_list.get(0).unwrap().to_string();
    let last_id = id_list.last().unwrap().to_string();
    items.get_mut(0).unwrap().id = Some(first_id);
    items.last_mut().unwrap().id = Some(last_id);
    items.iter_mut().for_each(|i| i.i_type = Some(i_type));

    // fill everything in between
    deduce_missing_owned_items(&mut items, data);

    if DEBUG {
        items.iter_mut().for_each(|i| i.img = None);
        println!("{:#?}", items);
    }

    let hashes: Option<_> = items
        .iter()
        .map(|i| try { (i.id.as_ref()?.clone(), i.hash.clone()) })
        .collect();
    hashes.ok_or(BootstrapError::MissingId)
}

pub fn screenshots_to_owned_items(
    screenshots: Vec<RgbImage>,
    hashes: Option<MktItemHashes>,
) -> Vec<OwnedItemResult> {
    let lvl_templates = get_lvl_templates();
    let points_templates = get_points_templates();
    let item_hashes = hashes
        .into_iter()
        .flat_map(|h| h.hashes.into_iter())
        .flat_map(|(id, hashes)| hashes.into_iter().map(move |h| ItemHash(id.clone(), h)))
        .collect_vec();

    let mut owned_items = vec![];

    for (i, screenshot) in screenshots.into_iter().enumerate() {
        let mut debug_img = DEBUG_IMG.then(|| screenshot.clone());

        let mut s_owned_items = vec![];
        let mut i_type = None;

        // identify square
        let item_areas = find_item_areas(i, &screenshot);
        for (i, (area, item_result)) in item_areas
            .map(|area| (area, item_area_to_image(area, &screenshot)))
            .map(|(area, img)| {
                (
                    area,
                    item_image_to_owned_item(
                        area,
                        &img,
                        &lvl_templates,
                        &points_templates,
                        &item_hashes,
                    ),
                )
            })
            .filter(|(_, item)| item.is_some())
            .enumerate()
        {
            if DEBUG_IMG {
                if let Some(debug_img) = debug_img.as_mut() {
                    drawing::draw_filled_rect_mut(
                        debug_img,
                        area.to_rect(),
                        image::Rgb([255, 0, 0]),
                    );
                }
            }
            if DEBUG {
                println!("{} - x:{} y:{}", i, area.x1, area.y1);
            }
            if let Some(item_result) = item_result {
                // assume one type per screenshot
                if item_result.i_type.is_some() {
                    i_type = item_result.i_type;
                }
                s_owned_items.push(item_result);
            }
            if DEBUG {
                println!("-------");
            }
        }

        // give a type to unknown items
        s_owned_items
            .iter_mut()
            .filter(|i| i.i_type.is_none())
            .for_each(|i| i.i_type = i_type);
        owned_items.extend(s_owned_items);

        if DEBUG_IMG {
            if let Some(debug_img) = debug_img {
                debug_img
                    .save(format!("pics/find_item_areas_{}.png", i))
                    .unwrap();
            }
        }
    }

    owned_items
}

pub fn deduce_missing_owned_items(owned_items: &mut Vec<OwnedItemResult>, data: &MktData) {
    // try to identify items in order
    let mut item_offset = 0;
    let mut last_found_type: Option<ItemType> = None;
    let mut last_found_item: Option<(usize, &OwnedItemResult)> = None;
    let mut try_items: Vec<(usize, &mut OwnedItemResult)> = vec![];
    let mut potential_item_ids = None;
    // the end of the list
    let the_end_id = "<the_end>";
    for (i, item_result) in owned_items
        .iter_mut()
        .chain(Some(&mut OwnedItemResult {
            id: Some(the_end_id.into()),
            i_type: None,
            lvl: None,
            points: None,
            hash: String::new(),
            img: None,
        }))
        .enumerate()
    {
        // only load the potential ids when the type change, if ever
        if item_result.i_type != last_found_type {
            if let Some(i_type) = item_result.i_type {
                let potential_items = match i_type {
                    ItemType::Driver => &data.drivers,
                    ItemType::Kart => &data.karts,
                    ItemType::Glider => &data.gliders,
                };
                potential_item_ids = Some(
                    potential_items
                        .values()
                        .sorted_by_key(|i| i.sort.map(|x| x as i32).unwrap_or(-1))
                        .map(|i| i.id.clone())
                        .chain(Some(the_end_id.into()))
                        .collect_vec(),
                );
            } else if item_result.id.as_deref() != Some(the_end_id) {
                potential_item_ids = None;
            }
            last_found_type = item_result.i_type;
            // reset expectation
            try_items = vec![];
        }

        // found item
        if let Some(id) = &item_result.id {
            if let Some(potential_item_ids) = &potential_item_ids {
                // check expectation
                if let Some(last_found_item) = last_found_item {
                    if !try_items.is_empty() {
                        // check if expected item matches
                        if let Some(expected_item_id) =
                            potential_item_ids.get(i - last_found_item.0 + item_offset)
                        {
                            if expected_item_id == id {
                                // all the items in between are know now
                                for mut t_item in try_items {
                                    let item_id = &potential_item_ids
                                        [t_item.0 - last_found_item.0 + item_offset];
                                    t_item.1.id = Some(item_id.clone());
                                }
                            }
                        }
                        try_items = vec![];
                    }
                }
                item_offset = potential_item_ids
                    .iter()
                    .find_position(|p_id| *p_id == id)
                    .map(|i| i.0)
                    .unwrap_or(0);
                last_found_item = Some((i, item_result));
            } else {
                item_offset = 0;
                last_found_item = None;
            }
        } else {
            // unknown item
            try_items.push((i, item_result));
        }
    }
}

pub fn screenshots_to_inventory(
    screenshots: Vec<RgbImage>,
    data: &MktData,
    hashes: Option<&MktItemHashes>,
) -> (MktInventory, MktItemHashes) {
    let mut inv = MktInventory::new();

    let mut data_hashes = data.hashes();
    if let Some(hashes) = hashes.cloned() {
        data_hashes.merge(hashes);
    }
    let mut items = screenshots_to_owned_items(screenshots, Some(data_hashes));
    deduce_missing_owned_items(&mut items, data);

    let hashes = items
        .iter()
        .filter(|i| i.id.is_some() && i.img.is_some())
        .map(|i| (i.id.as_ref().expect("is some").clone(), i.hash.clone()))
        .collect();
    let items = items
        .into_iter()
        .filter(|i| i.id.is_some() && i.lvl.is_some() && i.points.is_some())
        .flat_map(result_owned_item)
        .collect();
    inv.update_inventory(MktInventory::from_items(items, data));

    (inv, hashes)
}

pub fn _test_img_hash() {
    let imgs = (1..=8)
        .map(|i| format!("tmp/yoshi_{}.png", i))
        .collect_vec();

    for img1 in &imgs {
        println!("---------------------------");
        println!("img 1: {}", img1);

        for img2 in &imgs {
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
        println!("---------------------------");
    }
}

fn to_image_hash(img: &RgbImage) -> String {
    let hasher = HasherConfig::new()
        .preproc_dct()
        .hash_alg(img_hash::HashAlg::DoubleGradient)
        .to_hasher();

    let mut hashes = vec![];

    let red = map::red_channel(img);
    let green = map::green_channel(img);
    let blue = map::blue_channel(img);

    let rh = hasher.hash_image(&red);
    let gh = hasher.hash_image(&green);
    let bh = hasher.hash_image(&blue);
    hashes.push(rh.to_base64());
    hashes.push(gh.to_base64());
    hashes.push(bh.to_base64());

    let red = hue_gray_image(img, 0.0);
    let green = hue_gray_image(img, 120.0);
    let blue = hue_gray_image(img, 240.0);
    let sat = sat_gray_image(img);

    let rhh = hasher.hash_image(&red);
    let ghh = hasher.hash_image(&green);
    let bhh = hasher.hash_image(&blue);
    let sh = hasher.hash_image(&sat);
    hashes.push(rhh.to_base64());
    hashes.push(ghh.to_base64());
    hashes.push(bhh.to_base64());
    hashes.push(sh.to_base64());

    let hash = hashes.join("|");

    if DEBUG {
        println!("{}", hash);
    }

    hash
}

pub fn dist_hash(h1: &str, h2: &str) -> u32 {
    let dist: Option<_> = try {
        let h1 = h1.split('|').collect_vec();
        let h2 = h2.split('|').collect_vec();

        let a_dist_h = h1
            .iter()
            .zip(h2.iter())
            .filter_map(|(h1, h2)| try {
                ImageHash::<Box<[u8]>>::from_base64(h1)
                    .ok()?
                    .dist(&ImageHash::from_base64(h2).ok()?)
            })
            .collect_vec();
        if h1.len() != a_dist_h.len() || h2.len() != a_dist_h.len() {
            None?;
        }
        let dist_h = a_dist_h
            .iter()
            .map(|d| if *d <= 2 { 1 } else { *d })
            .product();

        if DEBUG {
            println!("{:?} = {}", a_dist_h, dist_h);
        }
        dist_h
    };
    dist.unwrap_or(u32::MAX)
}

pub fn _test_gray_image() {
    let img = image::open("tmp/inv_ipad.jpg").unwrap().to_rgb8();

    let red = map::red_channel(&img);
    red.save("pics/gray_1_red.png").unwrap();

    let green = map::green_channel(&img);
    green.save("pics/gray_2_green.png").unwrap();

    let blue = map::blue_channel(&img);
    blue.save("pics/gray_3_blue.png").unwrap();

    let red_hue = hue_gray_image(&img, 0.0);
    red_hue.save("pics/gray_4_red_hue.png").unwrap();

    let green_hue = hue_gray_image(&img, 120.0);
    green_hue.save("pics/gray_5_green_hue.png").unwrap();

    let blue_hue = hue_gray_image(&img, 240.0);
    blue_hue.save("pics/gray_6_blue_hue.png").unwrap();

    let sat = sat_gray_image(&img);
    sat.save("pics/gray_7_sat.png").unwrap();
}

fn hue_gray_image(img: &RgbImage, base_hue: f32) -> GrayImage {
    GrayImage::from_raw(
        img.width(),
        img.height(),
        img.pixels()
            .flat_map(|p| {
                let h = hue(p.0);
                let diff = if let Some(h) = h {
                    base_hue - h
                } else {
                    180.0.into()
                };
                [((1.0 - diff.to_degrees().abs().min(90.0) / 90.0) * 255.0) as u8]
            })
            .collect(),
    )
    .unwrap()
}

fn sat_gray_image(img: &RgbImage) -> GrayImage {
    GrayImage::from_raw(
        img.width(),
        img.height(),
        img.pixels()
            .flat_map(|p| {
                let s = hsv(p.0).1;
                [((1.0 - s) * 255.0) as u8]
            })
            .collect(),
    )
    .unwrap()
}
