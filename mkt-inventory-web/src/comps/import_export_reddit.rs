use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agents::data_inventory::{
    DataInvItem, DataInventory, DataInventoryAgent, DataInventoryRequest, Shared,
};

#[wasm_bindgen(module = "/js/utils.js")]
extern "C" {
    fn copyTextToClipboard(text: &str);
}

pub enum Msg {
    CopyReddit,
    DataInventory(Shared<DataInventory>),
    _ToggleDisplay,
}

pub struct ImportExportReddit {
    items: Vec<Shared<DataInvItem>>,
    visible: bool,
    data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for ImportExportReddit {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::DataInventory);
        Self {
            items: Vec::new(),
            visible: true,
            data_inventory: DataInventoryAgent::bridge(callback),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataInventory(state) => {
                let state = state.read().unwrap();
                if state.drivers.len() + state.karts.len() + state.gliders.len() != self.items.len()
                {
                    self.items = state
                        .drivers
                        .values()
                        .chain(state.karts.values())
                        .chain(state.gliders.values())
                        .cloned()
                        .collect();
                    self.items.sort_by_key(|c| {
                        (c.read().unwrap().data.i_type, c.read().unwrap().data.sort)
                    });
                    true
                } else {
                    false
                }
            }
            Msg::_ToggleDisplay => {
                self.visible = !self.visible;
                true
            }
            Msg::CopyReddit => {
                use std::fmt::Write;
                let mut text = String::new();
                for i in &self.items {
                    let i = i.read().unwrap();
                    writeln!(
                        &mut text,
                        "{}\t{}",
                        i.data.get_reddit_name(),
                        i.inv.as_ref().map(|n| n.lvl).unwrap_or(0),
                    )
                    .unwrap();
                }
                copyTextToClipboard(&text);
                false
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        self.items.clear();
        self.data_inventory.send(DataInventoryRequest::Refresh);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let reddit = if self.visible {
            html! {
                <div class="block">
                <button class={classes!("button", "is-info")} onclick={ctx.link().callback(|_| Msg::CopyReddit)}>
                    <span class="icon"><i class="fas fa-copy"/></span>
                    <span>{ "Send /u/spinachpants Sheet to clipboard" }</span>
                </button>
                </div>
            }
        } else {
            html! {}
        };
        html! {
            <>
                { reddit }
            </>
        }
    }
}
