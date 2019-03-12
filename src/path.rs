use std::str::FromStr;

const DEFAULT_AMP: f32 = 25.0;
const DEFAULT_LAMBDA: f32 = 50.0;
const DEFAULT_CENTER: (f32, f32) = (0.5, 0.5);

const DEFAULT_SINE: PathShape = PathShape::Sine {
    amplitude: DEFAULT_AMP,
    lambda: DEFAULT_LAMBDA,
    offset: 0.0,
};
const DEFAULT_ELL: PathShape = PathShape::Ellipse {
    eccentricity: 0.0,
    center: DEFAULT_CENTER,
};

pub enum PathShape {
    Linear,
    Sine {
        amplitude: f32,
        lambda: f32,
        offset: f32,
    },
    Ellipse {
        eccentricity: f32,
        center: (f32, f32),
    },
}

impl Default for PathShape {
    fn default() -> Self {
        PathShape::Linear
    }
}

fn unwrap_parens(s: &str) -> Result<&str, ()> {
    let st = s.trim();

    if st.starts_with('(') && st.ends_with(')')
        || st.starts_with('[') && st.ends_with(']')
        || st.starts_with('{') && st.ends_with('}')
        || st.starts_with('<') && st.ends_with('>')
    {
        Ok(&st[1..st.len() - 1])
    } else {
        Err(())
    }
}

impl FromStr for PathShape {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "" | "line" | "linear" => Ok(PathShape::Linear),
            "sine" => Ok(DEFAULT_SINE),
            "circle" => Ok(DEFAULT_ELL),
            st => {
                let err_msg = format!("Could not parse `{}` as a valid path shape", st);

                if st.starts_with("sine") {
                    let args = unwrap_parens(&st[4..])
                        .map_err(|_| err_msg.clone())?
                        .split(',')
                        .map(|a| a.trim().parse::<f32>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| err_msg.clone())?;
                    match args.len() {
                        0 => Ok(DEFAULT_SINE),
                        1 => Ok(PathShape::Sine {
                            amplitude: args[0],
                            lambda: DEFAULT_LAMBDA,
                            offset: 0.0,
                        }),
                        2 => Ok(PathShape::Sine {
                            amplitude: args[0],
                            lambda: args[1],
                            offset: 0.0,
                        }),
                        3 => Ok(PathShape::Sine {
                            amplitude: args[0],
                            lambda: args[1],
                            offset: args[2],
                        }),
                        _ => Err(err_msg),
                    }
                } else if st.starts_with("circle") {
                    let args = unwrap_parens(&st[6..])
                        .map_err(|_| err_msg.clone())?
                        .split(',')
                        .map(|a| a.trim().parse::<f32>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| err_msg.clone())?;
                    match args.len() {
                        0 => Ok(DEFAULT_ELL),
                        2 => Ok(PathShape::Ellipse {
                            eccentricity: 0.0,
                            center: (args[0], args[1]),
                        }),
                        _ => Err(err_msg),
                    }
                } else if st.starts_with("ellipse") {
                    let args = unwrap_parens(&st[7..])
                        .map_err(|_| err_msg.clone())?
                        .split(',')
                        .map(|a| a.trim().parse::<f32>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| err_msg.clone())?;
                    match args.len() {
                        0 => Ok(DEFAULT_ELL),
                        1 => Ok(PathShape::Ellipse {
                            eccentricity: args[0],
                            center: DEFAULT_CENTER,
                        }),
                        2 => Ok(PathShape::Ellipse {
                            eccentricity: 0.0,
                            center: (args[0], args[1]),
                        }),
                        3 => Ok(PathShape::Ellipse {
                            eccentricity: args[0],
                            center: (args[1], args[2]),
                        }),
                        _ => Err(err_msg),
                    }
                } else {
                    Err(err_msg)
                }
            }
        }
    }
}
