use std::path::PathBuf;

use image::ImageError;
use structopt::StructOpt;

use pxsort::Config;

#[derive(StructOpt)]
#[structopt(about = "Sort the pixels in an image")]
#[structopt(raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(rename_all = "kebab-case")]
struct Cli {
    /// Input file
    #[structopt(parse(try_from_str))]
    file: PathBuf,
    /// Output file
    #[structopt(short, long = "out", parse(try_from_str))]
    output: Option<PathBuf>,
    #[structopt(flatten)]
    config: Config,
}

fn main() -> Result<(), ImageError> {
    let cli = Cli::from_args();

    eprintln!("Opening image at {:?}", cli.file);
    let img = image::open(&cli.file)?;

    let img_out = cli.config.sort(img);

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
