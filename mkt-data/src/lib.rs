use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryFrom,
    error::Error,
    fmt::Display,
    fs,
    iter::FromIterator,
};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};

pub type CourseId = String;
pub type ItemId = String;
pub type ItemLvl = u8;
pub type ItemPoints = u16;
pub type ItemHash = String;

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

pub fn item_type_from_id(id: &str) -> Option<ItemType> {
    if id.starts_with("d_") {
        Some(ItemType::Driver)
    } else if id.starts_with("k_") {
        Some(ItemType::Kart)
    } else if id.starts_with("g_") {
        Some(ItemType::Glider)
    } else {
        None
    }
}

pub fn course_type_from_id(id: &str) -> CourseType {
    course_parts_from_id(id).2
}

pub fn course_generation_from_id(id: &str) -> CourseGeneration {
    course_parts_from_id(id).0
}

pub fn course_parts_from_id(id: &str) -> (CourseGeneration, String, CourseType) {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^c_((rmx|snes|n64|gba|gcn|ds|wii|3ds)_)?(.+?(_|\d|$))(r|t|r_t)?$")
                .unwrap();
    }
    let (g, n, t) = RE.captures(id).map_or(("", "", ""), |c| {
        (
            c.get(2).map_or("", |m| m.as_str()),
            c.get(3).map_or("", |m| m.as_str()),
            c.get(5).map_or("", |m| m.as_str()),
        )
    });
    (g.into(), n.trim_end_matches('_').into(), t.into())
}

fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Ord + Serialize,
    V: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

