use num::complex::Complex;

fn escape(c: Complex<f64>, limit: usize) -> Option<usize>{
    let mut x = Complex::new(0f64, 0f64);        
    for i in 0..limit {
        if x.norm_sqr() > 4.0 { 
            return Some(i);
        }
        x = x * x + c;
    }
    None
}

fn pixel_to_point(x: usize, y: usize, dx: f64, dy: f64, xs: f64, ys: f64) -> Complex<f64> {
    let nx = xs + (x as f64 * dx);                    
    let ny = ys + (y as f64 * dy);                    
    Complex::new(nx, ny)
}

pub fn render(pixels: &mut [u8], img_bounds: (usize, usize), bounds: (Complex<f64>, Complex<f64>)) {
    let xs = bounds.0.re;
    let xe = bounds.1.re;
    let ys = bounds.0.im;
    let ye = bounds.1.im;

    let dx = (xe - xs) / (img_bounds.0 as f64); 
    let dy = (ye - ys) / (img_bounds.1 as f64);
    for x in 0..img_bounds.0 {
        for y in 0..img_bounds.1 {
            let c = pixel_to_point(x, y, dx, dy, xs, ys);
            pixels[x + y * img_bounds.0] = match escape(c, 255) {
                None => 0,
                Some(i) => 255 - (i as u8), 
            };
        }
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
