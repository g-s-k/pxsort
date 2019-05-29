#![recursion_limit = "2048"]

use pxsort::{Config, Heuristic, Shape};
use yew::{
    html,
    prelude::*,
    services::reader::{FileData, IBlob, ReaderService, ReaderTask},
};

fn main() {
    yew::initialize();
    App::<Root>::new().mount_to_body();
    yew::run_loop();
}

struct Root {
    link: ComponentLink<Self>,
    reader: ReaderService,
    tasks: Vec<ReaderTask>,
    cfg: Config,
    input: Option<(FileData, String)>,
    output: Option<String>,
}

enum Msg {
    ChooseFile(ChangeData),
    LoadedFile(FileData, String),
    DoSort,

    ChangeAngle(ChangeData),
    ToggleRotate,
    ChangeShapeType(ChangeData),

    ChangeSineAmplitude(ChangeData),
    ChangeSineLambda(ChangeData),
    ChangeSineOffset(ChangeData),
    ChangeEllipseEccentricity(ChangeData),
    ChangeEllipseCenterX(ChangeData),
    ChangeEllipseCenterY(ChangeData),

    ChangeFunction(ChangeData),
    ToggleReverse,

    ToggleAlpha,
    ChangeMin(ChangeData),
    ChangeMax(ChangeData),
    ToggleInvert,
}

impl Component for Root {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            reader: ReaderService::new(),
            tasks: vec![],
            cfg: Config::default(),
            input: None,
            output: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChooseFile(ChangeData::Files(files)) => {
                for file in files {
                    let mime = file.mime().unwrap_or("image/jpg".into());
                    self.tasks.push(
                        self.reader.read_file(
                            file,
                            self.link
                                .send_back(move |v| Msg::LoadedFile(v, mime.clone())),
                        ),
                    );
                }
                return false;
            }
            Msg::LoadedFile(fd, mime) => {
                let c = base64::encode(&fd.content);
                self.input = Some((fd, format!("data:{};base64,{}", mime, c)));
            }
            Msg::DoSort => {
                if let Some((fd, _)) = &self.input {
                    if let Ok(img) = image::load_from_memory(&fd.content) {
                        let mut buffer = Vec::new();
                        if let Ok(()) = self
                            .cfg
                            .sort(img)
                            .write_to(&mut buffer, image::ImageOutputFormat::PNG)
                        {
                            self.output =
                                Some(format!("data:image/png;base64,{}", base64::encode(&buffer)));
                        }
                    }
                }
            }
            Msg::ChangeAngle(ChangeData::Value(s)) => {
                if let Ok(v) = s.parse() {
                    self.cfg.angle = v;
                }
            }
            Msg::ToggleRotate => self.cfg.vertical ^= true,
            Msg::ChangeShapeType(ChangeData::Select(s)) => {
                if let Some(v) = s.value() {
                    if let Ok(t) = v.parse() {
                        self.cfg.path = t;
                    }
                }
            }
            Msg::ChangeSineAmplitude(ChangeData::Value(s)) => {
                if let (Shape::Sine { lambda, offset, .. }, Ok(amplitude)) =
                    (&self.cfg.path, s.parse())
                {
                    self.cfg.path = Shape::Sine {
                        amplitude,
                        lambda: *lambda,
                        offset: *offset,
                    };
                }
            }
            Msg::ChangeSineLambda(ChangeData::Value(s)) => {
                if let (
                    Shape::Sine {
                        amplitude, offset, ..
                    },
                    Ok(lambda),
                ) = (&self.cfg.path, s.parse())
                {
                    self.cfg.path = Shape::Sine {
                        amplitude: *amplitude,
                        lambda,
                        offset: *offset,
                    };
                }
            }
            Msg::ChangeSineOffset(ChangeData::Value(s)) => {
                if let (
                    Shape::Sine {
                        amplitude, lambda, ..
                    },
                    Ok(offset),
                ) = (&self.cfg.path, s.parse())
                {
                    self.cfg.path = Shape::Sine {
                        amplitude: *amplitude,
                        lambda: *lambda,
                        offset,
                    };
                }
            }
            Msg::ChangeEllipseEccentricity(ChangeData::Value(s)) => {
                if let (Shape::Ellipse { center, .. }, Ok(eccentricity)) =
                    (&self.cfg.path, s.parse())
                {
                    self.cfg.path = Shape::Ellipse {
                        eccentricity,
                        center: *center,
                    };
                }
            }
            Msg::ChangeEllipseCenterX(ChangeData::Value(s)) => {
                if let (
                    Shape::Ellipse {
                        eccentricity,
                        center: (_, cy),
                        ..
                    },
                    Ok(cx),
                ) = (&self.cfg.path, s.parse())
                {
                    self.cfg.path = Shape::Ellipse {
                        eccentricity: *eccentricity,
                        center: (cx, *cy),
                    };
                }
            }
            Msg::ChangeEllipseCenterY(ChangeData::Value(s)) => {
                if let (
                    Shape::Ellipse {
                        eccentricity,
                        center: (cx, _),
                        ..
                    },
                    Ok(cy),
                ) = (&self.cfg.path, s.parse())
                {
                    self.cfg.path = Shape::Ellipse {
                        eccentricity: *eccentricity,
                        center: (*cx, cy),
                    };
                }
            }
            Msg::ChangeFunction(ChangeData::Select(s)) => {
                if let Some(v) = s.value() {
                    if let Ok(t) = v.parse() {
                        self.cfg.function = t;
                    }
                }
            }
            Msg::ToggleReverse => self.cfg.reverse ^= true,
            Msg::ToggleAlpha => self.cfg.mask_alpha ^= true,
            Msg::ChangeMin(ChangeData::Value(s)) => {
                if let Ok(v) = s.parse() {
                    self.cfg.minimum = v;
                }
            }
            Msg::ChangeMax(ChangeData::Value(s)) => {
                if let Ok(v) = s.parse() {
                    self.cfg.maximum = v;
                }
            }
            Msg::ToggleInvert => self.cfg.invert ^= true,
            _ => return false,
        }

        true
    }
}

