use std::time::{Duration, Instant};

use macroquad::prelude::*;
use num::complex::Complex;
use rayon::prelude::*;

struct Model {
  buf: Vec<u8>,
  center_x: f64,
  center_y: f64,
  scale: f64,
}

fn calculate_mandlebrot(c: Complex<f64>) -> u8 {
  let max = 255;
  (0..max)
    .fold((Complex::new(0f64, 0f64), 0), |acc, _| {
      if acc.0.norm_sqr() > 1f64 {
        acc
      } else {
        (acc.0.powu(2) + c, acc.1 + 1)
      }
    })
    .1
}

#[macroquad::main("Mandlebrot")]
async fn main() {
  let mut model = Model {
    buf: vec![0u8; 1024 * 1024 * 4],
    center_x: 0f64,
    center_y: 0f64,
    scale: 0.25f64,
  };
  loop {
    if is_key_down(KeyCode::W) {
      println!("up");
      model.center_y -= 0.02 / model.scale;
    }
    if is_key_down(KeyCode::S) {
      println!("down");
      model.center_y += 0.02 / model.scale;
    }
    if is_key_down(KeyCode::D) {
      println!("right");
      model.center_x += 0.02 / model.scale;
    }
    if is_key_down(KeyCode::A) {
      println!("left");
      model.center_x -= 0.02 / model.scale;
    }
    if is_key_down(KeyCode::Q) {
      println!("small");
      model.scale *= 1.05;
    }
    if is_key_down(KeyCode::E) {
      println!("big");
      model.scale *= 0.95;
    }
    model
      .buf
      .par_chunks_mut(4)
      .enumerate()
      .for_each(|(n, val)| {
        let x = (n as i32) % 1024 - 512;
        let y = (n as i32) / 1024 - 512;
        let c = 
          Complex::new((x as f64) / 1024f64 / model.scale, (y as f64) / 1024f64 / model.scale)
          + Complex::new(model.center_x, model.center_y);
        let mand = calculate_mandlebrot(c);
        val[0] = (mand as u32 * 7727) as u8;
        val[1] = (mand as u32 * 6151) as u8;
        val[2] = (mand as u32 * 5107) as u8;
        val[3] = 255;
      });
    let texture: Texture2D = Texture2D::from_rgba8(1024, 1024, &model.buf);
    draw_texture(&texture, 0., 0., WHITE);
    next_frame().await
  }
}
