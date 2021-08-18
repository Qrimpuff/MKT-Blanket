use std::collections::HashMap;

use image::RgbImage;
use itertools::Itertools;

pub type ItemId = String;
pub type ItemLvl = u8;
pub struct I18n {
    pub lang: String,
    pub text: String,
}

pub struct Course {
    pub id: ItemId,
    pub name: String,          // current default name (english)
    pub i18n_names: Vec<I18n>, // names in different languages
    pub aka: Vec<I18n>,        // previous names (for updating/merging)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemType {
    Driver,
    Kart,
    Glider,
}

pub struct Item {
    pub id: ItemId,
    pub i_type: ItemType,
    pub name: String,          // current default name (english)
    pub i18n_names: Vec<I18n>, // names in different languages
    pub aka: Vec<I18n>,        // previous names (for updating/merging)
    pub favorite_courses: Vec<(Course, ItemLvl)>,
    pub templates: Vec<RgbImage>, // TODO: used for screenshot import (not sure how yet)
}
impl Item {
    pub fn with_id_and_template(id: ItemId, i_type: ItemType, template: RgbImage) -> Self {
        Item {
            id: id.clone(),
            i_type,
            name: id,
            i18n_names: vec![],
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
