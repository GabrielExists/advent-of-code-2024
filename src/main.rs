mod app;
mod day1;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;

#[cfg(target_arch = "wasm32")]
use app::App;

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!(r#"Please don't run this manually, instead use "trunk serve" or "trunk build"."#)
}