use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    cursor, queue,
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use image::{
    codecs::gif::GifDecoder, imageops::FilterType, io::Reader as ImageReader, AnimationDecoder,
    DynamicImage, Frame, GenericImageView, ImageFormat, Pixel,
};
use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

/// Simple program that generates ASCII art from an input image
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

    /// Time to wait between frames when rendering a GIF
    #[arg(short, long, default_value_t = 200)]
    animation_delay: u64,
}

const HEAT_MAP: [char; 16] = [
    ' ', '.', '´', ':', 'i', '!', 'I', '~', '+', 'x', '$', 'X', '#', '▄', '■', '█',
];

fn resize_img(img: DynamicImage) -> Result<DynamicImage> {
    //TODO correct image resizing algortihm
    let canvas_dimensions = terminal::size()?;
    let canvas_dimensions = (canvas_dimensions.0 as u32, canvas_dimensions.1 as u32);
    let img_dimensions = img.dimensions();
    let wr = img_dimensions.0 / canvas_dimensions.0;
    let hr = img_dimensions.1 / canvas_dimensions.1;
    Ok(if hr > wr {
        img.resize(
            canvas_dimensions.1 * img_dimensions.0 / img_dimensions.1,
            canvas_dimensions.1,
            FilterType::Nearest,
        )
    } else {
        img.resize(
            canvas_dimensions.0,
            canvas_dimensions.0 * img_dimensions.1 / img_dimensions.0,
            FilterType::Nearest,
        )
    })
}

fn print_img(img: DynamicImage, args: &Args) -> Result<()> {
    let mut stdout = stdout();
    let img = if args.resize { resize_img(img)? } else { img };
    let (width, height) = img.dimensions();
    queue!(stdout, Clear(ClearType::FromCursorUp))?;
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
            if args.colored {
                let p = pixels_with_value[j as usize];
                queue!(
                    stdout,
                    PrintStyledContent(HEAT_MAP[p.3 as usize].to_string().with(Color::Rgb {
                        r: p.0,
                        g: p.1,
                        b: p.2
                    }))
                )?
            } else {
                queue!(
                    stdout,
                    Print(HEAT_MAP[pixels_with_value[j as usize].3 as usize].to_string())
                )?
            }
        }
        queue!(stdout, Print("\n"))?;
    }
    stdout.flush()?;
    Ok(())
}

fn print_gif(path: &str, args: &Args) -> Result<()> {
    //TODO fix scrolling issue when animating a GIF
    let file = std::fs::File::open(path)?;
    GifDecoder::new(file)?
        .into_frames()
        .collect_frames()?
        .into_iter()
        .map(Frame::into_buffer)
        .map(|frame| {
            let mut stdout = stdout();
            stdout.execute(cursor::MoveTo(0, 0))?;
            drop(stdout);
            let img = DynamicImage::ImageRgba8(frame);
            print_img(img, &args)?;
            sleep(Duration::from_millis(args.animation_delay));
            Ok::<_, anyhow::Error>(())
        })
        .collect()
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.file_path.clone();
    if image::ImageFormat::from_path(&path).context("file_path should be a valid path to a file")?
        == ImageFormat::Gif
    {
        print_gif(&path, &args)?;
    } else {
        let img = ImageReader::open(path)?
            .decode()
            .context("file_path should point to an image file with the correct extension")?;
        print_img(img, &args)?;
    }
    Ok(())
}
