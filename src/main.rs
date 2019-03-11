use std::path::PathBuf;

use image::{DynamicImage, ImageError, Rgb};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(try_from_str))]
    file: PathBuf,
    #[structopt(short, parse(try_from_str))]
    output: Option<PathBuf>,
    #[structopt(short, default_value = "0")]
    minimum: u8,
    #[structopt(short = "x", default_value = "255")]
    maximum: u8,
}

fn luma(Rgb { data, .. }: &Rgb<u8>) -> u8 {
    // https://stackoverflow.com/a/596241
    ((data[0] as u16 * 2 + data[1] as u16 + data[2] as u16 * 4) >> 3) as u8
}

fn main() -> Result<(), ImageError> {
    let cli = Cli::from_args();

    let mut img = image::open(&cli.file)?;

    match &mut img {
        DynamicImage::ImageRgb8(rgb) => {
            let w = rgb.width() as usize;
            for (idx_y, row) in rgb
                .clone()
                .pixels_mut()
                .collect::<Vec<_>>()
                .chunks_mut(w)
                .enumerate()
            {
                let mut ctr = 0;
                while ctr < w {
                    let numel = row[ctr..]
                        .iter()
                        .take_while(|p| {
                            let l = luma(p);
                            l >= cli.minimum && l <= cli.maximum
                        })
                        .count();
                    row[ctr..ctr + numel].sort_unstable_by(|left, right| luma(left).cmp(&luma(right)));
                    ctr += numel;
                    ctr += row[ctr..]
                        .iter()
                        .take_while(|p| {
                            let l = luma(p);
                            l < cli.minimum || l > cli.maximum
                        })
                        .count();
                }

                for (idx_x, px) in row.iter().enumerate() {
                    rgb.put_pixel(idx_x as u32, idx_y as u32, **px);
                }
            }
        }
        _ => (),
    }

    if let Some(p) = cli.output {
        img.save(p)?;
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
                img.save(pth)?;
            }
        }
    }

    Ok(())
}
