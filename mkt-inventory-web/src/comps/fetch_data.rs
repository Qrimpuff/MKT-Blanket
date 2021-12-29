use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agents::update::{UpdateAgent, UpdateRequest, UpdateResponse};

pub enum Msg {
    Update(UpdateResponse),
    Fetch,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {}

pub struct FetchData {
    fetching: bool,
    update: Box<dyn Bridge<UpdateAgent>>,
}

impl Component for FetchData {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Msg::Update);

        Self {
            fetching: false,
            update: UpdateAgent::bridge(callback),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Fetch => {
                self.fetching = true;
                self.update.send(UpdateRequest::CheckUpdateData);
                true
            }
            Msg::Update(resp) => match resp {
                UpdateResponse::DoneDataUpdate | UpdateResponse::NoDataUpdate => {
                    self.fetching = false;
                    true
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <button class={classes!("button", "is-success", self.fetching.then_some("is-loading"))} onclick={ctx.link().callback(|_| Msg::Fetch)}>
                    <span>{ "Fetch Data" }</span>
                    <span class="icon"><i class="fas fa-sync-alt"/></span>
                </button>
            </>
        }
    }
}
