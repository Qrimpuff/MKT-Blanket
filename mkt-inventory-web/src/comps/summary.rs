use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::agents::data::DataStore;

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct Summary {
    course_count: usize,
    driver_count: usize,
    kart_count: usize,
    glider_count: usize,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Component for Summary {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStore);
        Self {
            course_count: 0,
            driver_count: 0,
            kart_count: 0,
            glider_count: 0,
            _data_store: DataStore::bridge(callback),
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
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <h2>{ "Summary" }</h2>
                <ul>
                    <li>{ format!("courses: {}/{}", 0, self.course_count) }</li>
                    <li>{ format!("drivers: {}/{}", 0, self.driver_count) }</li>
                    <li>{ format!("karts: {}/{}", 0, self.kart_count) }</li>
                    <li>{ format!("gliders: {}/{}", 0, self.glider_count) }</li>
                </ul>
            </>
        }
    }
}
