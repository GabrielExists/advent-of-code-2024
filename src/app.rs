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
        <div class="flex flex-row">
            <div class="p-2 border border-slate-800 bg-blue-100 flex-col gap-y-2"></div>
        </div>
        }
    }
}

