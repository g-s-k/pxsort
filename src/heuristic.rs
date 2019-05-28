use image::Rgba;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

fn pixel_max(Rgba { data, .. }: &Rgba<u8>) -> u8 {
    data[..3].iter().max().cloned().unwrap_or_default()
}

fn pixel_min(Rgba { data, .. }: &Rgba<u8>) -> u8 {
    data[..3].iter().min().cloned().unwrap_or_default()
}

fn pixel_chroma(pixel: &Rgba<u8>) -> u8 {
    pixel_max(pixel) - pixel_min(pixel)
}

fn pixel_hue(pixel: &Rgba<u8>) -> u8 {
    let c = pixel_chroma(pixel);

    if c == 0 {
        return 0;
    }

    let Rgba { data, .. } = pixel;

    match data[..3].iter().enumerate().max_by_key(|&(_, e)| e) {
        Some((0, _)) => (data[1] as i16 - data[2] as i16).abs() as u8 / c * 43,
        Some((1, _)) => (data[2] as i16 - data[0] as i16).abs() as u8 / c * 43 + 85,
        Some((2, _)) => (data[0] as i16 - data[1] as i16).abs() as u8 / c * 43 + 171,
        _ => 0,
    }
}

fn pixel_saturation(pixel: &Rgba<u8>) -> u8 {
    match pixel_max(pixel) {
        0 => 0,
        v => pixel_chroma(pixel) / v,
    }
}

fn pixel_brightness(Rgba { data, .. }: &Rgba<u8>) -> u8 {
    data[0] / 3 + data[1] / 3 + data[2] / 3 + (data[0] % 3 + data[1] % 3 + data[2] % 3) / 3
}

/// https://stackoverflow.com/a/596241
fn pixel_luma(Rgba { data, .. }: &Rgba<u8>) -> u8 {
    ((data[0] as u16 * 2 + data[1] as u16 + data[2] as u16 * 4) >> 3) as u8
}

/// Basis to use for sorting individual pixels.
#[allow(non_camel_case_types)]
#[derive(EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Heuristic {
    Luma,
    Brightness,
    Max,
    Min,
    Chroma,
    Hue,
    Saturation,
    Value,
    Red,
    Blue,
    Green,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Heuristic {
    pub(crate) fn variants() -> Vec<&'static str> {
        Self::iter()
            .filter_map(|v| {
                if let Heuristic::__Nonexhaustive = v {
                    None
                } else {
                    Some(v)
                }
            })
            .map(Into::into)
            .collect()
    }

    /// Get the key extraction function for this heuristic.
    pub fn func(&self) -> Box<Fn(&Rgba<u8>) -> u8> {
        match self {
            Heuristic::Red => Box::new(|Rgba { data, .. }| data[0]),
            Heuristic::Green => Box::new(|Rgba { data, .. }| data[1]),
            Heuristic::Blue => Box::new(|Rgba { data, .. }| data[2]),
            Heuristic::Max => Box::new(pixel_max),
            Heuristic::Min => Box::new(pixel_min),
            Heuristic::Chroma => Box::new(pixel_chroma),
            Heuristic::Hue => Box::new(pixel_hue),
            Heuristic::Saturation => Box::new(pixel_saturation),
            Heuristic::Value => Box::new(pixel_max),
            Heuristic::Brightness => Box::new(pixel_brightness),
            Heuristic::Luma => Box::new(pixel_luma),
            Heuristic::__Nonexhaustive => unreachable!(),
        }
    }
}