impl Renderable<Root> for Root {
    fn view(&self) -> Html<Self> {
        let path_shape = match self.cfg.path {
            Shape::Sine {
                amplitude,
                lambda,
                offset,
            } => html! {
                <>
                    <br />
                    <label>
                        {"Amplitude: "}
                        <input
                            type="number",
                            min="0",
                            max="1000",
                            value={amplitude},
                            onchange=|c| Msg::ChangeSineAmplitude(c),
                        />
                    </label>
                    <br />
                    <label>
                        {"Wavelength: "}
                        <input
                            type="number",
                            min="0",
                            max="1000",
                            value={lambda},
                            onchange=|c| Msg::ChangeSineLambda(c),
                        />
                    </label>
                    <br />
                    <label>
                        {"Offset: "}
                        <input
                            type="number",
                            min="0",
                            max="1000",
                            value={offset},
                            onchange=|c| Msg::ChangeSineOffset(c),
                        />
                    </label>
                </>
            },
            Shape::Ellipse {
                eccentricity,
                center: (cx, cy),
            } => html! {
                <>
                    <br />
                    <label>
                        {"Eccentricity: "}
                        <input
                            type="number",
                            step="0.01",
                            value={eccentricity},
                            onchange=|c| Msg::ChangeEllipseEccentricity(c),
                        />
                    </label>
                    <br />
                    <label>
                        {"Center X: "}
                        <input
                            type="number",
                            min="0",
                            max="1",
                            step="0.01",
                            value={cx},
                            onchange=|c| Msg::ChangeEllipseCenterX(c),
                        />
                    </label>
                    <br />
                    <label>
                        {"Center Y: "}
                        <input
                            type="number",
                            min="0",
                            max="1",
                            step="0.01",
                            value={cy},
                            onchange=|c| Msg::ChangeEllipseCenterY(c),
                        />
                    </label>
                </>
            },
            _ => html! { <></> },
        };

        html! {
            <>
                <header>{ "Pixel sort" }</header>
                <form class="controls", onsubmit="return false;", >
                    <label>
                        {"Upload a file: "}
                        <input
                            type="file",
                            accept="image/*",
                            onchange=|c| Msg::ChooseFile(c),
                        />
                    </label>
                    <fieldset>
                        <legend>{"Path"}</legend>
                        <label>
                            {"Angle: "}
                            <input
                                type="number",
                                min="-89",
                                max="89",
                                step="0.5",
                                value={self.cfg.angle},
                                onchange=|c| Msg::ChangeAngle(c),
                            />
                        </label>
                        <br />
                        <label>
                            <input
                                type="checkbox",
                                checked={self.cfg.vertical},
                                onchange=|_| Msg::ToggleRotate,
                            />
                            {"Rotate by an additional 90 degrees"}
                        </label>
                        <br />
                        <section>
                            <label>
                                {"Path shape: "}
                                <select onchange=|c| Msg::ChangeShapeType(c), >
                                    <option value="linear", >{"linear"}</option>
                                    <option value="sine", >{"sine"}</option>
                                    <option value="ellipse", >{"ellipse"}</option>
                                </select>
                            </label>
                        {path_shape}
                        </section>
                    </fieldset>
                    <fieldset>
                        <legend>{"Ordering"}</legend>
                        <label>
                            {"Comparison function: "}
                            <select onchange=|c| Msg::ChangeFunction(c), >
                                {for Heuristic::concrete_variants().map(|v| html! {
                                    <option value={v}, selected={v == self.cfg.function}, >{v}</option>
                                })}
                            </select>
                        </label>
                        <br />
                        <label>
                            <input
                                type="checkbox",
                                checked={self.cfg.reverse},
                                onchange=|_| Msg::ToggleReverse,
                            />
                            {"Reverse sort direction"}
                        </label>
                    </fieldset>
                    <fieldset>
                        <legend>{"Masking"}</legend>
                        <label>
                            <input
                                type="checkbox",
                                checked={self.cfg.mask_alpha},
                                onchange=|_| Msg::ToggleAlpha,
                            />
                            {"Exclude transparent pixels"}
                        </label>
                        <br />
                        <label>
                            {"Minimum value: "}
                            <input
                                type="number",
                                min="0",
                                max={self.cfg.maximum},
                                value={self.cfg.minimum},
                                onchange=|c| Msg::ChangeMin(c),
                            />
                        </label>
                        <br />
                        <label>
                            {"Maximum value: "}
                            <input
                                type="number",
                                min={self.cfg.minimum},
                                max="255",
                                value={self.cfg.maximum},
                                onchange=|c| Msg::ChangeMax(c),
                            />
                        </label>
                        <br />
                        <label>
                            <input
                                type="checkbox",
                                checked={self.cfg.invert},
                                onchange=|_| Msg::ToggleInvert,
                            />
                            {"Invert range"}
                        </label>
                    </fieldset>
                    <br />
                    <button onclick=|_| Msg::DoSort, >{"Sort some pixels!"}</button>
                </form>
                <div class="images", >
                    <img src={self.input.as_ref().map(|(_, s)| s).unwrap_or(&"".to_string())}, />
                    <img src={self.output.as_ref().unwrap_or(&"".to_string())}, />
                </div>
                <footer />
            </>
        }
    }
}