fn ordered_set<S, V>(value: &HashSet<V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Ord + Serialize,
{
    let mut ordered: Vec<_> = value.iter().collect();
    ordered.sort();
    ordered.serialize(serializer)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemRequirement {
    pub id: ItemId,
    pub lvl: ItemLvl,
}

impl From<(ItemId, ItemLvl)> for ItemRequirement {
    fn from((id, lvl): (ItemId, ItemLvl)) -> Self {
        ItemRequirement { id, lvl }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CourseAvailability {
    pub id: CourseId,
    pub lvl: ItemLvl,
}

impl From<(CourseId, ItemLvl)> for CourseAvailability {
    fn from((id, lvl): (CourseId, ItemLvl)) -> Self {
        CourseAvailability { id, lvl }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CourseType {
    Normal,
    Reverse,
    Trick,
    ReverseTrick,
}

impl From<&str> for CourseType {
    fn from(t: &str) -> Self {
        match t {
            "r" => CourseType::Reverse,
            "t" => CourseType::Trick,
            "r_t" => CourseType::ReverseTrick,
            _ => CourseType::Normal,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CourseGeneration {
    New,
    Remix,
    SNES,
    N64,
    GBA,
    GCN,
    DS,
    Wii,
    _3DS,
}

impl From<&str> for CourseGeneration {
    fn from(g: &str) -> Self {
        match g {
            "rmx" => CourseGeneration::Remix,
            "snes" => CourseGeneration::SNES,
            "n64" => CourseGeneration::N64,
            "gba" => CourseGeneration::GBA,
            "gcn" => CourseGeneration::GCN,
            "ds" => CourseGeneration::DS,
            "wii" => CourseGeneration::Wii,
            "3ds" => CourseGeneration::_3DS,
            _ => CourseGeneration::New,
        }
    }
}

impl ToString for CourseGeneration {
    fn to_string(&self) -> String {
        match self {
            CourseGeneration::New => "New Courses".into(),
            CourseGeneration::Remix => "Remix".into(),
            CourseGeneration::SNES => "Super Mario Kart (SNES)".into(),
            CourseGeneration::N64 => "Mario Kart 64 (N64)".into(),
            CourseGeneration::GBA => "Mario Kart: Super Circuit (GBA)".into(),
            CourseGeneration::GCN => "Mario Kart: Double Dash!! (GCN)".into(),
            CourseGeneration::DS => "Mario Kart DS (DS)".into(),
            CourseGeneration::Wii => "Mario Kart Wii (Wii)".into(),
            CourseGeneration::_3DS => "Mario Kart 7 (3DS)".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Course {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<u32>,
    pub id: CourseId,
    pub name: String, // current english name
    #[serde(serialize_with = "ordered_set")]
    pub favorite_items: HashSet<ItemRequirement>, // previous names (for updating/merging)
    pub last_changed: Option<DateTime<Utc>>,
}
impl Course {
    pub fn new(name: String, sort: Option<u32>) -> Self {
        Course {
            sort,
            id: course_id_from_name(&name),
            name,
            favorite_items: HashSet::new(),
            last_changed: Some(Utc::now()),
        }
    }

    pub fn merge(
        &mut self,
        Course {
            sort,
            id,
            name,
            favorite_items,
            last_changed,
        }: Course,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Item {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<u32>,
    pub id: ItemId,
    pub i_type: ItemType,
    pub name: String, // current english name
    pub rarity: Rarity,
    #[serde(serialize_with = "ordered_set")]
    pub favorite_courses: HashSet<CourseAvailability>,
    pub hashes: Vec<ItemHash>, // used for screenshot import
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

    fn points_cap_tiers(&self) -> Vec<ItemPoints> {
        match self.i_type {
            ItemType::Driver => match self.rarity {
                Rarity::Normal => vec![400, 600, 648, 704, 760],
                Rarity::Super => vec![450, 675, 765, 870, 975],
                Rarity::HighEnd => vec![500, 800, 980, 1190, 1400],
            },
            _ => match self.rarity {
                Rarity::Normal => vec![200, 300, 324, 352, 380],
                Rarity::Super => vec![220, 330, 366, 408, 450],
                Rarity::HighEnd => vec![250, 400, 490, 595, 700],
            },
        }
    }

    fn valid_points(&self) -> Vec<ItemPoints> {
        match self.i_type {
            ItemType::Driver => match self.rarity {
                Rarity::Normal => Some(0)
                    .into_iter()
                    .chain((400..600).step_by(8))
                    .chain((600..648).step_by(8))
                    .chain((648..704).step_by(8))
                    .chain((704..=760).step_by(8))
                    .collect_vec(),
                Rarity::Super => Some(0)
                    .into_iter()
                    .chain((450..675).step_by(9))
                    .chain((675..765).step_by(15))
                    .chain((765..870).step_by(15))
                    .chain((870..=975).step_by(15))
                    .collect_vec(),
                Rarity::HighEnd => Some(0)
                    .into_iter()
                    .chain((500..800).step_by(12))
                    .chain((800..980).step_by(30))
                    .chain((980..1190).step_by(30))
                    .chain((1190..=1400).step_by(30))
                    .collect_vec(),
            },
            _ => match self.rarity {
                Rarity::Normal => Some(0)
                    .into_iter()
                    .chain((200..300).step_by(4))
                    .chain((300..324).step_by(4))
                    .chain((324..352).step_by(4))
                    .chain((352..=380).step_by(4))
                    .collect_vec(),
                Rarity::Super => Some(0)
                    .into_iter()
                    .chain((220..280).step_by(4))
                    .chain((280..330).step_by(5)) // this is a sneaky split
                    .chain((330..366).step_by(6))
                    .chain((366..408).step_by(6))
                    .chain((408..=450).step_by(6))
                    .collect_vec(),
                Rarity::HighEnd => Some(0)
                    .into_iter()
                    .chain((250..400).step_by(6))
                    .chain((400..490).step_by(15))
                    .chain((490..595).step_by(15))
                    .chain((595..=700).step_by(15))
                    .collect_vec(),
            },
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MktItemHashes {
    #[serde(serialize_with = "ordered_map")]
    pub hashes: HashMap<ItemId, Vec<ItemHash>>,
}

impl MktItemHashes {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_json(json: &str) -> Result<MktItemHashes, Box<dyn Error>> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_json(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn load(file_name: &str) -> Result<MktItemHashes, Box<dyn Error>> {
        let json = fs::read_to_string(file_name)?;
        MktItemHashes::from_json(&json)
    }

    pub fn save(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_name, json)?;
        Ok(())
    }

    pub fn merge(&mut self, mut new_hashes: MktItemHashes) {
        for (id, hashes) in &mut self.hashes {
            if let Some(mut new_hashes) = new_hashes.hashes.remove(id) {
                hashes.append(&mut new_hashes);
            }
        }
        self.hashes.extend(new_hashes.hashes);
    }
}

impl FromIterator<(ItemId, ItemHash)> for MktItemHashes {
    fn from_iter<T: IntoIterator<Item = (ItemId, ItemHash)>>(iter: T) -> Self {
        let mut h = MktItemHashes::new();
        for (id, hash) in iter {
            h.hashes.entry(id).or_insert_with(Vec::new).push(hash);
        }
        h
    }
}
impl FromIterator<(ItemId, Vec<ItemHash>)> for MktItemHashes {
    fn from_iter<T: IntoIterator<Item = (ItemId, Vec<ItemHash>)>>(iter: T) -> Self {
        let mut h = MktItemHashes::new();
        for (id, hash) in iter {
            h.hashes.entry(id).or_insert_with(Vec::new).extend(hash);
        }
        h
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MktData {
    #[serde(serialize_with = "ordered_map")]
    pub courses: HashMap<CourseId, Course>,
    #[serde(serialize_with = "ordered_map")]
    pub drivers: HashMap<ItemId, Item>,
    #[serde(serialize_with = "ordered_map")]
    pub karts: HashMap<ItemId, Item>,
    #[serde(serialize_with = "ordered_map")]
    pub gliders: HashMap<ItemId, Item>,
}
impl MktData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_json(json: &str) -> Result<MktData, Box<dyn Error>> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn load(file_name: &str) -> Result<MktData, Box<dyn Error>> {
        let json = fs::read_to_string(file_name)?;
        MktData::from_json(&json)
    }

    pub fn save(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_name, json)?;
        Ok(())
    }

    pub fn get_item(&self, id: &str) -> Option<&Item> {
        self.drivers
            .get(id)
            .or_else(|| self.karts.get(id))
            .or_else(|| self.gliders.get(id))
    }

    pub fn merge_hashes(&mut self, MktItemHashes { hashes }: &MktItemHashes) {
        let types = [&mut self.drivers, &mut self.karts, &mut self.gliders];
        for list in types {
            for item in list.values_mut() {
                if let Some(hash) = hashes.get(&item.id) {
                    item.hashes.extend(hash.clone());
                }
            }
        }
    }

    pub fn hashes(&self) -> MktItemHashes {
        self.drivers
            .values()
            .chain(self.karts.values())
            .chain(self.gliders.values())
            .filter(|i| !i.hashes.is_empty())
            .map(|i| (i.id.clone(), i.hashes.clone()))
            .collect()
    }

    pub fn merge(&mut self, mut new_data: MktData) {
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
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedItem {
    pub id: ItemId,
    pub lvl: ItemLvl,
    pub points: ItemPoints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_changed: Option<DateTime<Utc>>,
}

impl OwnedItem {
    pub fn new(id: ItemId, lvl: ItemLvl, points: ItemPoints) -> Self {
        OwnedItem {
            id,
            lvl,
            points,
            added: Some(Utc::now()),
            last_changed: Some(Utc::now()),
        }
    }

    // replace item
    pub fn merge(
        &mut self,
        OwnedItem {
            id,
            lvl,
            points,
            added,
            last_changed,
        }: OwnedItem,
    ) {
        let mut changed = false;

        if !id.is_empty() && self.id != id {
            self.id = id;
            changed = true;
        }
        if self.lvl != lvl {
            self.lvl = lvl;
            changed = true;
        }
        if self.points != points {
            self.points = points;
            changed = true;
        }

        if changed {
            self.last_changed = last_changed;
            if self.added.is_none() {
                self.added = added;
            }
        }
    }

    // increase level and points
    pub fn improve(
        &mut self,
        OwnedItem {
            id,
            lvl,
            points,
            added,
            last_changed,
        }: OwnedItem,
    ) {
        let mut changed = false;

        // only update from same or none
        if !id.is_empty() && self.id != id {
            return;
        }
        if self.lvl < lvl {
            self.lvl = lvl;
            changed = true;
        }
        if self.points < points {
            self.points = points;
            changed = true;
        }

        if changed {
            self.last_changed = last_changed;
            if self.added.is_none() {
                self.added = added;
            }
        }
    }

    pub fn normalize_points(&mut self, item: &Item) {
        let incs = item.valid_points();
        let max_points = *incs.last().expect("max points");
        for (a, b) in incs.iter().tuple_windows() {
            if (a + 1..=*b).contains(&self.points) {
                self.points = *b;
                break;
            }
        }
        self.points = self.points.max(0).min(max_points);
    }

    pub fn increment_points(&mut self, item: &Item) {
        let incs = item.valid_points();
        self.points = incs
            .into_iter()
            .find(|p| *p > self.points)
            .unwrap_or(self.points);
    }
    pub fn decrement_points(&mut self, item: &Item) {
        let incs = item.valid_points();
        self.points = incs
            .into_iter()
            .rev()
            .find(|p| *p < self.points)
            .unwrap_or(self.points);
    }

    pub fn point_cap_lvl(&self, item: &Item) -> u8 {
        let caps = item.points_cap_tiers();
        for (l, cap) in caps.into_iter().skip(1).enumerate() {
            if self.points <= cap {
                return l as u8;
            }
        }
        0
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MktInventory {
    #[serde(serialize_with = "ordered_map")]
    pub drivers: HashMap<ItemId, OwnedItem>,
    #[serde(serialize_with = "ordered_map")]
    pub karts: HashMap<ItemId, OwnedItem>,
    #[serde(serialize_with = "ordered_map")]
    pub gliders: HashMap<ItemId, OwnedItem>,
}
impl MktInventory {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_json(json: &str) -> Result<MktData, Box<dyn Error>> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_json(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn load(file_name: &str) -> Result<MktData, Box<dyn Error>> {
        let json = fs::read_to_string(file_name)?;
        MktData::from_json(&json)
    }

    pub fn from_items(items: Vec<OwnedItem>, data: &MktData) -> Self {
        let mut items = items.into_iter().into_group_map_by(|i| {
            data.drivers
                .get(&i.id)
                .or_else(|| data.karts.get(&i.id))
                .or_else(|| data.gliders.get(&i.id))
                .map(|i| i.i_type)
        });
        MktInventory {
            drivers: items
                .remove(&Some(ItemType::Driver))
                .unwrap_or_default()
                .into_iter()
                .map(|i| (i.id.clone(), i))
                .collect(),
            karts: items
                .remove(&Some(ItemType::Kart))
                .unwrap_or_default()
                .into_iter()
                .map(|i| (i.id.clone(), i))
                .collect(),
            gliders: items
                .remove(&Some(ItemType::Glider))
                .unwrap_or_default()
                .into_iter()
                .map(|i| (i.id.clone(), i))
                .collect(),
        }
    }

    pub fn update_inventory(&mut self, mut new_inv: MktInventory) {
        // drivers
        for (id, driver) in &mut self.drivers {
            if let Some(new_driver) = new_inv.drivers.remove(id) {
                driver.merge(new_driver);
            }
        }
        self.drivers.extend(new_inv.drivers);

        // karts
        for (id, kart) in &mut self.karts {
            if let Some(new_kart) = new_inv.karts.remove(id) {
                kart.merge(new_kart);
            }
        }
        self.karts.extend(new_inv.karts);

        // gliders
        for (id, glider) in &mut self.gliders {
            if let Some(new_glider) = new_inv.gliders.remove(id) {
                glider.merge(new_glider);
            }
        }
        self.gliders.extend(new_inv.gliders);
    }

    pub fn clear_dates(&mut self) {
        self.drivers
            .values_mut()
            .chain(self.karts.values_mut())
            .chain(self.gliders.values_mut())
            .for_each(|i| {
                i.added = None;
                i.last_changed = None;
            })
    }
}
