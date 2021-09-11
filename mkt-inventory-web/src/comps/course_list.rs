use mkt_data::CourseId;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, ReadOnly, StoreWrapper},
    Bridge,
};

use crate::{agents::data::DataStore, comps::course::Course};

pub enum Msg {
    DataStoreMsg(ReadOnly<DataStore>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct CourseList {
    course_ids: Vec<CourseId>,
    _course_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Component for CourseList {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStoreMsg);
        Self {
            course_ids: Vec::new(),
            _course_store: DataStore::bridge(callback),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStoreMsg(state) => {
                let state = state.borrow();
                if state.data.courses.len() != self.course_ids.len() {
                    self.course_ids = state.data.courses.keys().cloned().collect();
                    self.course_ids.sort_unstable();
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
                <div>{ "course list" }</div>
                { for self.course_ids.iter().map(|id| html!{ <Course key={id.clone()} id={id.clone()} /> }) }
            </>
        }
    }
}
