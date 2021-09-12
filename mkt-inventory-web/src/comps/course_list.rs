use mkt_data::CourseId;
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
                <ul>
                { for self.course_ids.iter().map(|id| html!{ <li><Course id={id.clone()} /></li> }) }
                </ul>
            }
        } else {
            html! {}
        };
        html! {
            <>
                <h2 onclick={ctx.link().callback(|_| Msg::Toggle)}>{ format!("{} {}", "Courses", if self.visible {'-'} else {'+'}) }</h2>
                { courses }
            </>
        }
    }
}
