use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::RwLock,
};

use super::{
    data::DataStore,
    inventory::{Inventory, InventoryRequest},
};
use mkt_data::{item_type_from_id, Course, CourseId, Item, ItemId, ItemType, OwnedItem};
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Agent, AgentLink, Bridge, Context, HandlerId,
};
use yew_agent::{Dispatched, Dispatcher};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Inventory(ReadOnly<Inventory>),
}

pub enum DataInventoryRequest {
    Refresh,
    RefreshAll,
}

pub type Shared<T> = Rc<RwLock<T>>;

pub struct CourseStats {
    pub driver_count: usize,
    pub kart_count: usize,
    pub glider_count: usize,
    pub driver_owned_count: usize,
    pub kart_owned_count: usize,
    pub glider_owned_count: usize,
}

pub struct DataInvCourse {
    pub data: Course,
    pub stats: Option<CourseStats>,
}

pub struct ItemStats {
    pub fav_course_count: usize,
    pub max_fav_course_count: usize,
    pub add_course_count: usize,
    pub max_add_course_count: usize,
}

pub struct DataInvItem {
    pub data: Item,
    pub inv: Option<OwnedItem>,
    pub stats: Option<ItemStats>,
}

#[derive(Default)]
pub struct DataInventory {
    pub courses: HashMap<CourseId, Shared<DataInvCourse>>,
    pub drivers: HashMap<ItemId, Shared<DataInvItem>>,
    pub karts: HashMap<ItemId, Shared<DataInvItem>>,
    pub gliders: HashMap<ItemId, Shared<DataInvItem>>,
}

impl DataInventory {
    fn update_stats(&mut self) {
        // courses statistics
        for course in self.courses.values() {
            let mut course = course.write().unwrap();

            let mut driver_count = 0;
            let mut kart_count = 0;
            let mut glider_count = 0;
            let mut driver_owned_count = 0;
            let mut kart_owned_count = 0;
            let mut glider_owned_count = 0;

            for r in &course.data.favorite_items {
                if let Some(i_type) = item_type_from_id(&r.id) {
                    match i_type {
                        ItemType::Driver => {
                            driver_count += 1;
                            if self
                                .drivers
                                .get(&r.id)
                                .and_then(|i| i.read().unwrap().inv.as_ref().map(|i| i.lvl))
                                .unwrap_or(0)
                                >= r.lvl
                            {
                                driver_owned_count += 1;
                            }
                        }
                        ItemType::Kart => {
                            kart_count += 1;
                            if self
                                .karts
                                .get(&r.id)
                                .and_then(|i| i.read().unwrap().inv.as_ref().map(|i| i.lvl))
                                .unwrap_or(0)
                                >= r.lvl
                            {
                                kart_owned_count += 1;
                            }
                        }
                        ItemType::Glider => {
                            glider_count += 1;
                            if self
                                .gliders
                                .get(&r.id)
                                .and_then(|i| i.read().unwrap().inv.as_ref().map(|i| i.lvl))
                                .unwrap_or(0)
                                >= r.lvl
                            {
                                glider_owned_count += 1;
                            }
                        }
                    }
                }
            }

            course.stats = Some(CourseStats {
                driver_count,
                kart_count,
                glider_count,
                driver_owned_count,
                kart_owned_count,
                glider_owned_count,
            })
        }

        // items statistics
        for item in self
            .drivers
            .values()
            .chain(self.karts.values())
            .chain(self.gliders.values())
        {
            let mut item = item.write().unwrap();

            let mut fav_course_count = 0;
            let mut max_fav_course_count = 0;
            let mut add_course_count = 0;
            let mut max_add_course_count = 0;

            let lvl = item.inv.as_ref().map(|i| i.lvl).unwrap_or(0);
            for r in &item.data.favorite_courses {
                let add = self
                    .courses
                    .get(&r.id)
                    .and_then(|c| {
                        c.read()
                            .unwrap()
                            .stats
                            .as_ref()
                            .map(|s| match item.data.i_type {
                                ItemType::Driver => s.driver_owned_count,
                                ItemType::Kart => s.kart_owned_count,
                                ItemType::Glider => s.glider_owned_count,
                            })
                    })
                    .filter(|c| *c > 0)
                    .map(|_| 0)
                    .unwrap_or(1);
                if lvl == 0 {
                    if r.lvl == 1 {
                        add_course_count += add;
                    }
                    max_add_course_count += add;
                } else if lvl >= r.lvl {
                    fav_course_count += 1;
                } else {
                    max_add_course_count += add;
                }
                max_fav_course_count += 1;
            }

            item.stats = Some(ItemStats {
                fav_course_count,
                max_fav_course_count,
                add_course_count,
                max_add_course_count,
            })
        }
    }
}

