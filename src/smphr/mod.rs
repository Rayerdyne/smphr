mod stickman;

extern crate clap;
use clap::{Arg, App};
use image::{ImageError, Rgb, RgbImage};
use stickman::*;

const DEF_HEIGHT : u32 = 400;
const DEF_WIDTH : u32 = 600;

pub enum SmphrError {
    NoData,
    InvalidData,
    ShouldNotAppear,
    NoPath,
    CouldNotWrite(ImageError)
}

impl std::fmt::Display for SmphrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoData => write!(f, "No input provided"),
            Self::InvalidData => write!(f, "No valid character in input"),
            Self::NoPath => write!(f, "No path providen for output file"),
            Self::CouldNotWrite(e) => write!(f, "Could not write output file:\
{}", e),
            Self::ShouldNotAppear => write!(f, "SHOULD NOT BE PRINTED"),
        }
    }
}

impl std::convert::From<StickmanError> for SmphrError {
    fn from(_: StickmanError) -> SmphrError {
        SmphrError::ShouldNotAppear
    }
}

pub struct SmphrParams {
    height: u32,
    width: u32,
    data: String,
    path: String,
}

impl SmphrParams {
    pub fn from_args() -> Result<SmphrParams, SmphrError> {
        let matches = app_args();

        let mut height = DEF_HEIGHT;
        let mut width = DEF_WIDTH;

        if let Some(s) = matches.value_of("height") {
            height = match s.parse::<u32>() {
                Ok(v) => v,
                Err(_) => DEF_HEIGHT,
            }
        }

        if let Some(s) = matches.value_of("width") {
            width = match s.parse::<u32>() {
                Ok(v) => v,
                Err(_) => DEF_WIDTH,
            }
        }

        let data = match matches.value_of("data") {
            Some(s) => s.to_string(),
            None => return Err(SmphrError::NoData),
        };

        let path = match matches.value_of("path") {
            Some(s) => s.to_string(),
            None => return Err(SmphrError::NoPath),
        };

        Ok(SmphrParams::from_values(height, width, data, path))
    }

    fn from_values(height: u32, width: u32, data: String, path: String)
        -> SmphrParams {
        SmphrParams {
            height, 
            width,
            data,
            path,
        }
    }

    #[allow(dead_code)]
    fn default() -> SmphrParams {
        SmphrParams {
            height: DEF_HEIGHT,
            width: DEF_WIDTH,
            data: String::new(),
            path: String::new(),
        }
    }
}

fn app_args() -> clap::ArgMatches<'static> {
    App::new("smphr")
        .version("0.1.0")
        .author("François Straet")
        .about("Generate semaphore images from text")
        .arg(Arg::with_name("path")
            .help("Path of output image to be written")
            .required(true)
            .index(1))
        .arg(Arg::with_name("data")
            .help("The text to be translated in smphr")
            .required(true)
            .index(2))
        .arg(Arg::with_name("height")
            .help("Set the output image's height. When incorrect, use default 400.")
            .short("h")
            .long("height")
            .takes_value(true))
        .arg(Arg::with_name("width")
            .help("Set the output image's width. When incorrect, use default 600.")
            .short("w")
            .long("width")
            .takes_value(true))
        .get_matches()
}

pub fn exec(params: SmphrParams) -> Result<(), SmphrError> {

    // first detect invalid inputs:
    let mut valid = true;
    for c in params.data.chars() {
        if c.is_alphanumeric() {
            valid = true;
            break;
        }
    }
    if !valid { return Err(SmphrError::InvalidData); }

    let (tabw, tabh) = (params.width as usize, params.height as usize);
    let mut vec_tab = vec![0 as u8; tabw * tabh];
    let tab = vec_tab.as_mut_slice();

    let mut it = params.data.chars();
    let mut prev = Stickman::first_from_letter('z')?;

    loop {
        let c = it.next().unwrap();
        let k = Stickman::first_from_letter(c);
        if let Ok(prev) = k {
            prev.draw(tab, &params);
            break;
        }
    }

    while let Some(c) = it.next() {
        match Stickman::from_letter_with_prev(c, prev.get_pos(), &params) {
            Ok(s) => {
                s.draw(tab, &params);
                prev = s;
            },
            Err(e) => {
                match e {
                    StickmanError::VerticalOverflow => {
                        println!("Vertical overflow, exiting loop.\
\nThis will cut the input text.");
                        break;
                    },
                    _ => {
                        println!("Error creating stickman, {}.", e);
                    }
                }
            }
        }
    }

    let white = Rgb([255, 255, 255]);
    let black = Rgb([0, 0, 0]);
    let red =   Rgb([255, 0, 0]);
    let imgbuf = RgbImage::from_fn(params.width, params.height, |x, y| {
        match tab[y as usize * tabw + x as usize] {
            1 => black,
            2 => red,
            _ => white,
        }
    });

    match imgbuf.save(params.path) {
        Ok(_) => {},
        Err(e) => { return Err(SmphrError::CouldNotWrite(e)); }
    }

    Ok(())
}
