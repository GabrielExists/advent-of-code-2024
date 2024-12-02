use yew::prelude::*;
use crate::*;
use web_sys::HtmlTextAreaElement;

pub struct App {
    input: String,
    first_output: String,
    second_output: String,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    NewText(String),
}

const LOCAL_STORAGE_KEY: &'static str = "INPUT";

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let input = local_storage.get_item(LOCAL_STORAGE_KEY).unwrap().unwrap_or(String::new());
        ctx.link().send_message(AppMessage::NewText(input));
        Self {
            input: "".to_string(),
            first_output: String::new(),
            second_output: String::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::NewText(contents) => {
                self.input = contents;
                let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                let _ = local_storage.set_item(LOCAL_STORAGE_KEY, &self.input);
                (self.first_output, self.second_output) = day1::puzzle(&self.input);
                // self.output = b.unwrap();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <div class="flex flex-col gap-2 text-gray-100 border-gray-400 m-2 mx-3">
            <div class="p-2 border border-gray-400 rounded">
                {"Input"}
            </div>
            <textarea onchange={ctx.link().batch_callback(move |event: Event| {
                    if let Some(input) = event.target_dyn_into::<HtmlTextAreaElement>() {
                        Some(AppMessage::NewText(input.value()))
                    } else {
                        None
                    }
               })}
                value={self.input.clone()}
                class="rounded-md h-72 bg-gray-700"
            >
            </textarea>
            <div class="p-2 border border-gray-400 rounded">
                {"Output for first part"}
            </div>
            <div class="p-2 border border-gray-400 rounded">
                {&self.first_output}
            </div>
            <div class="p-2 border border-gray-400 rounded">
                {"Output for second part"}
            </div>
            <div class="p-2 border border-gray-400 rounded">
                {&self.second_output}
            </div>
        </div>
        }
    }
}

