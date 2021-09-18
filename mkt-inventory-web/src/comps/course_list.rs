use itertools::Itertools;
use mkt_data::{
    course_generation_from_id, course_type_from_id, CourseId, CourseType,
};
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::{agents::data::DataStore, comps::course::Course};

pub enum Msg {
    DataStore(ReadOnly<DataStore>),
    Toggle,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct CourseList {
    course_ids: Vec<CourseId>,
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    visible: bool,
}

impl Component for CourseList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStore);
        Self {
            course_ids: Vec::new(),
            _data_store: DataStore::bridge(callback),
            visible: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStore(state) => {
                let state = state.borrow();
                if state.data.courses.len() != self.course_ids.len() {
                    self.course_ids = state.data.courses.keys().cloned().collect();
                    true
                } else {
                    false
                }
            }
            Msg::Toggle => {
                self.visible = !self.visible;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let courses = if self.visible {
            html! {
                { for self.course_ids.iter().group_by(|id| course_generation_from_id(id)).into_iter().map(|(gen, ids)| {
                    let mut expected = 0;
                    html! {
                        <>
                        <h3 class="subtitle"> { gen } </h3>
                        <div class="columns is-multiline">
                        { for ids.map(|id| {
                            let mut actual = match course_type_from_id(id) {
                                CourseType::Normal => 0,
                                CourseType::Reverse => 1,
                                CourseType::Trick => 2,
                                CourseType::ReverseTrick => 3,
                            };
                            if actual < expected {
                                actual += 4;
                            }
                            let offset = html! { for (expected..actual).map(|_| html! { <div class="column is-one-quarter py-1 is-hidden-touch"/> }) };
                            expected = (actual + 1) % 4;
                            html!{
                                <>
                                { offset }
                                <div class="column is-one-quarter py-1">
                                    <Course id={id.clone()} />
                                </div>
                                </>
                            }
                        }) }
                        </div>
                        </>
                    }
                }) }
            }
        } else {
            html! {}
        };
        html! {
            <>
                <h2 class="subtitle" onclick={ctx.link().callback(|_| Msg::Toggle)}>{ format!("{} {}", "Courses", if self.visible {'-'} else {'+'}) }</h2>
                { courses }
            </>
        }
    }
}
