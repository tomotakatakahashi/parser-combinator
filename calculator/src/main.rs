use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Ipt {}
    }
}

#[component]
fn Ipt() -> Element {
    let mut input_text = use_signal(|| String::from("1 + (2 + 3) * 4"));

    let oninput = move |event: Event<FormData>| {
        input_text.set(event.value() + "aaa");
    };

    rsx! {
    input {
        class: "input",
        value: input_text,
        oninput: oninput
    }
    }
}
