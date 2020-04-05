//! Sort pixels in an image.
#![warn(clippy::pedantic)]

use image::{DynamicImage, Rgba};
#[cfg(not(target_arch = "wasm32"))]
use indicatif::{ProgressBar, ProgressStyle};
use structopt::StructOpt;

mod heuristic;
mod path;

pub use heuristic::Heuristic;
pub use path::Shape;

#[allow(clippy::needless_pass_by_value)]
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

/// Sorting configuration.
///
/// Includes how to traverse the pixel grid, which regions of the image to skip,
/// and what metric to sort by.
#[derive(StructOpt)]
pub struct Config {
    /// Minimum value to sort
    #[structopt(short, long = "min", default_value = "0")]
    pub minimum: u8,
    /// Maximum value to sort
    #[structopt(short = "x", long = "max", default_value = "255")]
    pub maximum: u8,
    /// Sort heuristic to use
    #[structopt(
        short,
        long,
        default_value = "luma",
        raw(
            possible_values = "&Heuristic::variants()",
            case_insensitive = "true",
            set = "structopt::clap::ArgSettings::NextLineHelp"
        )
    )]
    pub function: Heuristic,
    /// Reverse the sort direction
    #[structopt(short, long)]
    pub reverse: bool,
    /// Sort outside specified range rather than inside
    #[structopt(short, long)]
    pub invert: bool,
    /// Rotate the sort path by 90 degrees
    #[structopt(short, long)]
    pub vertical: bool,
    /// Don't sort pixels that have zero alpha
    #[structopt(short = "k", long)]
    pub mask_alpha: bool,
    /// Rotate the sort path by a custom angle
    #[structopt(short, long, default_value = "0", raw(validator = "check_angle"))]
    pub angle: f32,
    /// Path shape to traverse the image
    #[structopt(
        short,
        long,
        default_value = "line",
        raw(set = "structopt::clap::ArgSettings::NextLineHelp")
    )]
    pub path: Shape,
    #[structopt(raw(hidden = "true"))]
    __: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            minimum: 0,
            maximum: 255,
            function: Heuristic::Luma,
            reverse: false,
            invert: false,
            vertical: false,
            mask_alpha: false,
            angle: 0.0,
            path: Shape::Linear,
            __: false,
        }
    }
}

