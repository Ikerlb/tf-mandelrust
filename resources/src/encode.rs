use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

pub fn convert_to_png(pixels: &[u8], bounds: (usize, usize)) -> Result<Vec<u8>, std::io::Error>{
    let mut buf: Vec<u8> = vec!(); 
    let e = PNGEncoder::new(&mut buf);

    e.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(buf)
}
