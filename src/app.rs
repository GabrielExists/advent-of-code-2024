use yew::prelude::*;
use crate::*;
use web_sys::HtmlTextAreaElement;

pub struct App {
    days: Vec<Day>,
    day_index: usize,
    title_text: String,
    input: String,
    first_output: String,
    second_output: String,
}

pub type DayFunction = fn(&str) -> (String, String);

#[derive(Clone, Debug)]
pub struct Day {
    pub text: String,
    pub puzzle: DayFunction,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    NewText(String),
    SetDay(usize),
}

const LOCAL_STORAGE_INPUT: &'static str = "INPUT";
const LOCAL_STORAGE_INDEX: &'static str = "INDEX";

fn get_days() -> Vec<Day> {
    let mut index = 0;
    let mut days = Vec::new();
    days.push(add_day(day1::puzzle, &mut index));
    days.push(add_day(day2::puzzle, &mut index));
    days
}

fn add_day(function: DayFunction, index: &mut usize) -> Day {
    let this_index = *index;
    *index += 1;
    Day {
        text: format!("Day {}", this_index + 1),
        puzzle: function,
        index: this_index,
    }
}

impl App {
    fn refresh(&mut self) {
        if let Some(pair) = Self::get_refresh_values(&self.days, self.day_index, &self.input) {
            ((self.first_output, self.second_output), self.title_text) = pair;
        } else {
            self.first_output = String::new();
            self.second_output = String::new();
            self.title_text = "No day with that index found".to_string();
        }
    }
    fn get_refresh_values(days: &Vec<Day>, day_index: usize, input: &str) -> Option<((String, String), String)> {
        let day = days.get(day_index);
        if let Some(day) = day {
            Some(((day.puzzle)(input), day.text.clone()))
        } else {
            None
        }
    }
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let days = get_days();
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let input = local_storage.get_item(LOCAL_STORAGE_INPUT).unwrap().unwrap_or(String::new());
        let index: usize = local_storage.get_item(LOCAL_STORAGE_INDEX).unwrap().map(|index| index.parse::<usize>().ok()).unwrap_or(None).unwrap_or(0);
        let ((first_output, second_output), title_text) = App::get_refresh_values(&days, index, &input).unwrap_or(((String::new(), String::new()), "No initial day with that index found".to_string()));
        Self {
            days,
            day_index: index,
            title_text,
            input,
            first_output,
            second_output,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::NewText(contents) => {
                self.input = contents;
                let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                let _ = local_storage.set_item(LOCAL_STORAGE_INPUT, &self.input);
                self.refresh();
                // self.output = b.unwrap();
                true
            }
            AppMessage::SetDay(index) => {
                self.day_index = index;
                let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                let _ = local_storage.set_item(LOCAL_STORAGE_INDEX, &format!("{}", self.day_index));
                self.refresh();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <div class="flex flex-row text-gray-100 border-gray-400 m-2 mx-3">
            <div class="flex flex-col gap-2 m-2">
                { for self.days.iter().map(|day| {
                    let index = day.index;
                    html! {
                        <button onclick={ctx.link().callback(move |_| {
                            AppMessage::SetDay(index)
                        })} class="p-2 border rounded">
                            {&day.text}
                        </button>
                    }
                })}
            </div>
            <div class="flex flex-col gap-2 text-gray-100 border-gray-400 m-2 mx-3">
                <div class="p-2 text-2xl border-b-2 border-gray-400 rounded">
                    {&self.title_text}
                </div>
                <div class="p-2 border-b border-gray-400 rounded">
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
                <div class="p-2 border-b border-gray-400 rounded">
                    {"Output for first part"}
                </div>
                <div class="p-2 border border-gray-400 rounded">
                    {&self.first_output}
                </div>
                <div class="p-2 border-b border-gray-400 rounded">
                    {"Output for second part"}
                </div>
                <div class="p-2 border border-gray-400 rounded">
                    {&self.second_output}
                </div>
            </div>
        </div>
        }
    }
}

