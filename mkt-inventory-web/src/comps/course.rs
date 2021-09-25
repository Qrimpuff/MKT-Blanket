use gloo::console;
use mkt_data::{item_type_from_id, CourseId};
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::{
    agents::{data::DataStore, inventory::Inventory},
    comps::item::Item,
};

pub enum Msg {
    Toggle,
    DataStore(ReadOnly<DataStore>),
    Inventory(ReadOnly<Inventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: CourseId,
}

pub struct Course {
    course: Option<mkt_data::Course>,
    visible: bool,
    driver_count: usize,
    kart_count: usize,
    glider_count: usize,
    driver_owned_count: usize,
    kart_owned_count: usize,
    glider_owned_count: usize,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    _inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for Course {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback_data = ctx.link().callback(Msg::DataStore);
        let callback_inv = ctx.link().callback(Msg::Inventory);
        Self {
            course: None,
            visible: false,
            driver_count: 0,
            kart_count: 0,
            glider_count: 0,
            driver_owned_count: 0,
            kart_owned_count: 0,
            glider_owned_count: 0,
            _data_store: DataStore::bridge(callback_data),
            _inventory: Inventory::bridge(callback_inv),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Toggle => {
                self.visible = !self.visible;
                true
            }
            Msg::DataStore(state) => {
                let state = state.borrow();

                let course = state.data.courses.get(&ctx.props().id);

                if course.map(|c| c.last_changed) != self.course.as_ref().map(|c| c.last_changed) {
                    self.course = course.cloned();
                    self.driver_count = 0;
                    self.kart_count = 0;
                    self.glider_count = 0;
                    if let Some(course) = &self.course {
                        for r in &course.favorite_items {
                            if let Some(i_type) = item_type_from_id(&r.id) {
                                match i_type {
                                    mkt_data::ItemType::Driver => self.driver_count += 1,
                                    mkt_data::ItemType::Kart => self.kart_count += 1,
                                    mkt_data::ItemType::Glider => self.glider_count += 1,
                                }
                            }
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Msg::Inventory(state) => {
                let state = state.borrow();

                if let Some(course) = &self.course {
                    let mut driver_owned_count = 0;
                    let mut kart_owned_count = 0;
                    let mut glider_owned_count = 0;
                    for r in &course.favorite_items {
                        if let Some(item) = state.inv.drivers.get(&r.id) {
                            if item.lvl >= r.lvl {
                                driver_owned_count += 1;
                            }
                            continue;
                        }
                        if let Some(item) = state.inv.karts.get(&r.id) {
                            if item.lvl >= r.lvl {
                                kart_owned_count += 1;
                            }
                            continue;
                        }
                        if let Some(item) = state.inv.gliders.get(&r.id) {
                            if item.lvl >= r.lvl {
                                glider_owned_count += 1;
                            }
                            continue;
                        }
                    }

                    if self.driver_owned_count != driver_owned_count
                        || self.kart_owned_count != kart_owned_count
                        || self.glider_owned_count != glider_owned_count
                    {
                        self.driver_owned_count = driver_owned_count;
                        self.kart_owned_count = kart_owned_count;
                        self.glider_owned_count = glider_owned_count;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // prevent scrolling on modal
        let _: Option<_> = try {
            web_sys::window()?
                .document()?
                .query_selector("html")
                .ok()??
                .set_class_name(self.visible.then_some("is-clipped").unwrap_or(""));
        };
        if let Some(course) = self.course.as_ref() {
            let items = if self.visible {
                html! {
                    <ul>
                    { for course.favorite_items.iter().map(|r| html!{ <li><Item id={r.id.clone()} /></li> }) }
                    </ul>
                }
            } else {
                html! {}
            };
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::Toggle)}>
                        <span>{ &course.name }</span>
                        <span>{self.driver_owned_count}</span>
                        <span>{self.kart_owned_count}</span>
                        <span>{self.glider_owned_count}</span>
                        <span class="icon is-small ml-auto">
                            // TODO: add karts and gliders
                            {
                                if self.driver_owned_count == self.driver_count {
                                    html! {<i class="fas fa-star has-text-success"></i>}
                                } else if self.driver_owned_count > 0 {
                                    html! {<i class="fas fa-check has-text-success"></i>}
                                } else if self.driver_owned_count == 0 {
                                    html! {<i class="fas fa-times has-text-danger"></i>}
                                } else {
                                    html! {}
                                }
                            }
                        </span>
                    </button>
                    <div class={classes!("modal", self.visible.then_some("is-active"))}>
                        <div class="modal-background" onclick={ctx.link().callback(|_| Msg::Toggle)}></div>
                        <div class="modal-content">
                            <div class="box">
                                { items }
                            </div>
                        </div>
                        <button class="modal-close is-large" aria-label="close" onclick={ctx.link().callback(|_| Msg::Toggle)}></button>
                    </div>
                </>
            }
        } else {
            html! {
                <p>{ "no_course" }</p>
            }
        }
    }
}
