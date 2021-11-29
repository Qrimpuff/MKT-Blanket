use gloo::events::EventListener;
use itertools::Itertools;
use mkt_data::{item_type_from_id, CourseId, ItemType};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{
        DataInvCourse, DataInventory, DataInventoryAgent, DataInventoryRequest, Shared,
    },
    comps::item::Item,
};

use super::modal_popup::*;

#[derive(Clone)]
pub enum Msg {
    ToggleModal,
    DataInventory(Shared<DataInventory>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: CourseId,
    #[prop_or(0)]
    pub lvl_req: u8,
    #[prop_or(None)]
    pub i_type: Option<ItemType>,
}

pub struct Course {
    course: Option<Shared<DataInvCourse>>,
    visible: bool,
    popup_listener: Option<EventListener>,
    data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for Course {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            course: None,
            visible: false,
            popup_listener: None,
            data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.visible = !self.visible;
                self.popup_listener = update_popup_layer(self.visible, ctx, Msg::ToggleModal);
                true
            }
            Msg::DataInventory(state) => {
                let state = state.read().unwrap();
                self.course = state.courses.get(&ctx.props().id).cloned();
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        self.data_inventory.send(DataInventoryRequest::Refresh);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(course) = &self.course {
            let course = course.read().unwrap();
            html! {
                <>
                    <button class="button is-fullwidth" onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                        <span class="is-clipped-ellipsis">{ &course.data.name }</span>
                        <span>{if ctx.props().lvl_req > 1 { html! { <i class="ml-4">{format!(" Lvl. {}", ctx.props().lvl_req)}</i> } } else { html! {} }}</span>
                        <span class="icon is-small ml-auto">
                            {
                                if let Some(stats) = &course.stats {
                                    let count = match ctx.props().i_type {
                                        None => stats.driver_count + stats.kart_count + stats.glider_count,
                                        Some(ItemType::Driver) => stats.driver_count,
                                        Some(ItemType::Kart) => stats.kart_count,
                                        Some(ItemType::Glider) => stats.glider_count,
                                    };
                                    let owned_count = match ctx.props().i_type {
                                        None => stats.driver_owned_count + stats.kart_owned_count + stats.glider_owned_count,
                                        Some(ItemType::Driver) => stats.driver_owned_count,
                                        Some(ItemType::Kart) => stats.kart_owned_count,
                                        Some(ItemType::Glider) => stats.glider_owned_count,
                                    };
                                    let covered = match ctx.props().i_type {
                                        None => stats.driver_owned_count > 0 && stats.kart_owned_count > 0 && stats.glider_owned_count > 0,
                                        Some(ItemType::Driver) => stats.driver_owned_count > 0,
                                        Some(ItemType::Kart) => stats.kart_owned_count > 0,
                                        Some(ItemType::Glider) => stats.glider_owned_count > 0,
                                    };
                                    if owned_count == count {
                                        html! {<i class="fas fa-star has-text-success"></i>}
                                    } else if covered {
                                        html! {<i class="fas fa-check has-text-success"></i>}
                                    } else if owned_count == 0 {
                                        html! {<i class="fas fa-times has-text-danger"></i>}
                                    } else {
                                        html! {<i class="fas fa-info-circle has-text-danger"></i>}
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </span>
                    </button>
                    { self.view_course_modal(ctx) }
                </>
            }
        } else {
            html! {
                <p>{ "no_course" }</p>
            }
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        if self.visible {
            self.popup_listener = update_popup_layer(false, ctx, Msg::ToggleModal);
        }
    }
}

impl Course {
    fn view_course_modal(&self, ctx: &Context<Self>) -> Html {
        if self.visible {
            if let Some(course) = &self.course {
                let course = course.read().unwrap();
                let mut drivers = course
                    .data
                    .favorite_items
                    .iter()
                    .filter(|r| item_type_from_id(&r.id) == Some(ItemType::Driver))
                    .collect_vec();
                drivers.sort_by_key(|i| &i.id);
                let mut karts = course
                    .data
                    .favorite_items
                    .iter()
                    .filter(|r| item_type_from_id(&r.id) == Some(ItemType::Kart))
                    .collect_vec();
                karts.sort_by_key(|i| &i.id);
                let mut gliders = course
                    .data
                    .favorite_items
                    .iter()
                    .filter(|r| item_type_from_id(&r.id) == Some(ItemType::Glider))
                    .collect_vec();
                gliders.sort_by_key(|i| &i.id);
                let items = html! {
                    <>
                    <p class="subtitle is-6">{"Drivers"}</p>
                    <div class="columns is-multiline">
                    { for drivers.iter().map(|r| html!{ <div class="column is-full py-1"><Item id={r.id.clone()} lvl_req={r.lvl} /></div> }) }
                    </div>
                    <p class="subtitle is-6">{"Karts"}</p>
                    <div class="columns is-multiline">
                    { for karts.iter().map(|r| html!{ <div class="column is-full py-1"><Item id={r.id.clone()} lvl_req={r.lvl} /></div> }) }
                    </div>
                    <p class="subtitle is-6">{"Gliders"}</p>
                    <div class="columns is-multiline">
                    { for gliders.iter().map(|r| html!{ <div class="column is-full py-1"><Item id={r.id.clone()} lvl_req={r.lvl} /></div> }) }
                    </div>
                    </>
                };

                let title = html! { &course.data.name };
                let content = html! { items };

                view_popup_modal(
                    self.visible,
                    Some(title),
                    content,
                    None,
                    ctx,
                    Msg::ToggleModal,
                )
            } else {
                html! {
                    <p>{ "no_course" }</p>
                }
            }
        } else {
            html! {}
        }
    }
}
