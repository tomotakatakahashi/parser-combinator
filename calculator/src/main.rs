use dioxus::prelude::*;
mod expr_parser;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Calculator {}
    }
}

#[component]
fn Calculator() -> Element {
    let initial_input = "1 + (2 + 3) * 4";
    let mut input_text = use_signal(|| String::from(initial_input));
    let input_text_clone = input_text();
    let mut calc_result = use_signal(|| {
        expr_parser::expr()(&input_text_clone)
            .unwrap()
            .0
            .to_string()
    });

    let oninput = move |event: Event<FormData>| {
        input_text.set(event.value());
        let input_text = input_text();
        let parser = expr_parser::expr();
        let result = match parser(&input_text) {
            None => String::from("Error!"),
            Some((v, "")) => v.to_string(),
            Some((_, reminder)) => format!("Remains: {reminder}"),
        };
        calc_result.set(result);
    };

    rsx! {
         input {
           class: "input",
            value: input_text,
            oninput: oninput
        }
        div {
            { calc_result.to_string() }
        }
    }
}
