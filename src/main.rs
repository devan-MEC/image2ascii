use anyhow::{Context, Result};
use clap::Parser;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::Pixel;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the image file to be asii art'd
    #[arg(short, long)]
    file_path: String,
}

const HEAT_MAP: [char; 16] = [
    ' ', '.', '´', ':', 'i', '!', 'I', '~', '+', 'x', '$', 'X', '#', '▄', '■', '█',
];

fn main() -> Result<()> {
    let args = Args::parse();
    let img = ImageReader::open(args.file_path)
        .context("file_path should be a valid path to a file!")?
        .decode()
        .context("file_path should point to an image file!")?;

    let (width, height) = img.dimensions();
    let avg_pixels: Vec<u8> = img
        .pixels()
        .map(|p| {
            let p = p.2.channels();
            ((p[0] as u32 + p[1] as u32 + p[2] as u32) / 3) as u8
        })
        .map(|p| p / 16)
        .collect();
    for i in 0..height {
        for j in i * width..i * width + width {
            print!("{}", HEAT_MAP[avg_pixels[j as usize] as usize]);
        }
        println!("");
    }
    Ok(())
}
