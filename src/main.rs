extern crate rand;
extern crate rand_chacha;
extern crate image;
extern crate imageproc;

use rand::prelude::*;
use std::time::Instant;
use std::collections::HashSet;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_antialiased_line_segment_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use imageproc::pixelops::interpolate;

type Coord = (i32, i32);
type P = u16;
type PP = (u16, u16);

#[derive(Debug)]
struct Clew(Vec<PP>);

impl Clew {
    fn new(size: P) -> Clew {
        Clew((0..size).map(|i| (i, i)).collect())
    }

    fn iter(&self) -> ClewIterator {
        ClewIterator {
            clew: &self,
            current: 0,
            second: false,
        }
    }

    fn yarn(&self, (current, next): PP) -> YarnIterator {
        let x = self.0[current as usize];
        assert!(x.0 == next || x.1 == next, "Not adjancent next [{:?}]={:?} {:?}", current, x, next);
        YarnIterator {
            clew: &self,
            start: current,
            current,
            next,
            moved: false,
        }
    }

    fn cas(&mut self, pos: P, next: P, new: P) {
        let x = &mut self.0[pos as usize];
        if x.0 == next {
            x.0 = new;
        } else {
            assert_eq!(x.1, next);
            x.1 = new;
        }
    }

    fn get_not(&self, pos: P, next: P) -> u16 {
        let x = self.0[pos as usize];
        if x.0 == next {
            x.1
        } else {
            assert_eq!(x.1, next);
            x.0
        }
    }

    fn merge(&mut self, (a, b): PP, (c, d): PP) {
        self.cas(a, b, c);
        self.cas(b, a, d);
        self.cas(c, d, a);
        self.cas(d, c, b);
    }
}

struct ClewIterator<'a> {
    clew: &'a Clew,
    current: P,
    second: bool,
}

impl Iterator for ClewIterator<'_> {
    type Item = PP;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.clew.0.len() as P {
            let i = self.current;
            let (prev, next) = self.clew.0[i as usize];
            let v = if self.second {
                self.current += 1;
                next
            } else { prev };
            self.second = !self.second;
            if i < v || (self.second && i == v) {
                return Some((i, v));
            }
        }
        None
    }
}

struct YarnIterator<'a> {
    clew: &'a Clew,
    start: P,
    current: P,
    next: P,
    moved: bool,
}

impl Iterator for YarnIterator<'_> {
    type Item = PP;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.current && self.moved { None } else {
            self.moved = true;
            let t = self.current;
            self.current = self.next;
            self.next = self.clew.get_not(self.next, t);
            Some(ord((t, self.current)))
        }
    }
}

fn ord<T: Ord>((a, b): (T, T)) -> (T, T) {
    if a <= b { (a, b) } else { (b, a) }
}

fn main() {
    let start = Instant::now();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);

    let size: P = 100;
    let index = |a: P, b: P| a as usize * size as usize + b as usize;
    let width: u32 = 256;
    let points: Vec<Coord> = (0..size).map(|_| (rng.gen_range(0, width as i32), rng.gen_range(0, width as i32))).collect();

    let cost_matrix: Vec<i32> = {
        let mut r = Vec::with_capacity(size as usize * size as usize);
        for a in 0..size {
            for b in 0..size {
                r.push((((points[a as usize].0 - points[b as usize].0) as f64)
                    .hypot((points[a as usize].1 - points[b as usize].1) as f64) * 100f64)
                    .ceil() as i32)
            }
        }
        r
    };

    let cost = |a: P, b: P| cost_matrix[index(a, b)];

    let draw = |clew: &Clew, i: u32| {
        let mut image: RgbImage = RgbImage::new(width, width);
        let black = Rgb([0u8, 0u8, 0u8]);
        let gray = Rgb([200u8, 200u8, 200u8]);
        let white = Rgb([255u8, 255u8, 255u8]);
        let _yellow = Rgb([255u8, 255u8, 0u8]);

        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(width, width), white);

        for p in &points {
            draw_filled_rect_mut(&mut image, Rect::at(p.0 - 1, p.1 - 1).of_size(3, 3), black);
        }

        let coord = |p| {
            points[p as usize]
        };
        for f in clew.iter() {
            draw_antialiased_line_segment_mut(&mut image, coord(f.0), coord(f.1), gray, interpolate);
        }

        image.save(format!("plot/{}.png", i)).unwrap();
    };

    let pot = move |(a, b): PP, (c, d): PP| {
        let acbd = cost(a, c) + cost(b, d);
        let adbc = cost(a, d) + cost(b, c);
        let abcd = cost(a, b) + cost(c, d);
        if acbd <= adbc {
            (acbd - abcd, (c, d))
        } else {
            (adbc - abcd, (d, c))
        }
    };

    let mut clew = Clew::new(size);
    let mut cycle: HashSet<PP> = (0..size).map(|a| (a, a)).collect();

    for m in 0..size - 1 {
        let cl = &clew;
        let (c1, c2, (a1, _a2, (_p, a2x))) = cycle.iter()
            .flat_map(|&c1| cycle.iter()
                .filter(move |&c2| c1 != *c2)
                .map(move |&c2| {
                    let a12px = cl.yarn(c1).
                        flat_map(|a1| cl.yarn(c2).map(move |a2| (a1, a2, pot(a1, a2))))
                        .min_by_key(|(_a1, _a2, (p, _a2x))| *p)
                        .unwrap();
                    (c1, c2, a12px)
                })
            )
            .min_by_key(|(_c1, _c2, (_a1, _a2, (p, _a2x)))| *p)
            .unwrap();
        cycle.remove(&c1);
        cycle.remove(&c2);
        cycle.insert((a1.0, a2x.0));
        clew.merge(a1, a2x);
        //draw(&clew, m as u32);
    }

    let best_cost: i32 = clew.iter().map(|(a, b)| cost(a, b)).sum();

    println!("{:?} {:?}", start.elapsed(), best_cost);
    draw(&clew, 999);
}

