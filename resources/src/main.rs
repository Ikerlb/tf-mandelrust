mod render;
mod encode;

use num::complex::Complex;
use std::str::FromStr;

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
            let zoom_level = rq.zoom_level; 
            let w = rq.width as f64; 
            let h = rq.height as f64; 

            let total = 1280 * 720;
            let zoom = 1f64 / (zoom_level as f64);
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
            
            let center = Complex::new(-0.761574,-0.0847596);

            let c1 = Complex::new(center.re - xspan, center.im + yspan);
            let c2 = Complex::new(center.re + xspan, center.im - yspan);

            let bounds = (c1, c2);

            let mut pixels: Vec<u8> = vec![0; img_bounds.0 * img_bounds.1];

            render::render(&mut pixels, img_bounds, bounds); 
            let buf = encode::convert_to_png(&pixels, img_bounds).unwrap();
                 
            Ok(Response::builder()
               .status(200)
               .header("Access-Control-Allow-Headers", "Content-Type")
               .body(base64::encode(&buf))
               .unwrap())
        },
        _ => unimplemented!(),
    }
}
