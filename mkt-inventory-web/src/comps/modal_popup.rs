use yew::prelude::*;

use crate::{
    agents::data_inventory::{DataInvCourse, DataInvItem, Shared},
    comps::{course::Course, item::Item},
};

pub fn update_popup_layer(visible: bool) {
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
        layer += if visible { 1 } else { -1 };
        html.set_attribute("data-popup-layer", &layer.to_string())
            .ok()?;
        if layer == 1 {
            html.set_class_name("is-clipped");
        } else if layer == 0 {
            html.set_class_name("");
        }
    };
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
                    <>
                        <div>{ format!("Level: {}", inv.lvl) }</div>
                        <div>{ format!("Points: {}", inv.points)}</div>
                    </>
                }
            } else {
                html! {}
            };
            let courses = html! {
                <ul>
                { for item.data.favorite_courses.iter().map(|r| html!{ <li><Course id={r.id.clone()} /></li> }) }
                </ul>
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
            let items = html! {
                <ul>
                { for course.data.favorite_items.iter().map(|r| html!{ <li><Item id={r.id.clone()} /></li> }) }
                </ul>
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
