use mkt_data::CourseId;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{DataInvCourse, DataInventory, DataInventoryAgent, Shared},
    comps::item::Item,
};

pub enum Msg {
    Toggle,
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
            Msg::Toggle => {
                self.visible = !self.visible;
                // prevent scrolling on modal
                let _: Option<_> = try {
                    let html = web_sys::window()?
                        .document()?
                        .query_selector("html")
                        .ok()??;
                    let mut layer = html
                        .get_attribute("data-popup-layer")
                        .and_then(|a| a.parse().ok())
                        .unwrap_or(0);
                    layer += if self.visible { 1 } else { -1 };
                    html.set_attribute("data-popup-layer", &layer.to_string())
                        .ok()?;
                    if layer == 1 {
                        html.set_class_name("is-clipped");
                    } else if layer == 0 {
                        html.set_class_name("");
                    }
                };
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
            let items = if self.visible {
                html! {
                    <ul>
                    { for course.data.favorite_items.iter().map(|r| html!{ <li><Item id={r.id.clone()} /></li> }) }
                    </ul>
                }
            } else {
                html! {}
            };
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::Toggle)}>
                        <span>{ &course.data.name }</span>
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
                    <div class={classes!("modal", self.visible.then_some("is-active"))}>
                        <div class="modal-background" onclick={ctx.link().callback(|_| Msg::Toggle)}></div>
                        <div class="modal-content">
                            <div class="box">
                                <div class="subtitle">{ &course.data.name }</div>
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
