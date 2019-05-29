#![recursion_limit = "2048"]

use pxsort::{Config, Heuristic};
use yew::{html, prelude::*};

fn main() {
    yew::initialize();
    App::<Root>::new().mount_to_body();
    yew::run_loop();
}

struct Root {
    cfg: Config,
}

enum Msg {
    AngleChange(ChangeData),
    RotateToggle,
}

impl Component for Root {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            cfg: Config::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AngleChange(ChangeData::Value(s)) => {
                if let Ok(v) = s.parse() {
                    self.cfg.angle = v;
                }

                true
            }
            Msg::RotateToggle => {
                self.cfg.vertical ^= true;

                true
            }
            _ => false,
        }
    }
}

impl Renderable<Root> for Root {
    fn view(&self) -> Html<Self> {
        html! {
            <>
                <header>{ "Pixel sort" }</header>
                <form class="controls", onsubmit="return false;", >
                    <label>
                        {"Upload a file: "}
                        <input type="file", accept="image/*", />
                    </label>
                    <fieldset>
                        <legend>{"Path"}</legend>
                        <label>
                            {"Angle: "}
                            <input
                                type="number",
                                min="-89",
                                max="89",
                                value={self.cfg.angle},
                                onchange=|c| Msg::AngleChange(c),
                            />
                        </label>
                        <br />
                        <label>
                            <input
                                type="checkbox",
                                checked={self.cfg.vertical},
                                onchange=|_| Msg::RotateToggle,
                            />
                            {"Rotate by an additional 90 degrees"}
                        </label>
                        <br />
                        <section>
                            <label>
                                {"Path shape: "}
                                <select>
                                    <option value="linear", >{"linear"}</option>
                                    <option value="sine", >{"sine"}</option>
                                    <option value="ellipse", >{"ellipse"}</option>
                                </select>
                            </label>
                        // TODO: add conditional inputs for path params
                        </section>
                    </fieldset>
                    <fieldset>
                        <legend>{"Ordering"}</legend>
                        <label>
                            {"Comparison function: "}
                            <select>
                                {for Heuristic::concrete_variants().map(|v| html! {
                                    <option value={v}, selected={v == self.cfg.function}, >{v}</option>
                                })}
                            </select>
                        </label>
                        <br />
                        <label>
                            <input type="checkbox", checked={self.cfg.reverse}, />
                            {"Reverse sort direction"}
                        </label>
                    </fieldset>
                    <fieldset>
                        <legend>{"Masking"}</legend>
                        <label>
                            <input type="checkbox", checked={self.cfg.mask_alpha}, />
                            {"Exclude transparent pixels"}
                        </label>
                        <br />
                        <label>
                            {"Minimum value: "}
                            <input type="number", min="0", max="255", step="5", value={self.cfg.minimum}, />
                        </label>
                        <br />
                        <label>
                            {"Maximum value: "}
                            <input type="number", min="0", max="255", step="5", value={self.cfg.maximum}, />
                        </label>
                        <br />
                        <label>
                            <input type="checkbox", checked={self.cfg.invert}, />
                            {"Invert range"}
                        </label>
                    </fieldset>
                    <br />
                    <label>{"Progress: "}<progress /></label>
                </form>
                <div class="images", ><img /><img /></div>
                <footer />
            </>
        }
    }
}
