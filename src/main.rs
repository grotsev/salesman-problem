extern crate rand;
extern crate rand_chacha;
extern crate image;
extern crate imageproc;

use rand::prelude::*;
use std::time::{Instant};
use std::collections::BTreeSet;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_antialiased_line_segment_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use imageproc::pixelops::interpolate;

type Coord = (i32, i32);
type P = u16;

fn main() {
    let start = Instant::now();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);

    let size: P = 100;
    let index = |a: P, b: P| a as usize * size as usize + b as usize;
    let width: u32 = 256;
    let points: Vec<Coord> = (0..size).map(|_| (rng.gen_range(0, width as i32), rng.gen_range(0, width as i32))).collect();

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

    /*let near_matrix: Vec<P> = {
        let mut r = Vec::with_capacity(size as usize * (size as usize - 1));
        for a in 0..size {
            r.extend(0..a);
            r.extend(a + 1..size);
            r[a as usize * (size as usize - 1)..(a as usize + 1) * (size as usize - 1)]
                .sort_by_key(|&b| cost(a, b));
        }
        r
    };

    let near = |a: P| &near_matrix[a as usize * (size as usize - 1)..(a as usize + 1) * (size as usize - 1)];
*/
    let draw = |visits: &Vec<P>, i:u32| {
        let mut image: RgbImage = RgbImage::new(width, width);
        let black = Rgb([0u8, 0u8, 0u8]);
        let gray = Rgb([200u8, 200u8, 200u8]);
        let white = Rgb([255u8, 255u8, 255u8]);
        let yellow = Rgb([255u8, 255u8, 0u8]);

        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(width, width), white);

        for p in &points {
            draw_filled_rect_mut(&mut image, Rect::at(p.0-1, p.1-1).of_size(3, 3), black);
        }
        /*fn m((x, y): (i32, i32)) -> (i32, i32) { (x + 1, y + 2) }
        for i in 0..size {
            let a = m(points[i as usize]);
            let b = m(points[near(i)[0] as usize]);
            draw_antialiased_line_segment_mut(&mut image, a, b, yellow, interpolate);
        }*/

        let coord = |i| {
            points[visits[i as usize] as usize]
        };
        let mut prev = coord(0);
        for i in 1..visits.len() {
            let curr = coord(i);
            draw_antialiased_line_segment_mut(&mut image, prev, curr, gray, interpolate);
            prev = curr;
        }

        image.save(format!("plot/{}.png", i)).unwrap();
    };

    let pot = |a: P, b: P, c: P| cost(a, c) + cost(c, b) - cost(a, b);

    let mut best = Vec::with_capacity(size as usize);
    best.push(0);
    best.push(0);
    let mut best_cost: u32 = 0;
    let mut rest: BTreeSet<P> = (1..size).collect();
    let mut i = 0;
    while !&rest.is_empty() {
        let mut min_pot = std::u32::MAX;
        let mut min_b = std::usize::MAX;
        let mut min_c = std::u16::MAX;
        for b in 1..best.len() {
            for c in &rest {
                let p = pot(best[b - 1], best[b], *c);
                if p < min_pot {
                    min_pot = p;
                    min_b = b;
                    min_c = *c;
                }
            }
        }
        best.insert(min_b, min_c);
        rest.remove(&min_c);
        best_cost += min_pot;
        draw(&best, i);
        i+=1;
    }

    /*let mut best = { // greedy
        let mut r: Vec<P> = Vec::with_capacity(size as usize);
        let mut prev = 0;
        r.push(prev);
        let mut rest: BTreeSet<P> = (1..size).collect();
        while !rest.is_empty() {
            prev = *near(prev).iter().find(|next| rest.contains(next)).unwrap();
            r.push(prev);
            rest.remove(&prev);
        }
        r.push(0);
        r
    };

    let mut best_cost =
        (1..best.len()).map(|i| cost(best[i - 1], best[i])).sum::<u32>();

    for _ in 0..2000000 {
        let a = rng.gen_range(1, size - 1);
        let b = rng.gen_range(a + 1, size);

        let estimate = |a: P, b: P| {
            let az = best[a as usize];
            let am = best[a as usize - 1];
            let bz = best[b as usize];
            let bp = best[b as usize + 1];
            (cost(am, az) + cost(bz, bp),
             cost(am, bz) + cost(az, bp))
        };
        let e = estimate(a, b);

        if e.1 <= e.0 {
            best[a as usize..=b as usize].reverse();
            best_cost = best_cost - e.0 + e.1;
        }
    }*/
    println!("{:?} {:?}", start.elapsed(), best_cost);
    //draw(&best);
}

