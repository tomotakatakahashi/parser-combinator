use parser_combinator::expr;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let result = use_state(|| 0);
    let text = use_state(|| "1 + (2 + 3) * 4".to_string());
    let text_cloned = (*text).clone();
    let onclick = {
        let result = result.clone();
        move |_| {
            //let text_value = (*text).clone();
            let evaluated = expr()(&text_cloned);
            match evaluated {
                None => result.set(123),
                Some((v, _)) => result.set(v),
            }
        }
    };

    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <input type="text" value={(*text).clone()}/>
            <p>{ *result }</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
