use gloo::events::EventListener;
use itertools::Itertools;
use mkt_data::{item_type_from_id, ItemType};
use wasm_bindgen::JsValue;
use yew::prelude::*;

use crate::{
    agents::data_inventory::{DataInvCourse, DataInvItem, Shared},
    comps::{course::Course, item::Item},
};

pub fn update_popup_layer<COMP>(
    visible: bool,
    ctx: &Context<COMP>,
    toggle: COMP::Message,
) -> Option<EventListener>
where
    COMP: Component,
    COMP::Message: Clone,
{
    // prevent scrolling on modal
    let html = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("html")
        .unwrap()
        .unwrap();
    let mut layer = html
        .get_attribute("data-popup-layer")
        .and_then(|a| a.parse().ok())
        .unwrap_or(0);
    layer += if visible { 1 } else { -1 };
    html.set_attribute("data-popup-layer", &layer.to_string())
        .unwrap();
    if layer == 1 {
        html.set_class_name("is-clipped");
    } else if layer == 0 {
        html.set_class_name("");
    }

    let history = yew::utils::window().history().expect("no history");
    if visible {
        if layer == 1 {
            history
                .push_state_with_url(&JsValue::TRUE, "", None)
                .expect("push history");
        }

        let toggle_cb = ctx.link().callback(move |_| toggle.clone());
        let href = yew::utils::window().location().href().unwrap();
        Some(EventListener::new(
            &yew::utils::window(),
            "popstate",
            move |_| {
                let prev_layer = layer;
                let prev_href = href.clone();
                gloo::console::info!("from popstate");

                let href = yew::utils::window().location().href().unwrap();
                let html = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .query_selector("html")
                    .unwrap()
                    .unwrap();
                let layer = html
                    .get_attribute("data-popup-layer")
                    .and_then(|a| a.parse().ok())
                    .unwrap_or(0);
                if prev_layer == layer && prev_href == href {
                    if layer > 1 {
                        history
                            .push_state_with_url(&JsValue::TRUE, "", None)
                            .expect("push history");
                    }
                    toggle_cb.emit(())
                }
            },
        ))
    } else {
        if layer == 0 && history.state().unwrap() == JsValue::TRUE {
            history.back().unwrap();
        }
        None
    }
}

pub fn view_item_modal<COMP>(
    visible: bool,
    item: &Option<Shared<DataInvItem>>,
    ctx: &Context<COMP>,
    toggle: COMP::Message,
) -> Html
where
    COMP: Component,
    COMP::Message: Clone,
{
    if visible {
        if let Some(item) = &item {
            let item = item.read().unwrap();
            let inv = if let Some(inv) = &item.inv {
                html! {
                    <div class="block">
                        <div>{ format!("Level: {}", inv.lvl) }</div>
                        <div>{ format!("Points: {}", inv.points)}</div>
                    </div>
                }
            } else {
                html! {}
            };
            let courses = html! {
                <div class="columns is-multiline">
                { for item.data.favorite_courses.iter().map(|r| html!{ <div class="column is-full py-1"><Course id={r.id.clone()} lvl_req={r.lvl} i_type={item.data.i_type} /></div> }) }
                </div>
            };
            let toggle_cb = ctx.link().callback(move |_| toggle.clone());
            html! {
                <div class={classes!("modal", "is-active")}>
                    <div class="modal-background" onclick={&toggle_cb}></div>
                    <div class="modal-content">
                        <div class="box">
                            <div class="subtitle">
                                <span class="icon rarity-dot">
                                    {
                                        match item.data.rarity {
                                            mkt_data::Rarity::Normal => html! {<i class="fas fa-circle rarity-normal"></i>},
                                            mkt_data::Rarity::Super => html! {<i class="fas fa-circle rarity-super"></i>},
                                            mkt_data::Rarity::HighEnd => html! {<i class="fas fa-circle rarity-high-end"></i>},
                                        }
                                    }
                                </span>
                                <span>{ &item.data.name }</span>
                            </div>
                            { inv }
                            { courses }
                        </div>
                    </div>
                    <button class="modal-close is-large" aria-label="close" onclick={&toggle_cb}></button>
                </div>
            }
        } else {
            html! {
                <p>{ "no_item" }</p>
            }
        }
    } else {
        html! {}
    }
}

pub fn view_course_modal<COMP>(
    visible: bool,
    course: &Option<Shared<DataInvCourse>>,
    ctx: &Context<COMP>,
    toggle: COMP::Message,
) -> Html
where
    COMP: Component,
    COMP::Message: Clone,
{
    if visible {
        if let Some(course) = &course {
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
            let toggle_cb = ctx.link().callback(move |_| toggle.clone());
            html! {
                <div class={classes!("modal", "is-active")}>
                    <div class="modal-background" onclick={&toggle_cb}></div>
                    <div class="modal-content">
                        <div class="box">
                            <div class="subtitle">{ &course.data.name }</div>
                            { items }
                        </div>
                    </div>
                    <button class="modal-close is-large" aria-label="close" onclick={&toggle_cb}></button>
                </div>
            }
        } else {
            html! {
                <p>{ "no_course" }</p>
            }
        }
    } else {
        html! {}
    }
}

pub fn view_confirm_modal<COMP>(
    visible: bool,
    content: Html,
    ctx: &Context<COMP>,
    toggle: COMP::Message,
    confirm: COMP::Message,
) -> Html
where
    COMP: Component,
    COMP::Message: Clone,
{
    if visible {
        let toggle_1 = toggle.clone();
        let toggle_cb = ctx.link().callback(move |_| toggle_1.clone());
        let confirm_cb = ctx
            .link()
            .batch_callback(move |_| vec![confirm.clone(), toggle.clone()]);
        html! {
            <div class={classes!("modal", "is-active")}>
                <div class="modal-background" onclick={&toggle_cb}></div>
                <div class="modal-content">
                    <div class="box">
                        <div class="block">
                        { content }
                        </div>
                        <div class="buttons">
                            <button class="button is-danger" onclick={&confirm_cb}>{"Confirm"}</button>
                            <button class="button" onclick={&toggle_cb}>{"Cancel"}</button>
                        </div>
                    </div>
                </div>
                <button class="modal-close is-large" aria-label="close" onclick={&toggle_cb}></button>
            </div>
        }
    } else {
        html! {}
    }
}
