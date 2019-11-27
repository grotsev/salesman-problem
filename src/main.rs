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
use std::path::Iter;
use std::collections::HashSet;

type Coord = (i32, i32);
type P = u16;
type PP = (u16, u16);

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
        let yellow = Rgb([240u8, 240u8, 120u8]);
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

    let len = |gofn: &Vec<u16>| -> u32 {
        (1..gofn.len()).map(|i| cost(gofn[i], gofn[i - 1])).sum()
    };

    let mut gofn: Vec<P> = (0..size).collect();

    //gofn[1..size as usize - 1].shuffle(&mut rng);
    //gofn[1..size as usize - 1].shuffle(&mut rng);
    let mut set = HashSet::new();

    let mut i = 0;
    loop {
        i += 1;
        println!("{:?}", i);
        let max = (1..size - 2).flat_map(|n_clip| {
            let gofn = &gofn;
            (n_clip + 2..size)
                .filter_map(move |n_cut| {
                    let cut0 = gofn[n_clip as usize - 1];
                    let clip0 = gofn[n_clip as usize];
                    let clip1 = gofn[n_cut as usize - 1];
                    let cut1 = gofn[n_cut as usize];
                    let old = cost(cut0, clip0) + cost(clip1, cut1);
                    let new = cost(cut0, clip1) + cost(clip0, cut1);
                    if new < old {
                        Some((n_clip, n_cut, old - new))
                    } else {
                        None
                    }
                })
        }
        )
            .max_by_key(|(_, _, p)| *p)
            .map(|(n_clip, n_cut, _)| gofn[n_clip as usize..n_cut as usize].reverse());
        if max.is_some() {
            continue;
        };

        println!("XXX {:?}", len(&gofn));

        const WIDTH: P = 3;
        let max = (1..size - 3).flat_map(|n_clip|
            (n_clip + 1..(n_clip + WIDTH).min(size - 3)).flat_map(move |n_cut|
                (n_cut + 2..size).map(move |n_paste| (n_clip, n_cut, n_paste))
            )
        ).chain(
            (3..size - 1).flat_map(|n_clip|
                (n_clip + 1..(n_clip + WIDTH).min(size - 3)).flat_map(move |n_cut|
                    (1..n_clip - 1).map(move |n_paste| (n_clip, n_cut, n_paste))
                )
            )
        ).filter_map(|(n_clip, n_cut, n_paste)| {
            let cut0 = gofn[n_clip as usize - 1];
            let clip0 = gofn[n_clip as usize];
            let clip1 = gofn[n_cut as usize - 1];
            let cut1 = gofn[n_cut as usize];
            let paste0 = gofn[n_paste as usize - 1];
            let paste1 = gofn[n_paste as usize];

            let old = cost(cut0, clip0) + cost(clip1, cut1) + cost(paste0, paste1);
            let direct = cost(paste0, clip0) + cost(clip1, paste1);
            let reverse = cost(paste0, clip1) + cost(clip0, paste1);
            let do_reverse = reverse < direct;
            let new = cost(cut0, cut1) + if do_reverse { reverse } else { direct };

            if new < old {
                Some((do_reverse, n_clip, n_cut, n_paste, old - new))
            } else {
                None
            }
        })
            .max_by_key(|(_, _, _, _, p)| *p)
            .map(|(do_reverse, n_clip, n_cut, n_paste, p)| {
                println!("{:?}", (do_reverse, n_clip, n_cut, n_paste, p));
                if do_reverse {
                    gofn[n_clip as usize..n_cut as usize].reverse();
                }
                if n_clip < n_paste {
                    gofn[n_clip as usize..n_paste as usize].rotate_left((n_cut - n_clip) as usize)
                } else {
                    gofn[n_paste as usize..n_cut as usize].rotate_right((n_cut - n_clip) as usize)
                }
            });
        if max.is_none() { break; }
    }

    let s: u32 = len(&gofn);
    println!("{:?} {:?} {:?}", start.elapsed(), i, s);
    draw(&gofn, 999);
    if !set.insert(gofn.clone()) {
        println!("!!!!!!clone {:?}", s);
        //draw(&gofn, s);
    }
    gofn[1..size as usize - 1].shuffle(&mut rng);
}


