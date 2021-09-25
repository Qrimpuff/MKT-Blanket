use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::RwLock,
};

use super::{
    data::DataStore,
    inventory::{Inventory, InventoryRequest},
};
use mkt_data::{item_type_from_id, Course, CourseId, Item, ItemId, OwnedItem};
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Agent, AgentLink, Bridge, Context, HandlerId,
};
use yew_agent::{Dispatched, Dispatcher};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Inventory(ReadOnly<Inventory>),
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

pub struct ItemStats {}

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
                        mkt_data::ItemType::Driver => {
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
                        mkt_data::ItemType::Kart => {
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
                        mkt_data::ItemType::Glider => {
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
    type Input = ();
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
        // TODO: update state
        match msg {
            Msg::DataStore(data) => {
                let data = &data.borrow().data;
                let mut state = self.state.write().unwrap();

                // update courses
                let mut new_courses = HashMap::<CourseId, Shared<DataInvCourse>>::default();
                for course in data.courses.values() {
                    let new_course = if let Some(c) = state.courses.remove(&course.id) {
                        (*c.write().unwrap()).data = course.clone();
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
                        (*i.write().unwrap()).data = driver.clone();
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
                        (*i.write().unwrap()).data = kart.clone();
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
                        (*i.write().unwrap()).data = glider.clone();
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
                for driver in inv.drivers.values() {
                    if let Some(i) = state.drivers.get(&driver.id) {
                        (*i.write().unwrap()).inv = Some(driver.clone());
                    };
                }

                // update karts
                for kart in inv.karts.values() {
                    if let Some(i) = state.karts.get(&kart.id) {
                        (*i.write().unwrap()).inv = Some(kart.clone());
                    };
                }

                // update gliders
                for glider in inv.gliders.values() {
                    if let Some(i) = state.gliders.get(&glider.id) {
                        (*i.write().unwrap()).inv = Some(glider.clone());
                    };
                }

                state.update_stats();

                for handler in self.handlers.iter() {
                    self.link.respond(*handler, self.state.clone());
                }
            }
        }
    }

    fn handle_input(&mut self, _msg: Self::Input, _id: yew_agent::HandlerId) {}

    fn connected(&mut self, id: HandlerId) {
        self.handlers.insert(id);
        self.link.respond(id, self.state.clone());
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.handlers.remove(&id);
    }
}
