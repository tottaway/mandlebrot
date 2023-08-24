#![feature(portable_simd)]
use std::ops::BitOr;
use std::simd::{f64x8, i64x8, mask64x8, SimdPartialOrd, StdFloat};
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

const TWOS: f64x8 = f64x8::from_array([2f64; 8]);
const FLOAT_ZEROS: f64x8 = f64x8::from_array([0f64; 8]);

impl Complexf64x8 {
  fn calculate_mandlebrot(c: Complexf64x8) -> i64x8 {
    let max = 256;
    let ones = i64x8::splat(1i64);
    let zeros = i64x8::splat(0i64);
    let float_ones = f64x8::splat(4f64);
    (0..max)
      .fold(
        (
          Complexf64x8 {
            real: f64x8::splat(0f64),
            imag: f64x8::splat(0f64),
          },
          i64x8::splat(0i64),
          mask64x8::splat(false),
        ),
        |acc, _| {
          if acc.2.all()
          {
            return acc
          }
          let (square, norm) = acc.0.square_and_norm();
          let mask = norm.simd_ge(float_ones).bitor(acc.2);
          (square.sum(c), acc.1 + mask.select(zeros, ones), mask)
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

  fn square_and_norm(self) -> (Complexf64x8, f64x8) {
    let real_sq = self.real * self.real;
    let imag_sq = self.imag * self.imag;
    return (
      Complexf64x8 {
        real: real_sq - imag_sq,
        imag: TWOS * self.real * self.imag,
      },
      real_sq + imag_sq,
    );
  }
}

#[macroquad::main("Mandlebrot")]
async fn main() {
  let x_res = 2560i64;
  let y_res = 1440i64;
  let mut model = Model {
    buf:      vec![255u8; (x_res * y_res * 4) as usize],
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
      model.scale *= 1.015;
    }
    if is_key_down(KeyCode::E) {
      println!("big");
      model.scale *= 0.985;
    }
    let scaler = f64x8::from_slice(&[0f64, 1f64, 2f64, 3f64, 4f64, 5f64, 6f64, 7f64])
      / f64x8::splat(model.scale)
      / f64x8::splat(x_res as f64);
    model
      .buf
      .par_chunks_mut(4 * 8)
      .enumerate()
      .for_each(|(n, val)| {
        let x = (((n * 8) as i64) % x_res - (x_res / 2)) as f64;
        let y = (((n * 8) as i64) / x_res - (y_res / 2)) as f64;
        let scale_x = x / x_res as f64 / model.scale + model.center_x;
        let scale_y = y / x_res as f64 / model.scale + model.center_y;
        let c = Complexf64x8 {
          real: f64x8::splat(scale_x) + scaler,
          imag: f64x8::splat(scale_y),
        };
        let mand = Complexf64x8::calculate_mandlebrot(c).to_array();
        for i in 0..8usize {
          val[0 + i * 4] = (mand[i] as u32 * 7727) as u8;
          val[1 + i * 4] = (mand[i] as u32 * 6151) as u8;
          val[2 + i * 4] = (mand[i] as u32 * 5107) as u8;
        }
      });
    let texture: Texture2D = Texture2D::from_rgba8(x_res as u16, y_res as u16, &model.buf);
    draw_texture(&texture, 0., 0., WHITE);
    next_frame().await;
    dbg!(Instant::now() - start);
  }
}
