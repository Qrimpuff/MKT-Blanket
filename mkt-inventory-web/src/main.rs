#![feature(bool_to_option)]
#![feature(try_blocks)]
// rust-analyser doesn't like gloo::console
#![allow(unused_unsafe)]

mod agents;
mod comps;

use agents::{
    data::{DataRequest, DataStore},
    inventory::{Inventory, InventoryRequest},
};
use mkt_data::ItemType;
use yew::prelude::*;
use yew_agent::{
    utils::store::{Bridgeable, StoreWrapper},
    Bridge,
};

use crate::comps::{
    course_list::CourseList, data_manager::*, import::Import, item_list::ItemList, summary::Summary,
};

struct App {
    _data_store: Box<dyn Bridge<StoreWrapper<DataStore>>>,
    _inventory: Box<dyn Bridge<StoreWrapper<Inventory>>>,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // initial load
        let mut _data_store = DataStore::bridge(Callback::noop());
        _data_store.send(DataRequest::Load);
        let mut _inventory = Inventory::bridge(Callback::noop());
        _inventory.send(InventoryRequest::Load);
        Self {
            _data_store,
            _inventory,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <div class="container is-clipped px-2">
                    <h1 class="title">{ "MKT Inventory" }</h1>
                    <Summary/>
                    <CourseList/>
                    <ItemList i_type={ItemType::Driver}/>
                    <ItemList i_type={ItemType::Kart}/>
                    <ItemList i_type={ItemType::Glider}/>
                    <Import/>
                    <DataManager/>
                </div>
            </section>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
