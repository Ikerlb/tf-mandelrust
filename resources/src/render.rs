use num::complex::Complex;

/// Squared escape radius; large so the fractional part of the escape count is accurate.
const BAILOUT: f64 = 256.0 * 256.0;

/// Smooth (fractional) number of iterations before `c` escapes, or None if it's still
/// bounded after `limit` iterations.
fn escape(c: Complex<f64>, limit: usize) -> Option<f64>{
    let mut x = Complex::new(0f64, 0f64);        
    for i in 0..limit {
        let r2 = x.norm_sqr();
        if r2 > BAILOUT { 
            return Some(i as f64 + 1.0 - (r2.ln() / 2.0).log2());
        }
        x = x * x + c;
    }
    None
}

/// Iterations needed to show the detail near the boundary `zoom` times closer than the
/// full view; 255 at zoom 1.
pub fn iteration_limit(zoom: f64) -> usize {
    (255.0 + 250.0 * zoom.max(1.0).log10()) as usize
}

fn pixel_to_point(x: usize, y: usize, dx: f64, dy: f64, xs: f64, ys: f64) -> Complex<f64> {
    let nx = xs + (x as f64 * dx);                    
    let ny = ys + (y as f64 * dy);                    
    Complex::new(nx, ny)
}

pub fn render(pixels: &mut [u8], img_bounds: (usize, usize), bounds: (Complex<f64>, Complex<f64>), limit: usize) {
    let xs = bounds.0.re;
    let xe = bounds.1.re;
    let ys = bounds.0.im;
    let ye = bounds.1.im;

    let dx = (xe - xs) / (img_bounds.0 as f64); 
    let dy = (ye - ys) / (img_bounds.1 as f64);
    let mut counts = vec![None; pixels.len()];
    for y in 0..img_bounds.1 {
        for x in 0..img_bounds.0 {
            let c = pixel_to_point(x, y, dx, dy, xs, ys);
            counts[x + y * img_bounds.0] = escape(c, limit);
        }
    }
    shade(pixels, &counts);
}

/// Shades each pixel by its escape count's rank within this frame (histogram
/// equalization), so every frame spans white (escapes first) to dark gray (escapes last)
/// whatever its iteration range. Points in the set are black.
fn shade(pixels: &mut [u8], counts: &[Option<f64>]) {
    let mut sorted: Vec<f64> = counts.iter().flatten().copied().collect();
    sorted.sort_unstable_by(f64::total_cmp);
    let escaped = sorted.len() as f64;
    for (pixel, count) in pixels.iter_mut().zip(counts) {
        *pixel = match count {
            None => 0,
            Some(n) => {
                let rank = sorted.partition_point(|s| s <= n) as f64 / escaped;
                (255.0 - 215.0 * rank) as u8
            }
        };
    }
}

/* pub fn invert(pixels: &mut [u8], img_bounds: (usize, usize)) {
    for x in 0..img_bounds.0 {
        for y in 0..img_bounds.1 {
            pixels[x + y * img_bounds.0] = 255 - pixels[x + y * img_bounds.0];
        }
    }
}*/

// pub fn concurrent_render(pixels: &mut [u8], img_bounds: (usize, usize), bounds: (Complex<f64>, Complex<f64>, usize: threads)){
// }

// #[test]
// fn test_quadratic_map() {
//    assert_eq!(escape());                
//}
