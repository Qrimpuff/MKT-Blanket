use mkt_data::CourseId;
use yew::prelude::*;
use yew_agent::{Bridge, utils::store::{Bridgeable, ReadOnly, StoreWrapper}};

use crate::agents::data::DataStore;

pub enum Msg {
    DataStoreMsg(ReadOnly<DataStore>),}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: CourseId,
}

pub struct Course {
    course: Option<mkt_data::Course>,
    _course_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
}

impl Component for Course {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataStoreMsg);
        Self {
            course: None,
            _course_store: DataStore::bridge(callback),
        }
    }
    
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataStoreMsg(state) => {
                let state = state.borrow();

                let course = state.data.courses.get(&ctx.props().id);

                if course != self.course.as_ref() {
                    self.course = course.cloned();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if let Some(course) = self.course.as_ref() {
            html! {
                <div>{ &course.name }</div>
            }
        } else {
            html! {
                <div>{ "no_course" }</div>
            }
        }
    }
}
