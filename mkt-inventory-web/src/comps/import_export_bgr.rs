use gloo_utils::{document, window};
use mkt_data::ItemType;
use wasm_bindgen::{ JsCast};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agents::data_inventory::{
    DataInvItem, DataInventory, DataInventoryAgent, DataInventoryRequest, Shared,
};

pub enum Msg {
    CopyBgr,
    DataInventory(Shared<DataInventory>),
    _ToggleDisplay,
}

pub struct ImportExportBgr {
    items: Vec<Shared<DataInvItem>>,
    visible: bool,
    data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for ImportExportBgr {
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
            Msg::CopyBgr => {
                Self::copy_bgr();
                false
            },
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        self.items.clear();
        self.data_inventory.send(DataInventoryRequest::Refresh);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let items = if self.visible {
            html! {
                <>
                <button class={classes!("button", "is-info")} onclick={ctx.link().callback(|_| Msg::CopyBgr)}>
                    <span>{ "Copy BGR Sheet" }</span>
                    <span class="icon"><i class="fas fa-copy"/></span>
                </button>
                <table id="bgr_table">
                { for self.items.iter().map(|i| {
                    let i = i.read().unwrap();
                    html!{
                        <tr>
                            <td>
                                {i.data.get_bgr_name()}
                            </td>
                            <td>
                                {match i.data.i_type {
                                    ItemType::Driver => 'D',
                                    ItemType::Kart => 'K',
                                    ItemType::Glider => 'G',
                                }}
                            </td>
                            <td>
                                {i.inv.as_ref().map(|n| n.lvl).unwrap_or(0)}
                            </td>
                            <td>
                                {i.inv.as_ref().map(|n| n.point_cap_tier(&i.data)).unwrap_or(0)}
                            </td>
                        </tr>
                    }
                }) }
                </table>
                </>
            }
        } else {
            html! {}
        };
        html! {
            <>
                { items }
            </>
        }
    }
}

impl ImportExportBgr {
    fn copy_bgr() {
        /*
        var range = document.createRange();
        range.selectNode(document.getElementById(containerid));
        window.getSelection().addRange(range);
        document.execCommand("copy");
        alert("Text has been copied, now paste in the text-area")
        */

        let r: Option<_> = try {
            let range = document().create_range().ok()?;
            range
                .select_node(&document().get_element_by_id("bgr_table")?.into())
                .ok()?;
            window().get_selection().ok()??.add_range(&range).ok()?;
            document()
                .dyn_into::<web_sys::HtmlDocument>().ok()?
                .exec_command("copy").ok()?;
        };
        r.unwrap()
    }
}
