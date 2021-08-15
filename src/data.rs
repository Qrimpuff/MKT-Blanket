type Id = String;
type Lvl = u8;
pub struct I18n {
    pub lang: String,
    pub text: String,
}

pub struct Course {
    pub id: Id,
    pub name: String,          // current default name (english)
    pub i18n_names: Vec<I18n>, // names in different languages
    pub aka: Vec<I18n>,        // previous names (for updating/merging)
}

pub struct Driver {
    pub id: Id,
    pub name: String,          // current default name (english)
    pub i18n_names: Vec<I18n>, // names in different languages
    pub aka: Vec<I18n>,        // previous names (for updating/merging)
    pub favorite_courses: Vec<(Course, Lvl)>,
    pub identifying_features: (), // TODO: used for screenshot import (not sure how yet)
}

pub struct Kart {
    pub id: Id,
    pub name: String,          // current default name (english)
    pub i18n_names: Vec<I18n>, // names in different languages
    pub aka: Vec<I18n>,        // previous names (for updating/merging)
    pub favorite_courses: Vec<(Course, Lvl)>,
    pub identifying_features: (), // TODO: used for screenshot import (not sure how yet)
}

pub struct Glider {
    pub id: Id,
    pub name: String,          // current default name (english)
    pub i18n_names: Vec<I18n>, // names in different languages
    pub aka: Vec<I18n>,        // previous names (for updating/merging)
    pub favorite_courses: Vec<(Course, Lvl)>,
    pub identifying_features: (), // TODO: used for screenshot import (not sure how yet)
}

pub struct MktDatabase {
    pub courses: Vec<Course>,
    pub drivers: Vec<Driver>,
    pub karts: Vec<Kart>,
    pub gliders: Vec<Glider>,
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

pub struct OwnedDriver {
    pub id: Id,
    pub lvl: Lvl,
    pub points: u16,
}

pub struct OwnedKart {
    pub id: Id,
    pub lvl: Lvl,
    pub points: u16,
}

pub struct OwnedGlider {
    pub id: Id,
    pub lvl: Lvl,
    pub points: u16,
}

pub struct MktInventory {
    pub drivers: Vec<OwnedDriver>,
    pub karts: Vec<OwnedKart>,
    pub gliders: Vec<OwnedGlider>,
}
impl MktInventory {
    pub fn update_inventory(&mut self, new_inv: MktInventory) {
        // update drivers
        new_inv.drivers.into_iter().for_each(|item| {
            if let Some(f_item) = self.drivers.iter_mut().find(|i_item| i_item.id == item.id) {
                *f_item = item;
            } else {
                self.drivers.push(item);
            }
        });

        // update karts
        new_inv.karts.into_iter().for_each(|item| {
            if let Some(f_item) = self.karts.iter_mut().find(|i_item| i_item.id == item.id) {
                *f_item = item;
            } else {
                self.karts.push(item);
            }
        });

        // update gliders
        new_inv.gliders.into_iter().for_each(|item| {
            if let Some(f_item) = self.gliders.iter_mut().find(|i_item| i_item.id == item.id) {
                *f_item = item;
            } else {
                self.gliders.push(item);
            }
        });
    }
}
