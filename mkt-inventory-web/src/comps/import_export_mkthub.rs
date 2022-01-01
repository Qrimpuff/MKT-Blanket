use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    agents::data_inventory::{
        DataInvItem, DataInventory, DataInventoryAgent, DataInventoryRequest, Shared,
    },
    comps::data_manager::download_file,
};

pub enum Msg {
    DownloadMktHub,
    DataInventory(Shared<DataInventory>),
    _ToggleDisplay,
}

pub struct ImportExportMktHub {
    items: Vec<Shared<DataInvItem>>,
    visible: bool,
    data_inventory: Box<dyn Bridge<DataInventoryAgent>>,
}

impl Component for ImportExportMktHub {
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
            Msg::DownloadMktHub => {
                use std::fmt::Write;
                let mut text = String::new();
                writeln!(&mut text, "DRIVER NAME,LEVEL,SKILL,POINTS").unwrap();
                for i in &self.items {
                    let i = i.read().unwrap();
                    if let Some(inv) = i.inv.as_ref() {
                        writeln!(
                            &mut text,
                            "{},{},0,{}",
                            i.data.name,
                            inv.lvl,
                            inv.points,
                        )
                        .unwrap();
                    }
                }
                download_file("mkthub_import.csv", text.as_str());
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
        let mkthub = if self.visible {
            html! {
                <div class="block">
                <button class={classes!("button", "is-info")} onclick={ctx.link().callback(|_| Msg::DownloadMktHub)}>
                    <span class="icon"><i class="fas fa-download"/></span>
                    <span>{ "Download MKTHub Sheet" }</span>
                </button>
                </div>
            }
        } else {
            html! {}
        };
        html! {
            <>
                { mkthub }
            </>
        }
    }
}
