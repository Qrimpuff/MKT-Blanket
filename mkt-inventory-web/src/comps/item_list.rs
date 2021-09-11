use yew::prelude::*;

use crate::comps::item::Item;

pub enum Msg {}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct ItemList {}

impl Component for ItemList {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div>{ "item list" }</div>
                <Item/>
            </>
        }
    }
}
