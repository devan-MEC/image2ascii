use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::Pixel;

/// Simple program generate ASCII art from an input image
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the image file to be asii art'd
    #[arg(short, long)]
    file_path: String,

    /// Colorize the ascii output
    #[arg(short, long, default_value_t = false)]
    colored: bool,

    /// Resize the image so that it fits the current terminal's dimensions, preserving aspect ratio.
    #[arg(short, long, default_value_t = false)]
    resize: bool,
}

const HEAT_MAP: [char; 16] = [
    ' ', '.', '´', ':', 'i', '!', 'I', '~', '+', 'x', '$', 'X', '#', '▄', '■', '█',
];

fn main() -> Result<()> {
    let args = Args::parse();
    let img = ImageReader::open(args.file_path)
        .context("file_path should be a valid path to a file")?
        .decode()
        .context("file_path should point to an image file with the correct extension")?;
    let img = if args.resize {
        img.resize(100, 100, FilterType::Nearest)
    } else {
        img
    };
    let (width, height) = img.dimensions();
    let pixels_with_value: Vec<(u8, u8, u8, u8)> = img
        .pixels()
        .map(|p| {
            let p = p.2.channels();
            (
                p[0],
                p[1],
                p[2],
                ((p[0] as u32 + p[1] as u32 + p[2] as u32) / 3) as u8,
            )
        })
        .map(|(r, g, b, p)| (r, g, b, p / 16))
        .collect();
    for i in 0..height {
        for j in i * width..i * width + width {
            print!(
                "{}",
                if args.colored {
                    let p = pixels_with_value[j as usize];
                    HEAT_MAP[p.3 as usize]
                        .to_string()
                        .truecolor(p.0, p.1, p.2)
                        .to_string()
                } else {
                    HEAT_MAP[pixels_with_value[j as usize].3 as usize].to_string()
                }
            );
        }
        println!("");
    }
    Ok(())
}
