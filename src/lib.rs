mod utils;

use wasm_bindgen::prelude::*;

use std::ops::{Add,Sub, Mul, Div};
use std::ops::Range;
use std::fmt;
use js_sys::Math;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
}

fn get_canvas_size(ctx : &CanvasRenderingContext2d) -> (u32, u32) {

    let canvas = ctx.canvas().unwrap();

    (canvas.width(), canvas.height())
}


#[derive(Debug, Copy, Clone, PartialEq)]
struct PixelF64 {
    r : f64,
    g : f64,
    b : f64,
    a : f64
}

impl PixelF64 {
    fn rgb(r: f64, g: f64, b: f64) -> Self {
        PixelF64 {r:r, g:g, b:b, a:255.0}
    }
}


impl From<Pixel> for PixelF64 {
    fn from(p: Pixel) -> Self {
        PixelF64{
            r: p.r as f64,
            g: p.g as f64,
            b: p.b as f64,
            a: p.a as f64,
        }
    }
}



impl Mul<f64> for PixelF64 {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        Self {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
            a: self.a * other
        }
    }
}


impl Add for PixelF64 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a
        }
    }
}




#[derive(Debug, Copy, Clone, PartialEq)]
struct Pixel {
    r : u8,
    g : u8,
    b : u8,
    a : u8
}

impl Pixel {
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        Pixel {r:r, g:g, b:b, a:255}
    }
    fn rgba(r: u8, g: u8, b: u8, a:u8) -> Self {
        Pixel {r:r, g:g, b:b, a:a}
    }
}


impl From<PixelF64> for Pixel {
    fn from(p: PixelF64) -> Self {
        Pixel {
            r: p.r as u8,
            g: p.g as u8,
            b: p.b as u8,
            a: p.a as u8,
        }
    }
}


impl Sub for Pixel {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
            a: self.a - other.a
        }
    }
}


impl Add for Pixel {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a
        }
    }
}


impl Mul<u8> for Pixel {
    type Output = Self;

    fn mul(self, other: u8) -> Self::Output {
        Self {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
            a: self.a * other
        }
    }
}



impl Div<u8> for Pixel {
    type Output = Self;

    fn div(self, other: u8) -> Self::Output {
        Self {
            r: self.r / other,
            g: self.g / other,
            b: self.b / other,
            a: self.a / other
        }
    }
}

enum EdgeHandling {
    Wrap,
    Clamp,
    None
}

struct Pixels {
    w : u32,
    h : u32,
    data : Vec<u8>,
    edges : EdgeHandling
}

//todo : implement Deref for data

impl Pixels {
    fn get_at_index(&self, i : usize) -> Option<Pixel>{
        Some(Pixel {
            r : *self.data.get(i*4)?,
            g : *self.data.get(i*4 + 1)?,
            b : *self.data.get(i*4 + 2)?,
            a : *self.data.get(i*4 + 3)?
        })
    }

    fn set_at_index(&mut self, i : usize, pix: Pixel){
        *self.data.get_mut(i*4).unwrap() = pix.r;
        *self.data.get_mut(i*4+1).unwrap() = pix.g;
        *self.data.get_mut(i*4+2).unwrap() = pix.b;
        *self.data.get_mut(i*4+3).unwrap() = pix.a;
    }

    fn coords_from_index(&self, i : u32) -> (u32, u32) {
        (i % self.w, i / self.w)
    }


    fn index_from_coords(&self, x : u32, y: u32) -> Option<usize> {
        match self.edges {
            EdgeHandling::Clamp => {
                let clamped_x = x.clamp(0,self.w -1);
                let clamped_y = y.clamp(0,self.h -1);

                Some(((clamped_y * self.w + clamped_x)) as usize)
            }
            EdgeHandling::None => {
                if x >= self.w {
                    None
                }
                else if y >= self.h {
                    None
                }
                else {
                    Some(((y * self.w + x)) as usize)
                }
            }
            EdgeHandling::Wrap => {
                let wrapped_x = x % self.w;
                let wrapped_y = y % self.h;

                Some(((wrapped_y * self.w + wrapped_x)) as usize)
            }
        }
        
    }

    fn get(&self, x : u32, y : u32) -> Option<Pixel> {
        let i = self.index_from_coords(x,y)?;
        self.get_at_index(i)
    }

    fn set(&mut self, x : u32, y : u32, pix : Pixel) -> Option<()> {
        let i = self.index_from_coords(x,y)?;
        self.set_at_index(i, pix);
        Some(())
    }

    fn index_range(& self) -> Range<usize> {
        0..((self.w*self.h) as usize)
    }

    fn create_from_ctx(ctx : &CanvasRenderingContext2d) -> Option<Self> {

        let canvas = ctx.canvas()?;

        let w = canvas.width();
        let h = canvas.height();

        Some(Pixels {
            w: w,
            h: h,
            data: ctx.get_image_data(0.0, 0.0, w as f64, h as f64).ok()?.data().to_vec(),
            edges: EdgeHandling::None
        })
    }

