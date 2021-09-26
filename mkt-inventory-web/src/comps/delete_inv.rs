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
    inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for DeleteInv {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            visible: false,
            inventory: Inventory::bridge(Callback::noop()),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.visible = !self.visible;
                update_popup_layer(self.visible);
                true
            }
            Msg::DeleteInv => {
                self.inventory.send(InventoryRequest::Delete);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let confirm = html! {
            <>
            <div class="subtitle">{ "Delete Inventory" }</div>
            <p>{ "Are you sure you want to delete your inventory?" }</p>
            </>
        };
        html! {
            <>
                <button class={classes!("button", "is-danger")} onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                    <span>{ "Delete Inventory" }</span>
                    <span class="icon"><i class="fas fa-trash-alt"/></span>
                </button>
                { view_confirm_modal(self.visible, confirm, ctx, Msg::ToggleModal, Msg::DeleteInv) }
            </>
        }
    }
}
