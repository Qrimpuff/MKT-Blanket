use std::{
    collections::{HashMap, HashSet},
    fs,
};

use image::RgbImage;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub type CourseId = String;
pub type ItemId = String;
pub type ItemLvl = u8;

pub fn course_id_from_name(name: &str) -> CourseId {
    "c_".to_string() + &id_from_name(name)
}

pub fn driver_id_from_name(name: &str) -> ItemId {
    "d_".to_string() + &id_from_name(name)
}

pub fn kart_id_from_name(name: &str) -> ItemId {
    "k_".to_string() + &id_from_name(name)
}

pub fn glider_id_from_name(name: &str) -> ItemId {
    "g_".to_string() + &id_from_name(name)
}

pub fn item_id_from_name(name: &str, i_type: ItemType) -> ItemId {
    match i_type {
        ItemType::Driver => driver_id_from_name(name),
        ItemType::Kart => kart_id_from_name(name),
        ItemType::Glider => glider_id_from_name(name),
    }
}

fn id_from_name(name: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("[^a-z0-9]+").unwrap();
    }
    RE.replace_all(&name.to_lowercase(), "_")
        .trim_matches('_')
        .to_string()
}

#[derive(Debug)]
pub struct Course {
    pub id: CourseId,
    pub name: String,                               // current english name
    pub favorite_items: HashSet<(ItemId, ItemLvl)>, // previous names (for updating/merging)
}
impl Course {
    pub fn new(name: String) -> Self {
        Course {
            id: course_id_from_name(&name),
            name,
            favorite_items: HashSet::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemType {
    Driver,
    Kart,
    Glider,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Rarity {
    Normal,
    Super,
    HighEnd,
}

#[derive(Debug)]
pub struct Item {
    pub id: ItemId,
    pub i_type: ItemType,
    pub name: String, // current english name
    pub rarity: Rarity,
    pub favorite_courses: HashSet<(CourseId, ItemLvl)>,
    pub templates: Vec<RgbImage>, // TODO: used for screenshot import (not sure how yet)
}
impl Item {
    pub fn new(i_type: ItemType, rarity: Rarity, name: String) -> Self {
        Item {
            id: item_id_from_name(&name, i_type),
            i_type,
            name,
            rarity,
            favorite_courses: HashSet::new(),
            templates: vec![],
        }
    }
    // TODO: will remove later
    pub fn with_id_and_template(id: ItemId, i_type: ItemType, template: RgbImage) -> Self {
        Item {
            id: id.clone(),
            i_type,
            name: id,
            rarity: Rarity::Normal,
            favorite_courses: HashSet::new(),
            templates: vec![template],
        }
    }
}

#[derive(Debug)]
pub struct MktDatabase {
    pub courses: HashMap<CourseId, Course>,
    pub drivers: HashMap<ItemId, Item>,
    pub karts: HashMap<ItemId, Item>,
    pub gliders: HashMap<ItemId, Item>,
}
impl MktDatabase {
    pub fn new() -> Self {
        MktDatabase {
            courses: HashMap::new(),
            drivers: HashMap::new(),
            karts: HashMap::new(),
            gliders: HashMap::new(),
        }
    }

    pub fn update_database(&mut self, _new_data: MktDatabase) {
        // TODO: update courses
        // TODO: same course
        // TODO: change or add course

        // TODO: update drivers

        // TODO: update karts

        // TODO: update gliders
    }
}

#[derive(Debug)]
pub struct OwnedItem {
    pub id: ItemId,
    pub lvl: ItemLvl,
    pub points: u16,
}

#[derive(Debug)]
pub struct MktInventory {
    pub drivers: Vec<OwnedItem>,
    pub karts: Vec<OwnedItem>,
    pub gliders: Vec<OwnedItem>,
}
impl MktInventory {
    pub fn new() -> Self {
        MktInventory {
            drivers: Vec::new(),
            karts: Vec::new(),
            gliders: Vec::new(),
        }
    }

    pub fn from_items(items: Vec<OwnedItem>, data: &MktDatabase) -> Self {
        let mut items = items.into_iter().into_group_map_by(|i| {
            data.drivers
                .get(&i.id)
                .or(data.karts.get(&i.id))
                .or(data.gliders.get(&i.id))
                .map(|i| i.i_type)
        });
        MktInventory {
            drivers: items.remove(&Some(ItemType::Driver)).unwrap_or_default(),
            karts: items.remove(&Some(ItemType::Kart)).unwrap_or_default(),
            gliders: items.remove(&Some(ItemType::Glider)).unwrap_or_default(),
        }
    }

    pub fn update_inventory(&mut self, new_inv: MktInventory) {
        let items = [
            (new_inv.drivers, &mut self.drivers),
            (new_inv.karts, &mut self.karts),
            (new_inv.gliders, &mut self.gliders),
        ];

        for (new_inv, inv) in items {
            new_inv.into_iter().for_each(|item| {
                // TODO: add last modified check to merge
                if let Some(f_item) = inv.iter_mut().find(|i_item| i_item.id == item.id) {
                    *f_item = item;
                } else {
                    inv.push(item);
                }
            });
        }
    }
}

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

// TODO: will be removed or transformed later
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
