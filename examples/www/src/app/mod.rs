use pxsort::Heuristic;
use yew::{prelude::*, html};

const HEURISTIC_ID: &str = "heuristic-options-list";

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        App
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        let heuristic_options = Heuristic::variants().into_iter().map(|v| html! {
            <option value={v}, />
        });

        html! {
            <>
                <header>{ "Pixel sort" }</header>
                <form class="controls", onsubmit="return false;", >
                    <input list={HEURISTIC_ID}, />
                    <datalist id={HEURISTIC_ID}, >
                        { for heuristic_options }
                    </datalist>
                </form>
                <div class="images", ><img /><img /></div>
                <footer />
            </>
        }
    }
}
