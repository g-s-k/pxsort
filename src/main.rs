use std::path::PathBuf;

use image::{DynamicImage, ImageError, Rgba};
use indicatif::{ProgressBar, ProgressStyle};
use structopt::{
    clap::{_clap_count_exprs, arg_enum},
    StructOpt,
};

const BUFFER_PIXELS: i64 = 100;

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

arg_enum! {
    enum SortHeuristic {
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
    }
}

impl SortHeuristic {
    fn func(&self) -> Box<Fn(&Rgba<u8>) -> u8> {
        match self {
            SortHeuristic::Red => Box::new(|Rgba { data, .. }| data[0]),
            SortHeuristic::Green => Box::new(|Rgba { data, .. }| data[1]),
            SortHeuristic::Blue => Box::new(|Rgba { data, .. }| data[2]),
            SortHeuristic::Max => Box::new(pixel_max),
            SortHeuristic::Min => Box::new(pixel_min),
            SortHeuristic::Chroma => Box::new(pixel_chroma),
            SortHeuristic::Hue => Box::new(pixel_hue),
            SortHeuristic::Saturation => Box::new(|p| match pixel_max(p) {
                0 => 0,
                v => pixel_chroma(p) / v,
            }),
            SortHeuristic::Value => Box::new(pixel_max),
            SortHeuristic::Brightness => Box::new(|Rgba { data, .. }| {
                data[0] / 3
                    + data[1] / 3
                    + data[2] / 3
                    + (data[0] % 3 + data[1] % 3 + data[2] % 3) / 3
            }),
            SortHeuristic::Luma => Box::new(|Rgba { data, .. }| {
                // https://stackoverflow.com/a/596241
                ((data[0] as u16 * 2 + data[1] as u16 + data[2] as u16 * 4) >> 3) as u8
            }),
        }
    }
}

arg_enum! {
    enum PathShape {
        Line,
        Sine,
        Ellipse,
    }
}

fn check_angle(angle: String) -> Result<(), String> {
    let ang = angle
        .parse::<f32>()
        .map_err(|_| "Could not parse as a number".to_string())?;
    if ang >= 90.0 || ang <= -90.0 {
        Err("Rotation angle must be between -90 and +90 degrees".to_string())
    } else {
        Ok(())
    }
}

#[derive(StructOpt)]
#[structopt(about = "Sort the pixels in an image")]
#[structopt(raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(rename_all = "kebab-case")]
struct Cli {
    /// Input file
    #[structopt(parse(try_from_str))]
    file: PathBuf,
    /// Output file
    #[structopt(short, parse(try_from_str))]
    output: Option<PathBuf>,
    /// Minimum value to sort
    #[structopt(short, default_value = "0")]
    minimum: u8,
    /// Maximum value to sort
    #[structopt(short = "x", default_value = "255")]
    maximum: u8,
    /// Sort heuristic to use
    #[structopt(
        short,
        default_value = "luma",
        raw(
            possible_values = "&SortHeuristic::variants()",
            case_insensitive = "true",
            set = "structopt::clap::ArgSettings::NextLineHelp"
        )
    )]
    function: SortHeuristic,
    /// Reverse the sort direction
    #[structopt(short)]
    reverse: bool,
    /// Sort outside specified range rather than inside
    #[structopt(short)]
    invert: bool,
    /// Rotate the sort path by 90 degrees
    #[structopt(short, group = "rotate")]
    vertical: bool,
    /// Don't sort pixels that have zero alpha
    #[structopt(long)]
    mask_alpha: bool,
    /// Rotate the sort path by a custom angle
    #[structopt(short, default_value = "0", raw(validator = "check_angle"))]
    angle: f32,
    /// Path shape to traverse the image
    #[structopt(
        short,
        long,
        default_value = "line",
        raw(
            case_insensitive = "true",
            possible_values = "&PathShape::variants()",
            set = "structopt::clap::ArgSettings::NextLineHelp"
        )
    )]
    path: PathShape,
    /// Parameters to modify the path
    #[structopt(long)]
    params: Vec<String>,
}

fn do_sort(cli: &Cli, pixels: &mut [&Rgba<u8>]) {
    let sort_fn = cli.function.func();
    let mask_fn = |p: &Rgba<u8>| !(cli.mask_alpha && p.data[3] == 0);

    let mut ctr = 0;
    while ctr < pixels.len() as usize {
        // find the end of the current "good" sequence
        let numel = pixels[ctr..]
            .iter()
            .take_while(|p| {
                let l = sort_fn(p);
                (l >= cli.minimum && l <= cli.maximum) != cli.invert && mask_fn(p)
            })
            .count();

        // sort
        pixels[ctr..ctr + numel].sort_unstable_by(|l, r| {
            if cli.reverse {
                sort_fn(r).cmp(&sort_fn(l))
            } else {
                sort_fn(l).cmp(&sort_fn(r))
            }
        });

        ctr += numel;

        // continue until another value in the right range appears
        ctr += pixels[ctr..]
            .iter()
            .take_while(|p| {
                let l = sort_fn(p);
                (l < cli.minimum || l > cli.maximum) != cli.invert || !mask_fn(p)
            })
            .count();
    }
}

