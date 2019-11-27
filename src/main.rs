extern crate rand;
extern crate rand_chacha;
extern crate image;
extern crate imageproc;

use rand::prelude::*;
use std::time::Instant;
use rusttype::{FontCollection, Scale};
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_antialiased_line_segment_mut, draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use imageproc::pixelops::interpolate;
use priority_queue::PriorityQueue;
use crate::Alg::{Rev, RotL, RotR};
use std::path::Iter;
use std::collections::HashSet;

type Coord = (i32, i32);
type P = u16;
type PP = (u16, u16);

fn ord<T: Ord>((a, b): (T, T)) -> (T, T) {
    if a <= b { (a, b) } else { (b, a) }
}

#[derive(Debug)]
enum Alg {
    Rev,
    RotL,
    RotR,
}

struct Or<A, B> {
    a: A,
    b: B,
    proc_a: Option<bool>,
}

impl<A, B> Or<A, B> {
    fn new(a: A, b: B) -> Or<A, B> where A: Iterator, B: Iterator {
        Or {
            a,
            b,
            proc_a: None,
        }
    }
}

impl<A, B, I> Iterator for Or<A, B> where A: Iterator<Item=I>, B: Iterator<Item=I> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self.proc_a {
            Some(true) => self.a.next(),
            Some(false) => self.b.next(),
            None => match self.a.next() {
                None => {
                    self.proc_a = Some(false);
                    self.b.next()
                }
                Some(x) => {
                    self.proc_a = Some(true);
                    Some(x)
                }
            },
        }
    }
}

fn main() {
    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font)
        .unwrap()
        .into_font()
        .unwrap();

    let start = Instant::now();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);

    let size: P = 100;
    let index = |a: P, b: P| a as usize * size as usize + b as usize;
    let width: u32 = 256;
    let points: Vec<Coord> = (0..size).map(|_| (rng.gen_range(0, width as i32), rng.gen_range(0, width as i32))).collect();

    let draw = |gofn: &Vec<P>, i: u32| {
        let mut image: RgbImage = RgbImage::new(width, width);
        let black = Rgb([0u8, 0u8, 0u8]);
        let gray = Rgb([200u8, 200u8, 200u8]);
        let red = Rgb([100u8, 0u8, 0u8]);
        let yellow = Rgb([200u8, 200u8, 0u8]);
        let white = Rgb([255u8, 255u8, 255u8]);
        let height = 12.4;
        let scale = Scale {
            x: height,
            y: height,
        };
        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(width, width), white);
        for i in 1..gofn.len() {
            draw_antialiased_line_segment_mut(&mut image, points[gofn[i - 1] as usize], points[gofn[i] as usize], yellow, interpolate);
        }
        for (i, p) in points.iter().enumerate() {
            draw_filled_rect_mut(&mut image, Rect::at(p.0, p.1).of_size(1, 1), red);
            //draw_text_mut(&mut image, gray, p.0 as u32 + 2, p.1 as u32, scale, &font, &i.to_string());
        }
        image.save(format!("plot/{}.png", i)).unwrap();
    };

    let cost_matrix: Vec<u32> = {
        let mut r = Vec::with_capacity(size as usize * size as usize);
        for a in 0..size {
            for b in 0..size {
                r.push((((points[a as usize].0 - points[b as usize].0) as f64)
                    .hypot((points[a as usize].1 - points[b as usize].1) as f64) * 100f64)
                    .ceil() as u32)
            }
        }
        r
    };

    let cost = |a: P, b: P| cost_matrix[index(a, b)];

    let mut gofn: Vec<P> = (0..size).collect();

    let rev = move |gofn: &Vec<P>, n1: P, n3: P| {
        let g0 = gofn[n1 as usize - 1];
        let g1 = gofn[n1 as usize];
        let g2 = gofn[n3 as usize - 1];
        let g3 = gofn[n3 as usize];
        let old = cost(g0, g1) + cost(g2, g3);
        let new = cost(g0, g2) + cost(g1, g3);
        if new < old {
            Some((Rev, n1, n3, old - new))
        } else {
            None
        }
    };

    let rotl = move |gofn: &Vec<P>, n1: P, n3: P| {
        let g0 = gofn[n1 as usize - 1];
        let g = gofn[n1 as usize];
        let g1 = gofn[n1 as usize + 1];
        let g2 = gofn[n3 as usize - 1];
        let g3 = gofn[n3 as usize];
        let old = cost(g0, g) + cost(g, g1) + cost(g2, g3);
        let new = cost(g0, g1) + cost(g2, g) + cost(g, g3);
        if new < old {
            Some((RotL, n1, n3, old - new))
        } else {
            None
        }
    };

    let rotr = move |gofn: &Vec<P>, n1: P, n3: P| {
        let g0 = gofn[n1 as usize - 1];
        let g1 = gofn[n1 as usize];
        let g2 = gofn[n3 as usize - 2];
        let g = gofn[n3 as usize - 1];
        let g3 = gofn[n3 as usize];
        let old = cost(g0, g1) + cost(g2, g) + cost(g, g3);
        let new = cost(g0, g) + cost(g, g1) + cost(g2, g3);
        if new < old {
            Some((RotR, n1, n3, old - new))
        } else {
            None
        }
    };

    //gofn[1..size as usize - 1].shuffle(&mut rng);
    //gofn[1..size as usize - 1].shuffle(&mut rng);
    let mut set = HashSet::new();

    for tr in 0..10 {
        let mut i = 0;
        while let Some((alg, n1, n3, p)) = {
            let gofnr = &gofn;
            Or::new(
                (1..size - 2).flat_map(|n1| (n1 + 2..size).filter_map(move |n3| rev(gofnr, n1, n3))),
                (1..size - 2).flat_map(|n1| (n1 + 3..size).filter_map(move |n3| rotl(gofnr, n1, n3)))
                    .chain((1..size - 2).flat_map(|n1| (n1 + 3..size).filter_map(move |n3| rotr(gofnr, n1, n3)))),
            )
                .max_by_key(|(_, _, _, p)| *p)
        } {
            let r = &mut gofn[n1 as usize..n3 as usize];
            match alg {
                Rev => r.reverse(),
                RotL => r.rotate_left(1),
                RotR => r.rotate_right(1),
            }
            //draw(&gofn, i);
            i += 1;
        }

        let s: u32 = (1..gofn.len()).map(|i| cost(gofn[i], gofn[i - 1])).sum();
        //println!("{:?} {:?} {:?}", start.elapsed(), i, s);
        draw(&gofn, s);
        if !set.insert(gofn.clone()) {
            println!("!!!!!!clone {:?}", s);
            //draw(&gofn, s);
        }
        gofn[1..size as usize - 1].shuffle(&mut rng);
    }
}

