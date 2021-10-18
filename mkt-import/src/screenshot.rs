#![allow(dead_code)]

use mkt_data::*;

use std::{cmp::Ordering, collections::HashMap, fs};

use image::{
    imageops::{self, FilterType},
    GenericImage, GrayImage, Luma, RgbImage,
};
use imageproc::{
    contrast, drawing, map,
    rect::Rect,
    template_matching::{self, MatchTemplateMethod},
};
use img_hash::{HasherConfig, ImageHash};
use itertools::Itertools;

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
const HASH_ITEM_THRESHOLD: u32 = 500;

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
        println!("{:?}", streaks);
        streaks
            .iter()
            .max_by_key(|(_, v)| *v)
            .map_or(0, |(k, _)| *k)
    };
    dbg!(&item_width);

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
                item_count = dbg!(streak_count);
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

    dbg!(item_rows.len());
    dbg!((item_rows, item_width))
}

fn find_item_areas(i: usize, img: &RgbImage) -> impl Iterator<Item = ItemArea> {
    // used for find_item_rows
    let img = GrayImage::from_raw(
        img.width(),
        img.height(),
        img.pixels()
            .flat_map(|p| {
                let g = if matches!(p.0, [0..=100, 50..=210, 130..=255]) {
                    0
                } else {
                    p.0[0] | p.0[1] | p.0[2]
                };
                if g > 10 {
                    [255]
                } else {
                    [0]
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

    let img = GrayImage::from_raw(
        lvl_img.width(),
        lvl_img.height(),
        lvl_img
            .pixels()
            .flat_map(|p| {
                if matches!(p.0, [100..=230, 160..=255, 0..=120]) {
                    [255]
                } else {
                    [0]
                }
            })
            .collect(),
    )
    .unwrap();

    // template testing levels
    let lvl = templates
        .iter()
        .map(|LvlTemplate(l, template)| (*l, template_score(&img, template)))
        .inspect(|i| {
            println!("lvl points: {:#?}", i);
        })
        .filter(|(_, score)| *score < TEMPLATE_LVL_THRESHOLD)
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

        let img = GrayImage::from_raw(
            points_img.width(),
            points_img.height(),
            points_img
                .pixels()
                .flat_map(|p| {
                    if matches!(p.0, [180..=255, 50..=220, 0..=100])
                        || p.0[0] > 150 && p.0[1] > 150 && p.0[2] > 150
                    {
                        [255]
                    } else {
                        [0]
                    }
                })
                .collect(),
        )
        .unwrap();

        // template testing points
        let point = templates
            .iter()
            .map(|PointsTemplate(l, template)| (*l, template_score(&img, template)))
            .inspect(|i| {
                println!("points points: {:#?}", i);
            })
            .filter(|(_, score)| *score < TEMPLATE_POINTS_THRESHOLD)
            .min_by(|(_, a), (_, b)| -> Ordering { a.partial_cmp(b).unwrap_or(Ordering::Equal) });

        if DEBUG_IMG {
            img.save(format!(
                "pics/test_{}_{}_{:?}_points_{}.png",
                y1,
                x1,
                &point.map(|l| l.0),
                num
            ))
            .unwrap();
        }
        println!("best points: {:?}", point);

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

#[derive(Debug)]
pub struct OwnedItemResult {
    pub id: Option<ItemId>,
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
    println!("area: {:?}", area);
    let lvl = item_level_from_image(area, img, lvl_templates);
    let points = item_points_from_image(area, img, points_templates);
    let (hash, id) = item_id_from_image(area, img, item_hashes);
    if id.is_some() {
        Some(OwnedItemResult {
            id,
            lvl,
            points,
            hash,
            img: None,
        })
    } else if maybe_item_image(img) {
        Some(OwnedItemResult {
            id: None,
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
    let id_list = match i_type {
        ItemType::Driver => &data.drivers,
        ItemType::Kart => &data.karts,
        ItemType::Glider => &data.gliders,
    }
    .keys()
    .collect_vec();

    let combine = vec![combine_screenshots(screenshots)];
    let items = screenshots_to_owned_items(combine, None);

    // remove duplicate rows
    let mut new_hashes: Vec<String> = vec![];
    let mut items = items
        .into_iter()
        .chunks(4)
        .into_iter()
        .flat_map(|c| {
            let chunk = c.collect_vec();
            if chunk.iter().all(|i| {
                new_hashes
                    .iter()
                    .any(|h| dist_hash(&i.hash, h) < HASH_ITEM_THRESHOLD)
            }) {
                println!("bad row x{}", chunk.len());
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

    // fill everything in between
    deduce_missing_owned_items(&mut items, data);

    items.iter_mut().for_each(|i| i.img = None);
    println!("{:#?}", items);

    let hashes: Option<_> = items
        .iter()
        .map(|i| try { (i.id.as_ref()?.clone(), i.hash.clone()) })
        .collect();
    hashes.ok_or(BootstrapError::MissingId)
}

pub fn combine_screenshots(screenshots: Vec<RgbImage>) -> RgbImage {
    let max_w = screenshots
        .iter()
        .map(|i| i.width())
        .max()
        .unwrap_or_default();
    let max_h = screenshots.iter().map(|i| i.height()).sum();
    let mut out = RgbImage::new(max_w, max_h);
    let mut y = 0;
    for s in screenshots {
        let s = imageops::resize(&s, max_w, s.height(), FilterType::Gaussian);
        out.copy_from(&s, 0, y).unwrap();
        y += s.height();
    }
    if DEBUG_IMG {
        out.save("pics/big_out.png").unwrap();
    }
    out
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
            println!("{} - x:{} y:{}", i, area.x1, area.y1);
            owned_items.push(item_result.expect("is some"));
            println!("-------");
        }
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
            lvl: None,
            points: None,
            hash: String::new(),
            img: None,
        }))
        .enumerate()
    {
        // found item
        if let Some(id) = &item_result.id {
            if let Some(i_type) = item_type_from_id(id).or_else(|| {
                if id == the_end_id {
                    last_found_type
                } else {
                    None
                }
            }) {
                // only load the potential ids when the type change, if ever
                if Some(i_type) != last_found_type {
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
                    last_found_type = Some(i_type);
                    // reset expectation
                    try_items = vec![];
                } else if let Some(potential_item_ids) = &potential_item_ids {
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
                }
                if let Some(potential_item_ids) = &potential_item_ids {
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
        .filter(|i| i.id.is_some() && i.lvl.is_some())
        .flat_map(result_owned_item)
        .collect();
    inv.update_inventory(MktInventory::from_items(items, data));

    (inv, hashes)
}

pub fn _test_img_hash() {
    let imgs = (1..=8)
        .map(|i| format!("tests/yoshi ({}).png", i))
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

    let red = map::red_channel(img);
    let green = map::green_channel(img);
    let blue = map::blue_channel(img);

    let rh = hasher.hash_image(&red);
    let gh = hasher.hash_image(&green);
    let bh = hasher.hash_image(&blue);

    let avg_r = hash_average_luma(&red);
    let avg_g = hash_average_luma(&green);
    let avg_b = hash_average_luma(&blue);

    let l_rg = color_angle(avg_r, avg_g);
    let l_rb = color_angle(avg_r, avg_b);

    format!(
        "{}|{}|{}|{}|{}",
        rh.to_base64(),
        gh.to_base64(),
        bh.to_base64(),
        color_angle_to_hash(l_rg),
        color_angle_to_hash(l_rb),
    )
}

fn dist_hash(h1: &str, h2: &str) -> u32 {
    let dist: Option<_> = try {
        let (r1, g1, b1, l_rg1, l_rb1) = h1.split('|').next_tuple()?;
        let (r2, g2, b2, l_rg2, l_rb2) = h2.split('|').next_tuple()?;

        let dist_h = [
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
        .product::<u32>() as f64;

        let dist_d = dist_angle(
            (hash_to_color_angle(l_rg1), hash_to_color_angle(l_rb1)),
            (hash_to_color_angle(l_rg2), hash_to_color_angle(l_rb2)),
        )
        .powf(2.0)
            + 1.0;

        // println!("{}, {} = {}", dist_h, dist_d, (dist_h * dist_d) as u32);
        (dist_h * dist_d) as u32
    };
    dist.unwrap_or(u32::MAX)
}

fn hash_average_luma(img: &GrayImage) -> u8 {
    average_luma(img, |i| {
        (i as f64) < img.len() as f64 * 0.2 || (i as f64) > img.len() as f64 * 0.8
    })
}

fn average_luma<P>(img: &GrayImage, predicate: P) -> u8
where
    P: Fn(usize) -> bool,
{
    average(img.pixels().map(|p| p.0[0] as usize), predicate)
}

fn average<I, O, P>(iter: I, predicate: P) -> u8
where
    I: IntoIterator<Item = O>,
    O: Ord,
    usize: std::iter::Sum<O>,
    P: Fn(usize) -> bool,
{
    let mut p = iter.into_iter().collect_vec();
    let len = p.len();
    p.sort_unstable();
    let sum: usize = p
        .into_iter()
        .enumerate()
        .filter(|(i, _)| predicate(*i))
        .map(|(_, p)| p)
        .sum();
    (sum / len) as u8
}
fn color_angle_to_hash(a: f64) -> String {
    let h = (a * 100.0) as u16;
    format!("{:x}", h)
}
fn hash_to_color_angle(s: &str) -> f64 {
    let a = u16::from_str_radix(s, 16).unwrap_or_default();
    a as f64 / 100.0
}

fn color_angle(color_1: u8, color_2: u8) -> f64 {
    ((color_1 + 1) as f64 / (color_2 + 1) as f64)
        .atan()
        .to_degrees()
}

fn dist_angle(
    (latitude_degrees_1, longitude_degrees_1): (f64, f64),
    (latitude_degrees_2, longitude_degrees_2): (f64, f64),
) -> f64 {
    let radius = 100.0_f64;

    let latitude_1 = latitude_degrees_1.to_radians();
    let latitude_2 = latitude_degrees_2.to_radians();

    let delta_latitude = (latitude_degrees_1 - latitude_degrees_2).to_radians();
    let delta_longitude = (longitude_degrees_1 - longitude_degrees_2).to_radians();

    let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
        + latitude_1.cos() * latitude_2.cos() * (delta_longitude / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    radius * central_angle
}

fn save_image_hash(item: &Item, img: &RgbImage) {
    let item_img = imageops::crop_imm(
        img,
        HASH_ITEM_X,
        HASH_ITEM_Y,
        HASH_ITEM_WIDTH,
        HASH_ITEM_HEIGHT,
    )
    .to_image();
    let h = to_image_hash(&item_img);
    fs::create_dir_all(format!("templates/{}s", item.i_type)).unwrap();
    fs::write(format!("templates/{}s/{}.txt", item.i_type, item.id), h).unwrap();
}
