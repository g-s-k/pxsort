use std::path::PathBuf;

use image::{DynamicImage, ImageError, Rgba};
use structopt::{
    clap::{_clap_count_exprs, arg_enum},
    StructOpt,
};

arg_enum! {
    enum SortHeuristic {
        Luma,
        Brightness,
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

#[derive(StructOpt)]
#[structopt(about = "Sort the pixels in an image")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
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
    /// Sort vertically instead of horizontally
    #[structopt(short)]
    vertical: bool,
}

fn main() -> Result<(), ImageError> {
    let cli = Cli::from_args();

    let mut img = image::open(&cli.file)?;

    if cli.vertical {
        img = img.rotate90();
    }

    let mut rgba = img.to_rgba();

    let w = rgba.width() as usize;
    for (idx_y, row) in rgba
        .clone()
        .pixels_mut()
        .collect::<Vec<_>>()
        .chunks_mut(w)
        .enumerate()
    {
        let sort_fn = cli.function.func();

        let mut ctr = 0;
        while ctr < w {
            // find the end of the current "good" sequence
            let numel = row[ctr..]
                .iter()
                .take_while(|p| {
                    let l = sort_fn(p);
                    (l >= cli.minimum && l <= cli.maximum) != cli.invert
                })
                .count();

            // sort
            row[ctr..ctr + numel].sort_unstable_by(|l, r| {
                if cli.reverse {
                    sort_fn(r).cmp(&sort_fn(l))
                } else {
                    sort_fn(l).cmp(&sort_fn(r))
                }
            });

            ctr += numel;

            // continue until another value in the right range appears
            ctr += row[ctr..]
                .iter()
                .take_while(|p| {
                    let l = sort_fn(p);
                    (l < cli.minimum || l > cli.maximum) != cli.invert
                })
                .count();
        }

        for (idx_x, px) in row.iter().enumerate() {
            rgba.put_pixel(idx_x as u32, idx_y as u32, **px);
        }
    }

    let mut img_out = DynamicImage::ImageRgba8(rgba);

    if cli.vertical {
        img_out = img_out.rotate270();
    }

    if let Some(p) = cli.output {
        img_out.save(p)?;
    } else {
        match (
            cli.file.parent(),
            cli.file.file_stem(),
            cli.file.extension(),
        ) {
            (None, _, _) | (_, None, _) | (_, _, None) => (),
            (Some(p), Some(b), Some(e)) => {
                let mut fname = b.to_owned();
                fname.push("_1.");
                fname.push(e);
                let mut pth = p.to_owned();
                pth.push(fname);
                img_out.save(pth)?;
            }
        }
    }

    Ok(())
}
