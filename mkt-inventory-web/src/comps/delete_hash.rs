use gloo::{
    events::EventListener,
    storage::{LocalStorage, Storage},
};
use yew::prelude::*;

use crate::comps::modal_popup::view_confirm_modal;

use super::modal_popup::update_popup_layer;

#[derive(Clone)]
pub enum Msg {
    ToggleModal,
    DeleteHash,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct DeleteHash {
    visible: bool,
    popup_listener: Option<EventListener>,
}

impl Component for DeleteHash {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            visible: false,
            popup_listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModal => {
                self.visible = !self.visible;
                self.popup_listener = update_popup_layer(self.visible, ctx, Msg::ToggleModal);
                true
            }
            Msg::DeleteHash => {
                LocalStorage::delete("mkt_hash");
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let confirm = html! {
            <>
            <div class="subtitle">{ "Delete Item Hashes" }</div>
            <p>{ "Are you sure you want to delete your item hashes?" }</p>
            </>
        };
        html! {
            <>
                <button class={classes!("button", "is-danger")} onclick={ctx.link().callback(|_| Msg::ToggleModal)}>
                    <span>{ "Delete Hashes" }</span>
                    <span class="icon"><i class="fas fa-trash-alt"/></span>
                </button>
                { view_confirm_modal(self.visible, confirm, ctx, Msg::ToggleModal, Msg::DeleteHash) }
            </>
        }
    }
}
