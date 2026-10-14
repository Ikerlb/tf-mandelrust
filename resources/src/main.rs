mod render;
mod encode;
mod targets;

use num::complex::Complex;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json;

use lambda_runtime::{self, Context, Error};
use log::LevelFilter;
use simple_logger::SimpleLogger;

use lambda_http::handler;
use lambda_http::Body;
use lambda_http::{Response, IntoResponse, Request};

use base64::{encode, decode};

/*#[derive(Deserialize, Debug)]
struct MyRequest {
    lower_right: String,
    upper_left: String,
}*/
#[derive(Deserialize, Debug)]
struct MyRequest {
    zoom_level: usize,
    width: usize,
    height: usize,
    /// Index into targets::TARGETS; a random target when absent. The response's
    /// X-Target header says which one was used, so later frames can ask for it again.
    target: Option<usize>,
}

/// Varies between requests, which is all target picking needs; avoids an RNG crate.
fn random_target() -> usize {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
    nanos as usize % targets::TARGETS.len()
}



fn from_pair<T: FromStr>(s: &str) -> Option<(T, T)>{
    let i = s.find(",")?;
    match (T::from_str(&s[..i]), T::from_str(&s[i + 1..])) {
        (Ok(fst), Ok(snd)) => Some((fst, snd)),
        (_, _) => None
    }
}

fn parse_complex<T: FromStr>(s: &str) -> Option<Complex<T>>{
    match from_pair(s) {
        Some((fst, snd)) => Some(Complex::new(fst, snd)),
        None => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_utc_timestamps()
        .init()
        .unwrap();

    let func = handler(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn my_handler(mut req: Request, _c: Context) -> Result<Response<String>, Error> {
    let payload = req.body_mut();
    log::warn!("{:?}", payload);   
    match payload {
        Body::Text(txt) => {
            let rq: MyRequest = serde_json::from_str(txt)
                .unwrap();
            let index = match rq.target {
                None => random_target(),
                Some(i) if i < targets::TARGETS.len() => i,
                Some(_) => {
                    return Ok(Response::builder()
                        .status(400)
                        .header("Access-Control-Allow-Headers", "Content-Type")
                        .body(format!("target must be below {}", targets::TARGETS.len()))
                        .unwrap())
                }
            };
            let target = &targets::TARGETS[index];
            // Past max_zoom the minibrot fills the frame (or f64 runs out), so hold there.
            let zoom_level = (rq.zoom_level.max(1) as f64).min(target.max_zoom);
            let w = rq.width as f64; 
            let h = rq.height as f64; 

            let total = 1280 * 720;
            let zoom = 1f64 / zoom_level;
            let mut xscale = 0f64;
            let mut yscale = 0f64;
            let mut nw = ((total as f64) / (h / w)).sqrt();  
            let mut nh = (total as f64) / nw;
            if w <= h {
                xscale = w / h;
                yscale = 1f64;
            } else {
                yscale = h / w;
                xscale = 1f64;
            }

            let img_bounds = (nw as usize, nh as usize);

            let xspan = 1.3 * xscale * zoom;
            let yspan = 1.3 * yscale * zoom;
            
            let center = Complex::new(target.re, target.im);

            let c1 = Complex::new(center.re - xspan, center.im + yspan);
            let c2 = Complex::new(center.re + xspan, center.im - yspan);

            let bounds = (c1, c2);

            let mut pixels: Vec<u8> = vec![0; img_bounds.0 * img_bounds.1];

            render::render(&mut pixels, img_bounds, bounds, render::iteration_limit(zoom_level)); 
            let buf = encode::convert_to_png(&pixels, img_bounds).unwrap();
                 
            Ok(Response::builder()
               .status(200)
               .header("Access-Control-Allow-Headers", "Content-Type")
               .header("Access-Control-Expose-Headers", "X-Target, X-Max-Zoom")
               .header("X-Target", index.to_string())
               .header("X-Max-Zoom", (target.max_zoom as u64).to_string())
               .body(base64::encode(&buf))
               .unwrap())
        },
        _ => unimplemented!(),
    }
}
