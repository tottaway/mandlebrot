#![feature(portable_simd)]
use std::simd::{f64x8, u64x8, SimdPartialOrd};
use std::time::{Duration, Instant};

use macroquad::prelude::*;
use num::complex::Complex;
use rayon::prelude::*;

struct Model {
  buf:      Vec<u8>,
  center_x: f64,
  center_y: f64,
  scale:    f64,
}

#[derive(Clone, Copy)]
struct Complexf64x8 {
  real: f64x8,
  imag: f64x8,
}

impl Complexf64x8 {
  fn calculate_mandlebrot(c: Complexf64x8) -> u64x8 {
    let max = 255;
    (0..max)
      .fold(
        (
          Complexf64x8 {
            real: f64x8::splat(0f64),
            imag: f64x8::splat(0f64),
          },
          u64x8::splat(0u64),
        ),
        |acc, _| {
          let norm_sqr = acc.0.norm_sqr() * f64x8::splat(2f64);
          let mask = norm_sqr.simd_le(f64x8::splat(1f64));
          let ones = u64x8::splat(1u64);
          let zeros = u64x8::splat(0u64);
          (acc.0.simd_square().sum(c), acc.1 + mask.select(ones, zeros))
        },
      )
      .1
  }

  fn sum(&self, other: Complexf64x8) -> Complexf64x8 {
    return Complexf64x8 {
      real: self.real + other.real,
      imag: self.imag + other.imag,
    };
  }

  fn simd_square(self) -> Complexf64x8 {
    Complexf64x8 {
      real: self.real * self.real - (self.imag * self.imag),
      imag: f64x8::splat(2f64) * self.real * self.imag,
    }
  }

  fn norm_sqr(&self) -> f64x8 {
    self.imag * self.imag + self.real * self.real
  }
}

#[macroquad::main("Mandlebrot")]
async fn main() {
  let mut model = Model {
    buf:      vec![0u8; 1024 * 1024 * 4],
    center_x: 0f64,
    center_y: 0f64,
    scale:    0.25f64,
  };
  loop {
    let start = Instant::now();
    if is_key_down(KeyCode::W) {
      println!("up");
      model.center_y -= 0.005 / model.scale;
    }
    if is_key_down(KeyCode::S) {
      println!("down");
      model.center_y += 0.005 / model.scale;
    }
    if is_key_down(KeyCode::D) {
      println!("right");
      model.center_x += 0.005 / model.scale;
    }
    if is_key_down(KeyCode::A) {
      println!("left");
      model.center_x -= 0.005 / model.scale;
    }
    if is_key_down(KeyCode::Q) {
      println!("small");
      model.scale *= 1.01;
    }
    if is_key_down(KeyCode::E) {
      println!("big");
      model.scale *= 0.99;
    }
    let scaler = f64x8::from_slice(&[0f64, 1f64, 2f64, 3f64, 4f64, 5f64, 6f64, 7f64])
      / f64x8::splat(model.scale)
      / f64x8::splat(1024f64);
    model
      .buf
      .par_chunks_mut(4 * 8)
      .enumerate()
      .for_each(|(n, val)| {
        let x = (((n * 8) as i32) % 1024 - 512) as f64;
        let y = (((n * 8) as i32) / 1024 - 512) as f64;
        let scale_x = x / 1024f64 / model.scale + model.center_x;
        let scale_y = y / 1024f64 / model.scale + model.center_y;
        let c = Complexf64x8 {
          real: f64x8::splat(scale_x) + scaler,
          imag: f64x8::splat(scale_y),
        };
        let mand = Complexf64x8::calculate_mandlebrot(c).to_array();
        for i in 0..8usize {
          val[0 + i * 4] = (mand[i] as u32 * 7727) as u8;
          val[1 + i * 4] = (mand[i] as u32 * 6151) as u8;
          val[2 + i * 4] = (mand[i] as u32 * 5107) as u8;
          val[3 + i * 4] = 255;
        }
      });
    let texture: Texture2D = Texture2D::from_rgba8(1024, 1024, &model.buf);
    draw_texture(&texture, 0., 0., WHITE);
    next_frame().await;
    dbg!(Instant::now() - start);
  }
}
