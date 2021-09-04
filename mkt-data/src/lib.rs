use std::{collections::HashSet, convert::TryFrom, error::Error, fmt::Display, fs, mem};

use chrono::{DateTime, Utc};
use hashlink::LinkedHashMap;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemRequirement {
    pub id: ItemId,
    pub lvl: ItemLvl,
}

impl From<(ItemId, ItemLvl)> for ItemRequirement {
    fn from((id, lvl): (ItemId, ItemLvl)) -> Self {
        ItemRequirement { id, lvl }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CourseAvailability {
    pub id: CourseId,
    pub lvl: ItemLvl,
}

impl From<(CourseId, ItemLvl)> for CourseAvailability {
    fn from((id, lvl): (CourseId, ItemLvl)) -> Self {
        CourseAvailability { id, lvl }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    pub id: CourseId,
    pub name: String,                             // current english name
    pub favorite_items: HashSet<ItemRequirement>, // previous names (for updating/merging)
    pub last_changed: Option<DateTime<Utc>>,
}
impl Course {
    pub fn new(name: String) -> Self {
        Course {
            id: course_id_from_name(&name),
            name,
            favorite_items: HashSet::new(),
            last_changed: Some(Utc::now()),
        }
    }

    pub fn merge(
        &mut self,
        Course {
            id,
            name,
            favorite_items,
            last_changed,
        }: Course,
    ) {
        let mut changed = false;

        if !id.is_empty() && self.id != id {
            self.id = id;
            changed = true;
        }
        if !name.is_empty() && self.name != name {
            self.name = name;
            changed = true;
        }
        if !favorite_items.is_empty() && self.favorite_items != favorite_items {
            self.favorite_items = favorite_items;
            changed = true;
        }

        if changed {
            self.last_changed = last_changed;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Driver,
    Kart,
    Glider,
}
impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ItemType::Driver => f.write_str("driver"),
            ItemType::Kart => f.write_str("kart"),
            ItemType::Glider => f.write_str("glider"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Normal,
    Super,
    HighEnd,
}
impl TryFrom<&str> for Rarity {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "normal" => Ok(Rarity::Normal),
            "super" => Ok(Rarity::Super),
            "high-end" => Ok(Rarity::HighEnd),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<u32>,
    pub id: ItemId,
    pub i_type: ItemType,
    pub name: String, // current english name
    pub rarity: Rarity,
    pub favorite_courses: HashSet<CourseAvailability>,
    pub hashes: Vec<String>, // used for screenshot import
    pub last_changed: Option<DateTime<Utc>>,
}
impl Item {
    pub fn new(i_type: ItemType, rarity: Rarity, name: String, sort: Option<u32>) -> Self {
        Item {
            sort,
            id: item_id_from_name(&name, i_type),
            i_type,
            name,
            rarity,
            favorite_courses: HashSet::new(),
            hashes: vec![],
            last_changed: Some(Utc::now()),
        }
    }

    pub fn merge(
        &mut self,
        Item {
            sort,
            id,
            i_type,
            name,
            rarity,
            favorite_courses,
            hashes,
            last_changed,
        }: Item,
    ) {
        let mut changed = false;

        if sort.is_some() && self.sort != sort {
            self.sort = sort;
            changed = true;
        }
        if !id.is_empty() && self.id != id {
            self.id = id;
            changed = true;
        }
        if self.i_type != i_type {
            self.i_type = i_type;
            changed = true;
        }
        if !name.is_empty() && self.name != name {
            self.name = name;
            changed = true;
        }
        if self.rarity != rarity {
            self.rarity = rarity;
            changed = true;
        }
        if !favorite_courses.is_empty() && self.favorite_courses != favorite_courses {
            self.favorite_courses = favorite_courses;
            changed = true;
        }
        if !hashes.is_empty() && self.hashes != hashes {
            self.hashes = hashes;
            changed = true;
        }

        if changed {
            self.last_changed = last_changed;
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MktDatabase {
    pub courses: LinkedHashMap<CourseId, Course>,
    pub drivers: LinkedHashMap<ItemId, Item>,
    pub karts: LinkedHashMap<ItemId, Item>,
    pub gliders: LinkedHashMap<ItemId, Item>,
}
impl MktDatabase {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load(file_name: &str) -> Result<MktDatabase, Box<dyn Error>> {
        let json = fs::read_to_string(file_name)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn save(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_name, json)?;
        Ok(())
    }

    pub fn load_hashes(&mut self) -> Result<(), Box<dyn Error>> {
        let types = [
            ("drivers", &mut self.drivers),
            ("karts", &mut self.karts),
            ("gliders", &mut self.gliders),
        ];
        for (p, list) in types {
            for file in fs::read_dir(format!("templates/{}", p))? {
                let file = file?.path();
                let id = file.file_stem().unwrap().to_str().unwrap();
                let hashes = fs::read_to_string(&file)?;
                if let Some(mut item) = list.get_mut(id) {
                    item.hashes = hashes
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect_vec();
                }
            }
        }
        Ok(())
    }

    pub fn merge(&mut self, mut new_data: MktDatabase) {
        // courses
        for (id, course) in &mut self.courses {
            if let Some(new_course) = new_data.courses.remove(id) {
                course.merge(new_course);
            }
        }
        self.courses.extend(new_data.courses);

        // drivers
        for (id, driver) in &mut self.drivers {
            if let Some(new_driver) = new_data.drivers.remove(id) {
                driver.merge(new_driver);
            }
        }
        self.drivers.extend(new_data.drivers);

        // karts
        for (id, kart) in &mut self.karts {
            if let Some(new_kart) = new_data.karts.remove(id) {
                kart.merge(new_kart);
            }
        }
        self.karts.extend(new_data.karts);

        // gliders
        for (id, glider) in &mut self.gliders {
            if let Some(new_glider) = new_data.gliders.remove(id) {
                glider.merge(new_glider);
            }
        }
        self.gliders.extend(new_data.gliders);

        // sort by items
        let mut swap = Default::default();
        mem::swap(&mut swap, &mut self.drivers);
        self.drivers
            .extend(swap.into_iter().sorted_by_key(|(_, i)| i.sort));

        let mut swap = Default::default();
        mem::swap(&mut swap, &mut self.karts);
        self.karts
            .extend(swap.into_iter().sorted_by_key(|(_, i)| i.sort));

        let mut swap = Default::default();
        mem::swap(&mut swap, &mut self.gliders);
        self.gliders
            .extend(swap.into_iter().sorted_by_key(|(_, i)| i.sort));
    }
}

#[derive(Debug, Clone)]
pub struct OwnedItem {
    pub id: ItemId,
    pub lvl: ItemLvl,
    pub points: u16,
}

#[derive(Debug, Default)]
pub struct MktInventory {
    pub drivers: Vec<OwnedItem>,
    pub karts: Vec<OwnedItem>,
    pub gliders: Vec<OwnedItem>,
}
impl MktInventory {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_items(items: Vec<OwnedItem>, data: &MktDatabase) -> Self {
        let mut items = items.into_iter().into_group_map_by(|i| {
            data.drivers
                .get(&i.id)
                .or_else(|| data.karts.get(&i.id))
                .or_else(|| data.gliders.get(&i.id))
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
