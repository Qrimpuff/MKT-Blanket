use gloo::events::EventListener;
use gloo_utils::window;
use wasm_bindgen::JsValue;
use yew::prelude::*;

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

    let history = window().history().expect("no history");
    if visible {
        if layer == 1 {
            history
                .push_state_with_url(&JsValue::TRUE, "", None)
                .expect("push history");
        }

        let toggle_cb = ctx.link().callback(move |_| toggle.clone());
        let href = window().location().href().unwrap();
        Some(EventListener::new(&window(), "popstate", move |_| {
            let prev_layer = layer;
            let prev_href = href.clone();
            gloo::console::info!("from popstate");

            let href = window().location().href().unwrap();
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
        }))
    } else {
        if layer == 0 && history.state().unwrap() == JsValue::TRUE {
            history.back().unwrap();
        }
        None
    }
}

pub fn view_confirm_modal<COMP>(
    visible: bool,
    title: Option<Html>,
    content: Html,
    ctx: &Context<COMP>,
    toggle: COMP::Message,
    confirm: COMP::Message,
) -> Html
where
    COMP: Component,
    COMP::Message: Clone,
{
    let toggle_1 = toggle.clone();
    let toggle_2 = toggle.clone();
    let toggle_cb = ctx.link().callback(move |_| toggle_1.clone());
    let confirm_cb = ctx
        .link()
        .batch_callback(move |_| vec![confirm.clone(), toggle.clone()]);
    view_popup_modal(
        visible,
        title,
        content,
        Some(html! {
            <>
            <button class="button is-danger" onclick={&confirm_cb}>{"Confirm"}</button>
            <button class="button" onclick={&toggle_cb}>{"Cancel"}</button>
            </>
        }),
        ctx,
        toggle_2,
    )
}

pub fn view_popup_modal<COMP>(
    visible: bool,
    title: Option<Html>,
    content: Html,
    buttons: Option<Html>,
    ctx: &Context<COMP>,
    toggle: COMP::Message,
) -> Html
where
    COMP: Component,
    COMP::Message: Clone,
{
    if visible {
        let toggle_cb = ctx.link().callback(move |_| toggle.clone());
        html! {
            <div class={classes!("modal", "is-active")}>
                <div class="modal-background" onclick={&toggle_cb}></div>
                <div class="modal-content">
                    <div class="box">
                        { title.map_or(html!{}, |title| html! {<div class="subtitle">{ title }</div>}) }
                        <div class="block">
                        { content }
                        </div>
                        {buttons.map_or(html!{}, |buttons| html! {<div class="buttons">{ buttons }</div>})}
                    </div>
                </div>
                <button class="modal-close is-large" aria-label="close" onclick={&toggle_cb}></button>
            </div>
        }
    } else {
        html! {}
    }
}
