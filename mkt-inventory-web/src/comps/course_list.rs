use itertools::Itertools;
use mkt_data::{course_generation_from_id, course_parts_from_id, course_type_from_id, CourseType};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{DataInvCourse, DataInventory, DataInventoryAgent, Shared},
    comps::course::Course,
};

pub enum Msg {
    DataInventory(Shared<DataInventory>),
    _Toggle,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct CourseList {
    courses: Vec<Shared<DataInvCourse>>,
    visible: bool,
    _data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for CourseList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            courses: Vec::new(),
            visible: true,
            _data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataInventory(state) => {
                let state = state.read().unwrap();
                if state.courses.len() != self.courses.len() {
                    self.courses = state.courses.values().cloned().collect();
                    self.courses
                        .sort_by_key(|c| course_parts_from_id(&c.read().unwrap().data.id));
                    true
                } else {
                    false
                }
            }
            Msg::_Toggle => {
                self.visible = !self.visible;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let courses = if self.visible {
            html! {
                { for self.courses.iter().group_by(|c| course_generation_from_id(&c.read().unwrap().data.id)).into_iter().map(|(gen, cs)| {
                    let mut expected = 0;
                    html! {
                        <>
                        <h3 class="subtitle"> { gen } </h3>
                        <div class="columns is-multiline">
                        { for cs.map(|c| {
                            let mut actual = match course_type_from_id(&c.read().unwrap().data.id) {
                                CourseType::Normal => 0,
                                CourseType::Reverse => 1,
                                CourseType::Trick => 2,
                                CourseType::ReverseTrick => 3,
                            };
                            if actual < expected {
                                actual += 4;
                            }
                            let offset = html! { for (expected..actual).map(|_| html! { <div class="column is-one-quarter py-1 is-hidden-mobile"/> }) };
                            expected = (actual + 1) % 4;
                            html!{
                                <>
                                { offset }
                                <div class="column is-one-quarter py-1">
                                    <Course id={c.read().unwrap().data.id.clone()} />
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
                <h2 class="title is-4">{"Coverage"}</h2>
                <div class="block">
                    { courses }
                </div>
            </>
        }
    }
}