    fn to_image_data(&mut self) -> Result<ImageData, JsValue>{

        Ok(ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut self.data), self.w, self.h)?)
    }
 }

// #[wasm_bindgen]
// pub struct GradientState {
//     innerColour : Pixel,
//     outerColour : Pixel,
//     pos : (f64, f64),
//     size : f64,
//     velocity : (f64, f64)
// }

// pub fn CreateGradients(width: f64, height : f64) -> Vec<GradientState> {
//     vec![GradientState{
//         innerColour : Pixel::rgb(255, 0, 255),
//         outerColour: Pixel::rgba(255,255,255,0),
//         pos:(Math::random() * width, Math::random() * height),
//         size: 100.0,
//         velocity: (Math::random() * 10.0, Math::random() * 10.0),
//     },
//     GradientState{
//         innerColour : Pixel::rgb(0, 0, 255),
//         outerColour: Pixel::rgba(255,255,255,0),
//         pos:(Math::random() * width, Math::random() * height),
//         size: 100.0,
//         velocity: (Math::random() * 10.0, Math::random() * 10.0),
//     },
//     GradientState{
//         innerColour : Pixel::rgb(255, 0, 0),
//         outerColour: Pixel::rgba(255,255,255,0),
//         pos:(Math::random() * width, Math::random() * height),
//         size: 100.0,
//         velocity: (Math::random() * 10.0, Math::random() * 10.0),
//     }]
// }


#[wasm_bindgen]
pub fn draw(ctx : &CanvasRenderingContext2d) -> Result<(), JsValue> {
    ctx.set_image_smoothing_enabled(false);

    let mut pixels = Pixels::create_from_ctx(ctx).unwrap();




    dither(&mut pixels);



    let imdata = pixels.to_image_data()?;

    ctx.put_image_data(&imdata, 0.0, 0.0)
}


#[wasm_bindgen]
pub fn dither_one_pixel(ctx : &CanvasRenderingContext2d, x: f64, y:f64) -> Result<(), JsValue> {
    ctx.set_image_smoothing_enabled(false);

    let mut pixels = Pixels::create_from_ctx(ctx).unwrap();

    dither_pixel(&mut pixels, x as u32, y as u32);

    let imdata = pixels.to_image_data()?;

    ctx.put_image_data(&imdata, 0.0, 0.0)
}


fn mutate(pixels : &mut Pixels) {
    for i in pixels.index_range() {
        if Math::random() < 0.005 {
            pixels.set_at_index(i, Pixel::rgb(255,255,0));
        }
    }
}


fn dither(pixels : &mut Pixels) {
    for y in 0..pixels.h {
        for x in 0..pixels.w {
            dither_pixel(pixels, x, y);
        }
    }
}

fn dither_pixel(pixels : &mut Pixels, x:u32, y:u32) {
    let old = pixels.get(x, y).unwrap();
    let new = get_closest_palette_colour(old, Palette::Colour);
    let diff = PixelF64 {
        r: old.r as f64 - new.r as f64,
        g: old.g as f64 - new.g as f64,
        b: old.b as f64 - new.b as f64,
        a: old.a as f64 - new.a as f64,
    };

    pixels.set(x, y, new);
    pixels.edges = EdgeHandling::Clamp;
    
    pixels.set(x+1, y  , Pixel::from(PixelF64::from(pixels.get(x+1,y   ).unwrap()) + diff * (7.0/16.0)));
    pixels.set(x-1, y+1, Pixel::from(PixelF64::from(pixels.get(x-1, y+1).unwrap()) + diff * (3.0/16.0)));
    pixels.set(x  , y+1, Pixel::from(PixelF64::from(pixels.get(x  , y+1).unwrap()) + diff * (5.0/16.0)));
    pixels.set(x+1, y+1, Pixel::from(PixelF64::from(pixels.get(x+1, y+1).unwrap()) + diff * (1.0/16.0)));
    
}

enum Palette {
    Colour,
    Grey
}

fn get_closest_palette_colour(p: Pixel, palette: Palette) -> Pixel {
    let mut outp = p.clone();

    match palette {
        Palette::Colour => {
            if p.r > 127 {outp.r = 255;}
            else {outp.r = 0;}
            if p.g > 127 {outp.g = 255;}
            else {outp.g = 0;}
            if p.b > 127 {outp.b = 255;}
            else {outp.b = 0;}
        }
        Palette::Grey => {
            let mut grey = (0.2126 * (p.r as f64) + 0.7152 * (p.g as f64) + 0.0722 * (p.b as f64)) as u8;

            if grey > 127 {
                grey = 255;
            }
            else {
                grey = 0;
            }

            outp.r = grey;
            outp.g = grey;
            outp.b = grey;
        }
    }
    
    outp
}


fn gen(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    let mut i = (Math::random() * 255.0) as u8;
    let inc = (Math::random() * 10.0) as u8;

    for x in 0..width {
        for y in 0..height {
            i += inc;
            data.push(i % 255);
            data.push(i % 255);
            data.push(i % 255);
            data.push(255);
        }
    }

    data
}