fn main() -> Result<(), ImageError> {
    let cli = Cli::from_args();

    eprintln!("Opening image at {:?}", cli.file);
    let mut img = image::open(&cli.file)?;

    if cli.vertical {
        img = img.rotate90();
    }

    let mut rgba = img.to_rgba();
    let (w, h) = rgba.dimensions();

    let prog = ProgressBar::new(h as u64);
    prog.set_style(ProgressStyle::default_bar().template("{prefix} {wide_bar} {pos:>4}/{len}"));

    match cli.path {
        PathShape::Ellipse => unimplemented!(),
        PathShape::Sine => {
            let amplitude = cli
                .params
                .iter()
                .find(|s| s.starts_with("amp="))
                .map(|s| &s[4..])
                .unwrap_or_default()
                .parse::<f32>()
                .unwrap_or(50.);
            let lambda = cli
                .params
                .iter()
                .find(|s| s.starts_with("period="))
                .map(|s| &s[7..])
                .unwrap_or_default()
                .parse::<f32>()
                .unwrap_or(180. / std::f32::consts::PI);
            let shift = cli
                .params
                .iter()
                .find(|s| s.starts_with("offset="))
                .map(|s| &s[7..])
                .unwrap_or_default()
                .parse::<f32>()
                .unwrap_or_default();

            let tan = cli.angle.to_radians().tan();
            let extra_height = (w as f32 / tan).floor() as i64;
            let range = if extra_height > 0 {
                -(extra_height + BUFFER_PIXELS)..(h as i64)
            } else {
                0..(h as i64 - extra_height + BUFFER_PIXELS)
            };

            prog.set_draw_delta((h as u64 + extra_height.abs() as u64) / 50);
            prog.set_prefix("Sorting rows:");
            prog.tick();

            let rgba_c = rgba.clone();
            for row_idx in range {
                let idxes = (0..w)
                    .into_iter()
                    .map(|xv| {
                        (
                            xv,
                            ((xv as f32 * tan + row_idx as f32)
                                + (xv as f32 / lambda + shift).sin() * amplitude)
                                as u32,
                        )
                    })
                    .filter(|(_, y)| *y > 0 && *y < h)
                    .collect::<Vec<_>>();
                let mut pixels = idxes
                    .iter()
                    .map(|(x, y)| rgba_c.get_pixel(*x, *y))
                    .collect::<Vec<_>>();
                do_sort(&cli, &mut pixels[..]);

                for ((idx_x, idx_y), px) in idxes.iter().zip(pixels.iter()) {
                    rgba.put_pixel(*idx_x, *idx_y, **px);
                }

                prog.inc(1);
            }
        }
        PathShape::Line if cli.angle != 0.0 => {
            let tan = cli.angle.to_radians().tan();
            let extra_height = (w as f32 / tan).floor() as i64;
            let range = if extra_height > 0 {
                -extra_height..(h as i64)
            } else {
                0..(h as i64 - extra_height)
            };

            prog.set_draw_delta((h as u64 + extra_height.abs() as u64) / 50);
            prog.set_prefix("Sorting rows:");
            prog.tick();

            let rgba_c = rgba.clone();
            for row_idx in range {
                let idxes = (0..w)
                    .into_iter()
                    .map(|xv| (xv, (xv as f32 * tan + row_idx as f32) as u32))
                    .filter(|(_, y)| *y > 0 && *y < h)
                    .collect::<Vec<_>>();
                let mut pixels = idxes
                    .iter()
                    .map(|(x, y)| rgba_c.get_pixel(*x, *y))
                    .collect::<Vec<_>>();
                do_sort(&cli, &mut pixels[..]);

                for ((idx_x, idx_y), px) in idxes.iter().zip(pixels.iter()) {
                    rgba.put_pixel(*idx_x, *idx_y, **px);
                }

                prog.inc(1);
            }
        }
        PathShape::Line => {
            prog.set_draw_delta(h as u64 / 50);
            prog.set_prefix(&format!(
                "Sorting {}:",
                if cli.vertical { "columns" } else { "rows" }
            ));
            prog.tick();

            for (idx_y, row) in rgba
                .clone()
                .pixels()
                .collect::<Vec<_>>()
                .chunks_mut(w as usize)
                .enumerate()
            {
                do_sort(&cli, &mut row[..]);

                for (idx_x, px) in row.iter().enumerate() {
                    rgba.put_pixel(idx_x as u32, idx_y as u32, **px);
                }

                prog.inc(1);
            }
        }
    }

    prog.finish_with_message("Done sorting!");

    let mut img_out = DynamicImage::ImageRgba8(rgba);

    if cli.vertical {
        img_out = img_out.rotate270();
    }

    let file_out = if let Some(p) = cli.output {
        p
    } else {
        match (
            cli.file.parent(),
            cli.file.file_stem(),
            cli.file.extension(),
        ) {
            (None, _, _) | (_, None, _) | (_, _, None) => panic!("Invalid filename"),
            (Some(p), Some(b), Some(e)) => {
                let mut fname = b.to_owned();
                fname.push("_1.");
                fname.push(e);
                let mut pth = p.to_owned();
                pth.push(fname);
                pth
            }
        }
    };

    eprintln!("Saving file to {:?}", file_out);
    img_out.save(file_out)?;

    Ok(())
}
