// #![cfg(target_arch = "wasm32")]

use std::cmp::max;
use std::collections::BTreeMap;
use yew::prelude::*;
use gloo::timers::callback::{Timeout};

pub struct App {
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    DummyMessage,
}


impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::DummyMessage => {
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <div class="flex flex-row text-gray-100 border-gray-400 m-2 mx-3">
            <div class="p-2 gap-y-2 border border-gray-400 flex-col rounded">
                {"ABCD"}
            </div>
        </div>
        }
    }
}

