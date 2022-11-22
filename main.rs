use image::io::Reader as ImageReader;
use image::Pixel;
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};
use std::env;

const HEAT_MAP: [char;16]= [' ','.','´',':','i','!','I','~','+','x','$','X','#', '▄','■','█'];

fn main() {
    let filename = env::args().nth(1).expect("Enter filename");
    let img = ImageReader::open(filename)
        .expect("filename invalid")
        .decode()
        .unwrap();

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
        for j in i*width..i*width+width {
            print!("{}", HEAT_MAP[avg_pixels[j as usize] as usize]);
        }
        println!("");
    }
}
