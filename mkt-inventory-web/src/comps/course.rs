use mkt_data::CourseId;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{DataInvCourse, DataInventory, DataInventoryAgent, Shared},
    comps::modal_popup::view_course_modal,
};

use super::modal_popup::update_popup_layer;

#[derive(Clone)]
pub enum Msg {
    ToggleModal,
    DataInventory(Shared<DataInventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: CourseId,
}

pub struct Course {
    course: Option<Shared<DataInvCourse>>,
    visible: bool,
    _data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for Course {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            course: None,
            visible: false,
            _data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.visible = !self.visible;
                update_popup_layer(self.visible);
                true
            }
            Msg::DataInventory(state) => {
                let state = state.read().unwrap();
                self.course = state.courses.get(&ctx.props().id).cloned();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(course) = &self.course {
            let course = course.read().unwrap();
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                        <span class="is-clipped-ellipsis">{ &course.data.name }</span>
                        <span class="icon is-small ml-auto">
                            // TODO: add karts and gliders
                            {
                                if let Some(stats) = &course.stats {
                                    if stats.driver_owned_count == stats.driver_count {
                                        html! {<i class="fas fa-star has-text-success"></i>}
                                    } else if stats.driver_owned_count > 0 {
                                        html! {<i class="fas fa-check has-text-success"></i>}
                                    } else if stats.driver_owned_count == 0 {
                                        html! {<i class="fas fa-times has-text-danger"></i>}
                                    } else {
                                        html! {}
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </span>
                    </button>
                    { view_course_modal(self.visible, &self.course, ctx, Msg::ToggleModal) }
                </>
            }
        } else {
            html! {
                <p>{ "no_course" }</p>
            }
        }
    }
}
