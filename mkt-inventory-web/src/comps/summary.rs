use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::agents::{data::DataStore, inventory::Inventory};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Inventory(ReadOnly<Inventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct Summary {
    course_count: usize,
    driver_count: usize,
    kart_count: usize,
    glider_count: usize,
    driver_owned_count: usize,
    kart_owned_count: usize,
    glider_owned_count: usize,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    _inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for Summary {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback_data = ctx.link().callback(Msg::DataStore);
        let callback_inv = ctx.link().callback(Msg::Inventory);
        Self {
            course_count: 0,
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

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(state) => {
                let state = state.borrow();
                if self.course_count != state.data.courses.len()
                    || self.driver_count != state.data.drivers.len()
                    || self.kart_count != state.data.karts.len()
                    || self.glider_count != state.data.gliders.len()
                {
                    self.course_count = state.data.courses.len();
                    self.driver_count = state.data.drivers.len();
                    self.kart_count = state.data.karts.len();
                    self.glider_count = state.data.gliders.len();
                    true
                } else {
                    false
                }
            }
            Msg::Inventory(state) => {
                let state = state.borrow();
                if self.driver_owned_count != state.inv.drivers.len()
                    || self.kart_owned_count != state.inv.karts.len()
                    || self.glider_owned_count != state.inv.gliders.len()
                {
                    self.driver_owned_count = state.inv.drivers.len();
                    self.kart_owned_count = state.inv.karts.len();
                    self.glider_owned_count = state.inv.gliders.len();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <h2>{ "Summary" }</h2>
                <ul>
                    <li>{ format!("courses: {}/{}", 0, self.course_count) }</li>
                    <li>{ format!("drivers: {}/{}", self.driver_owned_count, self.driver_count) }</li>
                    <li>{ format!("karts: {}/{}", self.kart_owned_count, self.kart_count) }</li>
                    <li>{ format!("gliders: {}/{}", self.glider_owned_count, self.glider_count) }</li>
                </ul>
            </>
        }
    }
}
