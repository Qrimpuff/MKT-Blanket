use gloo::events::EventListener;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Bridge,
};

use crate::{
    agents::inventory::{Inventory, InventoryRequest},
    comps::modal_popup::view_confirm_modal,
};

use super::modal_popup::update_popup_layer;

#[derive(Clone)]
pub enum Msg {
    ToggleModal,
    DeleteInv,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct DeleteInv {
    visible: bool,
    popup_listener: Option<EventListener>,
    inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for DeleteInv {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            visible: false,
            popup_listener: None,
            inventory: Inventory::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.visible = !self.visible;
                self.popup_listener = update_popup_layer(self.visible, ctx, Msg::ToggleModal);
                true
            }
            Msg::DeleteInv => {
                self.inventory.send(InventoryRequest::Delete);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button class={classes!("button", "is-danger")} onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                    <span class="icon"><i class="fas fa-trash-alt"/></span>
                    <span>{ "Delete Inventory" }</span>
                </button>
                { view_confirm_modal(self.visible,
                    Some(html!{ "Delete Inventory" }),
                    html!{ "Are you sure you want to delete your inventory?" },
                    ctx, Msg::ToggleModal, Msg::DeleteInv) }
            </>
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        if self.visible {
            self.popup_listener = update_popup_layer(false, ctx, Msg::ToggleModal);
        }
    }
}