impl Config {
    fn do_sort(&self, pixels: &mut [&Rgba<u8>]) {
        let sort_fn = self.function.func();
        let mask_fn = |p: &Rgba<u8>| !(self.mask_alpha && p.data[3] == 0);

        let mut ctr = 0;
        while ctr < pixels.len() as usize {
            // find the end of the current "good" sequence
            let numel = pixels[ctr..]
                .iter()
                .take_while(|p| {
                    let l = sort_fn(p);
                    (l >= self.minimum && l <= self.maximum) != self.invert && mask_fn(p)
                })
                .count();

            // sort
            pixels[ctr..ctr + numel].sort_unstable_by(|l, r| {
                if self.reverse {
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
                    (l < self.minimum || l > self.maximum) != self.invert || !mask_fn(p)
                })
                .count();
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::too_many_lines
    )]
    #[must_use]
    /// Sort pixels according to configured settings and return a new image.
    pub fn sort(&self, mut img: DynamicImage) -> DynamicImage {
        if self.vertical {
            img = img.rotate90();
        }

        let mut rgba = img.to_rgba();
        let (w, h) = rgba.dimensions();

        #[cfg(not(target_arch = "wasm32"))]
        let prog = {
            let p = ProgressBar::new(u64::from(h));
            p.set_style(
                ProgressStyle::default_bar().template("{prefix} {wide_bar} {pos:>5}/{len}"),
            );
            p
        };

        match self.path {
            Shape::Ellipse {
                eccentricity,
                center: (x_center, y_center),
            } => {
                let (c_x, c_y, diag) = (
                    (w as f32 * x_center).floor() as u32,
                    (h as f32 * y_center).floor() as u32,
                    (w as f32).hypot(h as f32).floor() as u32,
                );
                let n_shells = diag * 5 * (1. + eccentricity).powi(2).floor() as u32;

                #[cfg(not(target_arch = "wasm32"))]
                {
                    prog.set_prefix("Sorting rings:");
                    prog.set_length(u64::from(n_shells));
                    prog.set_draw_delta(u64::from(n_shells) / 50);
                }

                let cos = self.angle.to_radians().cos();
                let sin = self.angle.to_radians().sin();

                let rgba_c = rgba.clone();
                for a in (0..n_shells).rev().map(|da| (da as f32) / 5.) {
                    let b_sq = a.powi(2) * (1. - eccentricity.powi(2));
                    let c = (a.powi(2) - b_sq).sqrt();
                    let peri = (std::f32::consts::PI * 2. * ((a.powi(2) + b_sq) / 2.).sqrt())
                        .floor() as usize;
                    let mut idxes = (0..peri * 3)
                        .map(|dt| dt as f32 / 3.)
                        .map(|dt| (dt * 360. / (peri as f32)).to_radians())
                        .map(|theta| (b_sq / a / (1. - eccentricity * theta.cos()), theta))
                        .map(|(r, theta)| (r * theta.cos() - c, r * theta.sin()))
                        .map(|(x, y)| (x * cos - y * sin, y * cos + x * sin))
                        .map(|(x, y)| (x + c_x as f32, y + c_y as f32))
                        .filter_map(|(x, y)| {
                            if x >= 0. && x < w as f32 && y >= 0. && y < h as f32 {
                                Some((x.floor() as u32, y.floor() as u32))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    idxes.dedup();

                    let mut pixels = idxes
                        .iter()
                        .map(|(x, y)| rgba_c.get_pixel(*x, *y))
                        .collect::<Vec<_>>();
                    self.do_sort(&mut pixels[..]);

                    for ((idx_x, idx_y), px) in idxes.iter().zip(pixels.iter()) {
                        rgba.put_pixel(*idx_x, *idx_y, **px);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    prog.inc(1);
                }
            }
            Shape::Sine {
                amplitude,
                lambda,
                offset,
            } => {
                let (c_x, c_y, diag) = (
                    (w as f32 * 0.5).floor(),
                    (h as f32 * 0.5).floor(),
                    (w as f32).hypot(h as f32).floor() as u32,
                );

                #[cfg(not(target_arch = "wasm32"))]
                {
                    prog.set_prefix("Sorting rows:");
                    prog.set_length(u64::from(h + diag) * 3);
                    prog.set_draw_delta(u64::from(h + diag) * 3 / 50);
                }

                let ang = self.angle.to_radians();
                let (sin, cos) = (ang.sin(), ang.cos());

                let rgba_c = rgba.clone();
                for row_idx in 0..(diag * 3) {
                    let idxes = (0..diag)
                        .map(|x| x as f32)
                        .map(|x| {
                            (
                                x,
                                row_idx as f32 / 3. + (x / lambda + offset).sin() * amplitude,
                            )
                        })
                        .map(|(x, y)| (x - diag as f32 / 2., y - diag as f32 / 2.))
                        .map(|(x, y)| (x * cos - y * sin, y * cos + x * sin))
                        .map(|(x, y)| (x + c_x, y + c_y))
                        .filter_map(|(x, y)| {
                            if x >= 0. && x < w as f32 && y >= 0. && y < h as f32 {
                                Some((x.floor() as u32, y.floor() as u32))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let mut pixels = idxes
                        .iter()
                        .map(|(x, y)| rgba_c.get_pixel(*x, *y))
                        .collect::<Vec<_>>();
                    self.do_sort(&mut pixels[..]);

                    for ((idx_x, idx_y), px) in idxes.iter().zip(pixels.iter()) {
                        rgba.put_pixel(*idx_x, *idx_y, **px);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    prog.inc(1);
                }
            }
            Shape::Linear if self.angle != 0.0 => {
                let tan = self.angle.to_radians().tan();
                let extra_height = (tan * w as f32).floor() as i64;
                let range = if extra_height > 0 {
                    -extra_height..i64::from(h)
                } else {
                    0..(i64::from(h) - extra_height)
                };

                #[cfg(not(target_arch = "wasm32"))]
                {
                    prog.set_prefix("Sorting rows:");
                    prog.set_draw_delta((u64::from(h) + extra_height.abs() as u64) / 50);
                }

                let rgba_c = rgba.clone();
                for row_idx in range {
                    let idxes = (0..w)
                        .map(|xv| (xv, (xv as f32 * tan + row_idx as f32) as u32))
                        .filter(|(_, y)| *y > 0 && *y < h)
                        .collect::<Vec<_>>();
                    let mut pixels = idxes
                        .iter()
                        .map(|(x, y)| rgba_c.get_pixel(*x, *y))
                        .collect::<Vec<_>>();
                    self.do_sort(&mut pixels[..]);

                    for ((idx_x, idx_y), px) in idxes.iter().zip(pixels.iter()) {
                        rgba.put_pixel(*idx_x, *idx_y, **px);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    prog.inc(1);
                }
            }
            Shape::Linear => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    prog.set_draw_delta(u64::from(h) / 50);
                    prog.set_prefix(&format!(
                        "Sorting {}:",
                        if self.vertical { "columns" } else { "rows" }
                    ));
                    prog.tick();
                }

                for (idx_y, row) in rgba
                    .clone()
                    .pixels()
                    .collect::<Vec<_>>()
                    .chunks_mut(w as usize)
                    .enumerate()
                {
                    self.do_sort(&mut row[..]);

                    for (idx_x, px) in row.iter().enumerate() {
                        rgba.put_pixel(idx_x as u32, idx_y as u32, **px);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    prog.inc(1);
                }
            }
            _ => unreachable!(),
        }

        #[cfg(not(target_arch = "wasm32"))]
        prog.finish_with_message("Done sorting!");

        let mut img_out = DynamicImage::ImageRgba8(rgba);

        if self.vertical {
            img_out = img_out.rotate270();
        }

        img_out
    }
}
