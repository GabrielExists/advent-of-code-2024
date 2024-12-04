use yew::prelude::*;
use crate::*;
use web_sys::HtmlTextAreaElement;

pub struct App {
    days: Vec<Day>,
    day_index: usize,
    title_text: String,
    input: String,
    silver_output: String,
    gold_output: String,
    diagnostic_output: String,
}

pub struct DayOutput {
    pub silver_output: String,
    pub gold_output: String,
    pub diagnostic: String,
}
pub type DayFunction = fn(&str) -> DayOutput;

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
    days.push(add_day(day3::puzzle, &mut index));
    days.push(add_day(day4::puzzle, &mut index));
    days
}

fn add_day(function: DayFunction, index: &mut usize) -> Day {
    let this_index = *index;
    *index += 1; Day {
        text: format!("Day {}", this_index + 1),
        puzzle: function,
        index: this_index,
    }
}

impl App {
    fn refresh(&mut self) {
        if let Some((day_output, title_text)) = Self::get_refresh_values(&self.days, self.day_index, &self.input) {
            self.title_text = title_text;
            self.silver_output = day_output.silver_output;
            self.gold_output = day_output.gold_output;
            self.diagnostic_output = day_output.diagnostic;
        } else {
            self.silver_output = String::new();
            self.gold_output = String::new();
            self.title_text = "No day with that index found".to_string();
        }
    }
    fn get_refresh_values(days: &Vec<Day>, day_index: usize, input: &str) -> Option<(DayOutput, String)> {
        let day = days.get(day_index);
        if let Some(day) = day {
            let day_output = (day.puzzle)(input);
            Some((day_output, day.text.clone()))
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
        let day_output = App::get_refresh_values(&days, index, &input);
        match day_output {
            Some((day_output, title_text)) => {
                Self {
                    days,
                    day_index: index,
                    title_text,
                    input,
                    silver_output: day_output.silver_output,
                    gold_output: day_output.gold_output,
                    diagnostic_output: day_output.diagnostic,
                }
            }
            None => {
                Self {
                    days,
                    day_index: index,
                    title_text: "No initial day with that index found".to_string(),
                    input,
                    silver_output: String::new(),
                    gold_output: String::new(),
                    diagnostic_output: String::new(),
                }
            }
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
                    {&self.silver_output}
                </div>
                <div class="p-2 border-b border-gray-400 rounded">
                    {"Output for second part"}
                </div>
                <div class="p-2 border border-gray-400 rounded">
                    {&self.gold_output}
                </div>
                <div class="p-2 border-b border-gray-400 rounded">
                    {"Diagnostic field"}
                </div>
                <div class="p-2 border border-gray-400 rounded">
                    {&self.diagnostic_output}
                </div>
            </div>
        </div>
        }
    }
}

