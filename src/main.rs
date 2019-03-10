use std::path::PathBuf;

use image::{DynamicImage, ImageError};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(try_from_str))]
    file: PathBuf,
    #[structopt(short, parse(try_from_str))]
    output: Option<PathBuf>,
}

fn main() -> Result<(), ImageError> {
    let cli = Cli::from_args();

    let mut img = image::open(&cli.file)?;

    match &mut img {
        DynamicImage::ImageRgb8(rgb) => {
            let w = rgb.width();
            for (idx_y, row) in rgb
                .clone()
                .pixels_mut()
                .collect::<Vec<_>>()
                .chunks_mut(w as usize)
                .enumerate()
            {
                row.sort_by(|left, right| left.data[0].cmp(&right.data[0]));

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
