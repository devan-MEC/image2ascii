use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    cursor, execute, queue,
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

struct EventManager<'a> {
    events: Vec<(Box<dyn FnMut() -> bool + 'a>, Box<dyn Fn()>)>,
}

impl<'a> EventManager<'a> {
    pub fn append<F1, F2>(mut self, test: F1, callback: F2) -> Self
    where
        F1: FnMut() -> bool + 'a,
        F2: Fn() + 'static,
    {
        self.events.push((Box::new(test), Box::new(callback)));
        self
    }

    pub fn run(&mut self) {
        for e in &mut self.events {
            if e.0() {
                e.1();
            }
        }
    }
}

impl<'a> Default for EventManager<'a> {
    fn default() -> Self {
        EventManager { events: Vec::new() }
    }
}

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

    /// Time to wait between frames (GIF mode)
    #[arg(short, long, default_value_t = 200)]
    animation_delay: u64,

    /// Loop animation (stream mode). Will enable video for the webcam_feed option.
    #[arg(short, long, default_value_t = false)]
    loop_animation: bool,

    /// Block character (display the actual color of pixels using only ASCII's block character)
    #[arg(short, long, default_value_t = false)]
    block_character: bool,

    /// Display ASCII'd frame from your webcam feed
    #[arg(short, long, default_value_t = false)]
    webcam_feed: bool,
}

const HEAT_MAP_LENGTH: usize = 16;
const HEAT_MAP: [&str; HEAT_MAP_LENGTH] = [
    "   ",
    "...",
    "´´´",
    ":::",
    "~~~",
    "+++",
    "iii",
    "xxx",
    "!!!",
    "III",
    "###",
    "$$$",
    "XXX",
    "▄▄▄",
    "■■■",
    "███",
];

fn resize_img(img: DynamicImage) -> Result<DynamicImage> {
    let canvas_dimensions = terminal::size()?;
    let canvas_dimensions = (
        canvas_dimensions.0 as u32 / 3,
        canvas_dimensions.1 as u32 - 3,
    );
    let img_dimensions = img.dimensions();
    let wr = img_dimensions
        .1
        .checked_div(canvas_dimensions.0)
        .context("I don't like zero sized terminals")?;
    let hr = img_dimensions
        .1
        .checked_div(canvas_dimensions.1)
        .context("I don't like zero sized terminals")?;
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

fn print_img(img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, args: &Args) -> Result<()> {
    let mut stdout = stdout();
    let img = DynamicImage::ImageRgba8(img);
    let img = if args.resize { resize_img(img)? } else { img };
    let (width, height) = img.dimensions();
    stdout.execute(cursor::MoveTo(0, 0)).unwrap();
    let pixels_with_value: Vec<(u8, u8, u8, u8)> = img
        .pixels()
        .map(|p| {
            let p = p.2.channels();
            (
                p[0],
                p[1],
                p[2],
                (((p[0] as u32 + p[1] as u32 + p[2] as u32) / 3) / (256 / HEAT_MAP_LENGTH) as u32)
                    as u8,
            )
        })
        .map(|(r, g, b, p)| (r, g, b, p))
        .collect();
    for i in 0..height {
        for j in i * width..i * width + width {
            let p = pixels_with_value[j as usize];
            let text = if args.true_color {
                "███"
            } else {
                HEAT_MAP[p.3 as usize]
            };
            if args.colored {
                queue!(
                    stdout,
                    PrintStyledContent(text.with(Color::Rgb {
                        r: p.0,
                        g: p.1,
                        b: p.2
                    }))
                )?
            } else {
                queue!(stdout, Print(text))?
            }
        }
        queue!(stdout, Print("\n"))?;
    }
    stdout.flush()?;
    Ok(())
}

fn print_stream<I>(stream: I, args: &Args) -> Result<()>
where
    I: IntoIterator<Item = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
{
    let mut canvas_size = terminal::size()?;
    let mut events = EventManager::default().append(
        || {
            let size = terminal::size().unwrap();
            if canvas_size != size {
                canvas_size = size;
                true
            } else {
                false
            }
        },
        || {
            execute!(stdout(), Clear(ClearType::All)).unwrap();
        },
    );

    stream.into_iter().for_each(|frame| {
        print_img(frame, &args).unwrap();
        events.run();
        sleep(Duration::from_millis(args.animation_delay));
    });
    Ok(())
}

fn print_gif(path: &str, args: &Args) -> Result<()> {
    let file = std::fs::File::open(path)?;
    let frames = GifDecoder::new(file)?
        .into_frames()
        .collect_frames()?
        .into_iter()
        .map(Frame::into_buffer);
    if args.loop_animation {
        print_stream(frames.cycle(), args)
    } else {
        print_stream(frames, args)
    }
}

fn print_camera(args: &Args) -> Result<()> {
    let mut camera = CameraIter::default();
    if args.loop_animation {
        print_stream(camera, args)
    } else {
        print_img(
            camera.next().expect("Failed to get frame from camera"),
            args,
        )
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.file_path.clone();
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    if args.webcam_feed {
        print_camera(&args)?;
    } else if image::ImageFormat::from_path(&path)
        .context("file_path should be a valid path to a file")?
        == ImageFormat::Gif
    {
        print_gif(&path, &args)?;
    } else {
        let img = ImageReader::open(path)?
            .decode()
            .context("file_path should point to an image file with the correct extension")?;
        print_img(img.to_rgba8(), &args)?;
    }
    Ok(())
}

struct CameraIter {
    camera: nokhwa::Camera,
}

impl Default for CameraIter {
    fn default() -> Self {
        let index = nokhwa::utils::CameraIndex::Index(0);
        let requested = nokhwa::utils::RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
            nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate,
        );
        let mut camera = nokhwa::Camera::new(index, requested).unwrap();
        camera
            .open_stream()
            .context("Couldn't start camera stream")
            .unwrap();
        Self { camera }
    }
}

impl Iterator for CameraIter {
    type Item = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.camera
            .frame()
            .expect("Failed to get next frame from the camera")
            .decode_image::<nokhwa::pixel_format::RgbAFormat>()
            .ok()
    }
}