pub struct DataInventoryAgent {
    /// Currently subscribed components and agents
    handlers: HashSet<HandlerId>,
    /// Link to itself so Store::handle_input can send actions to reducer
    pub link: AgentLink<Self>,

    /// The actual Store
    pub state: Shared<DataInventory>,

    /// A circular dispatcher to itself so the store is not removed
    pub self_dispatcher: Dispatcher<Self>,

    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Agent for DataInventoryAgent {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = DataInventoryRequest;
    type Output = Shared<DataInventory>;

    fn create(link: AgentLink<Self>) -> Self {
        let callback_data = link.callback(Msg::DataStore);
        let callback_inv = link.callback(Msg::Inventory);

        let state = Rc::new(RwLock::new(Default::default()));
        let handlers = HashSet::new();

        // Link to self to never go out of scope
        let self_dispatcher = Self::dispatcher();

        Self {
            handlers,
            link,
            state,
            self_dispatcher,
            _data_store: DataStore::bridge(callback_data),
            inventory: Inventory::bridge(callback_inv),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        // update state
        match msg {
            Msg::DataStore(data) => {
                let data = &data.borrow().data;
                let mut state = self.state.write().unwrap();

                // update courses
                let mut new_courses = HashMap::<CourseId, Shared<DataInvCourse>>::default();
                for course in data.courses.values() {
                    let new_course = if let Some(c) = state.courses.remove(&course.id) {
                        c.write().unwrap().data = course.clone();
                        c
                    } else {
                        Rc::new(RwLock::new(DataInvCourse {
                            data: course.clone(),
                            stats: None,
                        }))
                    };
                    new_courses.insert(course.id.clone(), new_course);
                }
                state.courses = new_courses;

                // update drivers
                let mut new_drivers = HashMap::<ItemId, Shared<DataInvItem>>::default();
                for driver in data.drivers.values() {
                    let new_driver = if let Some(i) = state.drivers.remove(&driver.id) {
                        i.write().unwrap().data = driver.clone();
                        i
                    } else {
                        Rc::new(RwLock::new(DataInvItem {
                            data: driver.clone(),
                            inv: None,
                            stats: None,
                        }))
                    };
                    new_drivers.insert(driver.id.clone(), new_driver);
                }
                state.drivers = new_drivers;

                // update karts
                let mut new_karts = HashMap::<ItemId, Shared<DataInvItem>>::default();
                for kart in data.karts.values() {
                    let new_kart = if let Some(i) = state.karts.remove(&kart.id) {
                        i.write().unwrap().data = kart.clone();
                        i
                    } else {
                        Rc::new(RwLock::new(DataInvItem {
                            data: kart.clone(),
                            inv: None,
                            stats: None,
                        }))
                    };
                    new_karts.insert(kart.id.clone(), new_kart);
                }
                state.karts = new_karts;

                // update gliders
                let mut new_gliders = HashMap::<ItemId, Shared<DataInvItem>>::default();
                for glider in data.gliders.values() {
                    let new_glider = if let Some(i) = state.gliders.remove(&glider.id) {
                        i.write().unwrap().data = glider.clone();
                        i
                    } else {
                        Rc::new(RwLock::new(DataInvItem {
                            data: glider.clone(),
                            inv: None,
                            stats: None,
                        }))
                    };
                    new_gliders.insert(glider.id.clone(), new_glider);
                }
                state.gliders = new_gliders;

                self.inventory.send(InventoryRequest::Refresh);
            }
            Msg::Inventory(inv) => {
                let inv = &inv.borrow().inv;
                let mut state = self.state.write().unwrap();

                // update drivers
                for driver in state.drivers.values() {
                    let mut driver = driver.write().unwrap();
                    if let Some(i) = inv.drivers.get(&driver.data.id) {
                        driver.inv = Some(i.clone());
                    } else {
                        driver.inv = None;
                    }
                }

                // update karts
                for kart in state.karts.values() {
                    let mut kart = kart.write().unwrap();
                    if let Some(i) = inv.karts.get(&kart.data.id) {
                        kart.inv = Some(i.clone());
                    } else {
                        kart.inv = None;
                    }
                }

                // update gliders
                for glider in state.gliders.values() {
                    let mut glider = glider.write().unwrap();
                    if let Some(i) = inv.gliders.get(&glider.data.id) {
                        glider.inv = Some(i.clone());
                    } else {
                        glider.inv = None;
                    }
                }

                state.update_stats();

                self.link.send_input(DataInventoryRequest::RefreshAll);
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: yew_agent::HandlerId) {
        match msg {
            DataInventoryRequest::Refresh => self.link.respond(id, self.state.clone()),
            DataInventoryRequest::RefreshAll => {
                for handler in self.handlers.iter() {
                    self.link.respond(*handler, self.state.clone());
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.handlers.insert(id);
        self.link.respond(id, self.state.clone());
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.handlers.remove(&id);
    }
}
