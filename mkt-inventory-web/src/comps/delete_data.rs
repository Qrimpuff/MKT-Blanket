use gloo::{
    events::EventListener,
    storage::{LocalStorage, Storage},
};
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Bridge,
};

use crate::{
    agents::{
        data::{DataRequest, DataStore},
        inventory::{Inventory, InventoryRequest},
    },
    comps::modal_popup::view_confirm_modal,
};

use super::modal_popup::update_popup_layer;

#[derive(Clone)]
pub enum Msg {
    ToggleModal,
    DeleteData,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct DeleteData {
    visible: bool,
    popup_listener: Option<EventListener>,
    data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for DeleteData {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            visible: false,
            popup_listener: None,
            data_store: DataStore::bridge(Callback::noop()),
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
            Msg::DeleteData => {
                self.data_store.send(DataRequest::Delete);
                self.inventory.send(InventoryRequest::Delete);
                LocalStorage::delete("mkt_hash");
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button class={classes!("button", "is-danger")} onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                    <span>{ "Delete All Data" }</span>
                    <span class="icon"><i class="fas fa-trash-alt"/></span>
                </button>
                { view_confirm_modal(self.visible,
                    Some(html!{ "Delete All Data" }),
                    html!{ "Are you sure you want to delete ALL data, including your inventory and item hashes?" },
                    ctx, Msg::ToggleModal, Msg::DeleteData) }
            </>
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        if self.visible {
            self.popup_listener = update_popup_layer(false, ctx, Msg::ToggleModal);
        }
    }
}
