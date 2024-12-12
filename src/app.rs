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
    diagnostic: Diagnostic,
    tab_index: usize,
    up_to_date: bool,
}

pub struct DayOutput {
    pub silver_output: String,
    pub gold_output: String,
    pub diagnostic: Diagnostic,
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
    Run,
    TabClicked(usize),
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
    days.push(add_day(day5::puzzle, &mut index));
    days.push(add_day(day6::puzzle, &mut index));
    days.push(add_day(day7::puzzle, &mut index));
    days.push(add_day(day8::puzzle, &mut index));
    days.push(add_day(day9::puzzle, &mut index));
    days.push(add_day(day10::puzzle, &mut index));
    days.push(add_day(day11::puzzle, &mut index));
    days.push(add_day(day12::puzzle, &mut index));
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

#[derive(Clone, Debug, Default)]
pub struct Diagnostic {
    message: String,
    tabs: Vec<Tab>,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub title: String,
    pub strings: Vec<String>,
    pub grid: Vec<Vec<GridCell>>,
}
#[derive(Clone, Debug)]
pub struct GridCell {
    // pub icon: IconId,
    pub text: String,
    pub class: Classes,
}

impl Diagnostic {
    pub fn simple(message: String) -> Self {
        Diagnostic {
            message,
            tabs: vec![],
        }
    }
    pub fn with_tabs(tabs: Vec<Tab>, message: String) -> Self {
        Diagnostic {
            message,
            tabs,
        }
    }
}

impl App {
    fn run(&mut self) {
        if let Some(day) = self.days.get(self.day_index) {
            let output = (day.puzzle)(&self.input);
            self.silver_output = output.silver_output;
            self.gold_output = output.gold_output;
            self.diagnostic = output.diagnostic;
        } else {
            self.silver_output = format!("Day not found");
            self.gold_output = format!("Day not found");
        }
        self.up_to_date = true;
    }
    fn refresh(&mut self) {
        let (title_text, diagnostic) = Self::get_refresh_values(&self.days, self.day_index);
        self.title_text = title_text;
        self.diagnostic = diagnostic;
        self.silver_output = String::new();
        self.gold_output = String::new();
        self.up_to_date = false;
    }
    fn get_refresh_values(days: &Vec<Day>, day_index: usize) -> (String, Diagnostic) {
        if let Some(day) = days.get(day_index) {
            (day.text.clone(), Diagnostic::simple(format!("Puzzle not yet run")))
        } else {
            ("No day with that index found".to_string(), Default::default())
        }
    }
}

pub fn class_string(text: &'static str) -> Classes{
    let mut split = text.split(" ");
    if let Some(first) = split.next() {
        split.into_iter().fold(classes!(first), |mut class, substring| {
            class.extend(classes!(substring));
            class
        })
    } else {
        Classes::new()
    }
}

pub fn merge(base: &'static str, additional: &Classes) -> Classes {
    let mut base = class_string(base);
    base.extend(additional);
    base
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let days = get_days();
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let input = local_storage.get_item(LOCAL_STORAGE_INPUT).unwrap().unwrap_or(String::new());
        let index: usize = local_storage.get_item(LOCAL_STORAGE_INDEX).unwrap().map(|index| index.parse::<usize>().ok()).unwrap_or(None).unwrap_or(0);
        if let Some(day) = days.get(index) {
            let day_text = day.text.clone();
            Self {
                days,
                day_index: index,
                title_text: day_text,
                input,
                silver_output: String::new(),
                gold_output: String::new(),
                diagnostic: Diagnostic::simple(format!("Puzzle not yet run")),
                tab_index: 0,
                up_to_date: false,
            }
        } else {
            Self {
                days,
                day_index: index,
                title_text: "No initial day with that index found".to_string(),
                input,
                silver_output: String::new(),
                gold_output: String::new(),
                diagnostic: Default::default(),
                tab_index: 0,
                up_to_date: false,
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
                true
            }
            AppMessage::SetDay(index) => {
                self.day_index = index;
                let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                let _ = local_storage.set_item(LOCAL_STORAGE_INDEX, &format!("{}", self.day_index));
                self.refresh();
                true
            }
            AppMessage::Run => {
                self.run();
                true
            }
            AppMessage::TabClicked(index) => {
                self.tab_index = index;
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
                <button onclick={
                    ctx.link().callback(move |_| AppMessage::Run)
                } class="p-2 m-2 border border-gray-400 rounded-md text-lg">
                    { if self.up_to_date {
                        "Run"
                    } else {
                        "** Run **"
                    } }
                </button>
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
                    {&self.diagnostic.message}
                </div>
            </div>
            <div class="flex flex-col gap-2 m-2">
                <div class="flex flex-row">
                    {for self.diagnostic.tabs.iter().enumerate().map(|(index, tab)|{
                        html!{
                    <button onclick={ctx.link().callback(move |_| AppMessage::TabClicked(index))} class={merge("p-1 m-1 border border-gray-400 rounded-md", &if self.tab_index == index {classes!("bg-slate-700")} else {classes!("")})}>
                            {&tab.title}
                    </button>
                        }
                    })}
                </div>
                <div>
                    <button onclick={
                        let index = self.tab_index;
                        ctx.link().callback(move |_| AppMessage::TabClicked(index + 1))
                    } class="p-1 m-1 border border-gray-400 rounded-md">
                        {"Next tab"}
                    </button>
                </div>
                <div class="p-4 flex flex-col border border-gray-400">
                {if let Some(tab) = self.diagnostic.tabs.get(self.tab_index) {
                    html! {
                        <>
                    {for tab.strings.iter().map(|string|{
                        html!{
                            <div>
                                {string}
                            </div>
                        }
                    })}
                    {for tab.grid.iter().map(|row|{
                        html! {
                            <div class="flex flex-row">
                        {for row.iter().map(|cell|{
                            html! {
                                // <Icon icon_id={cell.icon} width={"2em".to_string()} height={"2em".to_string()} />
                                <div class={merge("w-4 h-4", &cell.class)}>
                                    {&cell.text}
                                </div>
                            }
                        })}
                            </div>
                        }
                    })}
                        </>
                    }
                } else {
                    html! {
                        <div class="p-2 font-mono">{"No tab info"}</div>
                    }
                }}
                </div>
            </div>
        </div>
        }
    }
}

