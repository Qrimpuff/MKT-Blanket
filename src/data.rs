use std::{collections::HashMap, fs};

use image::RgbImage;
use itertools::Itertools;

pub type ItemId = String;
pub type ItemLvl = u8;

pub struct Course {
    pub id: ItemId,
    pub name: String,          // current default name (english)
    pub aka: Vec<String>,        // previous names (for updating/merging)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemType {
    Course,
    Driver,
    Kart,
    Glider,
}

pub struct Item {
    pub id: ItemId,
    pub i_type: ItemType,
    pub name: String,          // current default name (english)
    pub aka: Vec<String>,        // previous names (for updating/merging)
    pub favorite_courses: Vec<(Course, ItemLvl)>,
    pub templates: Vec<RgbImage>, // TODO: used for screenshot import (not sure how yet)
}
impl Item {
    pub fn with_id_and_template(id: ItemId, i_type: ItemType, template: RgbImage) -> Self {
        Item {
            id: id.clone(),
            i_type,
            name: id,
            aka: vec![],
            favorite_courses: vec![],
            templates: vec![template],
        }
    }
}

pub struct MktDatabase {
    pub courses: HashMap<ItemId, Course>,
    pub drivers: HashMap<ItemId, Item>,
    pub karts: HashMap<ItemId, Item>,
    pub gliders: HashMap<ItemId, Item>,
}
impl MktDatabase {
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
