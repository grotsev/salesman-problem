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

type Coord = (i32, i32);
type P = u16;
type PP = (u16, u16);

fn ord<T: Ord>((a, b): (T, T)) -> (T, T) {
    if a <= b { (a, b) } else { (b, a) }
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
        //let gray = Rgb([200u8, 200u8, 200u8]);
        let red = Rgb([100u8, 0u8, 0u8]);
        let white = Rgb([255u8, 255u8, 255u8]);
        let height = 12.4;
        let scale = Scale {
            x: height,
            y: height,
        };
        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(width, width), white);
        for (i, p) in points.iter().enumerate() {
            draw_filled_rect_mut(&mut image, Rect::at(p.0 - 1, p.1 - 1).of_size(3, 3), black);
            draw_text_mut(&mut image, black, p.0 as u32 + 2, p.1 as u32, scale, &font, &i.to_string());
        }
        for i in 1..gofn.len() {
            draw_antialiased_line_segment_mut(&mut image, points[gofn[i - 1] as usize], points[gofn[i] as usize], red, interpolate);
        }
        image.save(format!("plot/{}.png", i)).unwrap();
    };

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

    let mut nofg: Vec<P> = (0..size).collect();
    let mut gofn: Vec<P> = (0..size).collect();

    let pot = move |nofg: &Vec<P>, gofn: &Vec<P>, g1, g2| {
        if g1 == g2 {
            return None;
        }
        let n1 = nofg[g1 as usize];
        let n2 = nofg[g2 as usize];
        let (n1, g1, n2, g2) = if n1 < n2 { (n1, g1, n2, g2) } else { (n2, g2, n1, g1) };
        if n1 == 0 || n2 >= size - 2 {
            return None;
        }
        let n0 = n1 - 1;
        let n3 = n2 + 1;
        let g0 = gofn[n0 as usize];
        let g3 = gofn[n3 as usize];

        let old = cost(g0, g1) + cost(g2, g3);
        let new = cost(g0, g2) + cost(g1, g3);
        if new < old {
            Some((ord((g1, g2)), (old - new) as u32))
        } else {
            None
        }
    };

    let nofgr = &nofg;
    let gofnr = &gofn;
    let mut pofg1g2: PriorityQueue<PP, u32> = (1..size - 2)
        .flat_map(|g1| (g1 + 1..size - 1).filter_map(move |g2| pot(nofgr, gofnr, g1, g2)))
        .collect();

    //println!("{:?}", pofg1g2);

    let mut i = 0;
    println!("{:?}", pofg1g2);
    draw(&gofn, i);
    i += 1;
    while let Some(((g1, g2), p)) = pofg1g2.pop() {
        println!("================== {:?}", i);
        let n1 = nofg[g1 as usize];
        let n2 = nofg[g2 as usize];
        let (n1, g1, n2, g2) = if n1 < n2 { (n1, g1, n2, g2) } else { (n2, g2, n1, g1) };
        let n0 = n1 - 1;
        let n3 = n2 + 1;
        let n1_n2 = n1 + n2;
        let g0 = gofn[n0 as usize];
        let g3 = gofn[n3 as usize];
        println!("{:?} {:?} {:?} {:?} {:?}", g0, g1, g2, g3, p);
        //assert_eq!((ord((g1, g2)), p), pot(g0, g1, g2, g3).unwrap());

        for g in 1..size - 1 {
            for &gi in &[g0, g1, g2, g3] {
                pot(&nofg, &gofn, gi, g).map(|((g1, g2), _p)| {
                    pofg1g2.push((g1, g2), std::u32::MAX);
                    assert_eq!((g1, g2), *pofg1g2.peek().unwrap().0);
                    pofg1g2.pop();
                });
            }
        }

        (n1..n3).for_each(|n| {
            let g = gofn[n as usize] as usize;
            assert_eq!(nofg[g], n); // TODO remove
            nofg[g] = n1_n2 - nofg[g]; // TODO try performance n1_n3 - n
        });
        gofn[n1 as usize..n3 as usize].reverse();
        println!("{:?}", gofn);

        for g in 1..size - 1 {
            for &gi in &[g0, g1, g2, g3] {
                pot(&nofg, &gofn, gi, g).map(|((g1, g2), p)| {
                    pofg1g2.push((g1, g2), p);
                });
            }
        }

        println!("{:?}", pofg1g2);
        draw(&gofn, i);
        i += 1;
    }

    println!("{:?}", start.elapsed());
    //draw(&gofn, 999);
}

