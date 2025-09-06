use dioxus::prelude::*;
mod expr_parser;

const MAIN_CSS: Asset = asset!("/assets/bulma.min.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        h1 { class: "title is-1", "Parser Combinator demo" }
        p { class: "block",
            "A simple calculator that only supports non-negative integers, parenthesis, addition, and multiplication."
        }
        p { class: "block", "This app is created to demonstrate the parser combinator library." }
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
        div { class: "field is-horizontal",
            div { class: "field-label is-normal",
                label { class: "label", "Input" }
            }
            div { class: "field-body",
                div { class: "field",
                    input { class: "input", value: input_text, oninput }
                }
            }
        }
        div { class: "field is-horizontal",
            div { class: "field-label is-normal",
                div { class: "label",
                    label { class: "label", "Result" }
                }
            }
            div { class: "field-body",
                div { class: "field",
                    input {
                        class: "input is-static",
                        readonly: true,
                        value: calc_result.to_string(),
                    }
                }
            }
        }
    }
}
