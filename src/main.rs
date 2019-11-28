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

struct Problem {
    size: u16,
    cost_matrix: Vec<u32>,
}

impl Problem {
    fn new(points: &Vec<(i32, i32)>) -> Problem {
        let size = points.len();
        let mut cost_matrix = Vec::with_capacity(size * size);
        for a in 0..size {
            for b in 0..size {
                cost_matrix.push((100f64 * ((points[a].0 - points[b].0) as f64)
                    .hypot((points[a].1 - points[b].1) as f64))
                    .ceil() as u32)
            }
        }
        Problem { size: size as u16, cost_matrix }
    }

    fn cost(&self, a: u16, b: u16) -> u32 {
        self.cost_matrix[a as usize * self.size as usize + b as usize]
    }
}

struct Solution<'a> {
    problem: &'a Problem,
    perm: Vec<u16>,
}

impl<'a> Solution<'a> {
    fn new(problem: &Problem) -> Solution {
        Solution { problem, perm: (0..problem.size).collect() }
    }

    fn cost(&self) -> u32 {
        (1..self.perm.len()).map(|i| self.problem.cost(self.perm[i], self.perm[i - 1])).sum()
    }

    fn local(&mut self) -> u32 {
        while Reverse::optimize(self)
            || Rotate::optimize(self) {}
        self.cost()
    }

    fn global<R>(&mut self, times: usize, rng: &mut R) -> u32
        where R: Rng + ?Sized {
        let mut best_cost = self.local();
        let mut best_perm = self.perm.clone();

        for _ in 1..times {
            self.perm[1..self.problem.size as usize - 1].shuffle(rng);
            let cost = self.local();
            if best_cost > cost {
                best_cost = cost;
                best_perm.copy_from_slice(&self.perm);
            }
        }
        best_cost
    }
}

trait Optimization {
    type Neighbor;

    fn optimize(solution: &mut Solution) -> bool {
        let mut optimized = false;
        while let Some(best_neighbor) = Self::best_neighbor(solution) {
            Self::apply_neighbor(solution, best_neighbor);
            optimized = true;
        }
        return optimized;
    }

    fn best_neighbor(solution: &Solution) -> Option<Self::Neighbor>;

    fn apply_neighbor(solution: &mut Solution, neighbor: Self::Neighbor);
}

struct Reverse;

impl Optimization for Reverse {
    type Neighbor = (u16, u16);

    fn best_neighbor(solution: &Solution) -> Option<Self::Neighbor> {
        let size = solution.problem.size;
        (1..size - 2).flat_map(|n_clip| (n_clip + 2..size)
            .filter_map(move |n_cut| {
                let cut0 = solution.perm[n_clip as usize - 1];
                let clip0 = solution.perm[n_clip as usize];
                let clip1 = solution.perm[n_cut as usize - 1];
                let cut1 = solution.perm[n_cut as usize];
                let old = solution.problem.cost(cut0, clip0) + solution.problem.cost(clip1, cut1);
                let new = solution.problem.cost(cut0, clip1) + solution.problem.cost(clip0, cut1);
                if new < old {
                    Some((old - new, n_clip, n_cut))
                } else {
                    None
                }
            }))
            .max_by_key(|(p, _, _)| *p)
            .map(|(_, n_clip, n_cut)| (n_clip, n_cut))
    }

    fn apply_neighbor(solution: &mut Solution, (n_clip, n_cut): Self::Neighbor) {
        solution.perm[n_clip as usize..n_cut as usize].reverse()
    }
}

struct Rotate;

impl Optimization for Rotate {
    type Neighbor = (bool, u16, u16, u16);

    fn best_neighbor(solution: &Solution) -> Option<Self::Neighbor> {
        const WIDTH: u16 = 3;
        let size = solution.problem.size;
        (1..size - 3).flat_map(|n_clip|
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
            let cut0 = solution.perm[n_clip as usize - 1];
            let clip0 = solution.perm[n_clip as usize];
            let clip1 = solution.perm[n_cut as usize - 1];
            let cut1 = solution.perm[n_cut as usize];
            let paste0 = solution.perm[n_paste as usize - 1];
            let paste1 = solution.perm[n_paste as usize];

            let old = solution.problem.cost(cut0, clip0) + solution.problem.cost(clip1, cut1) + solution.problem.cost(paste0, paste1);
            let direct = solution.problem.cost(paste0, clip0) + solution.problem.cost(clip1, paste1);
            let reverse = solution.problem.cost(paste0, clip1) + solution.problem.cost(clip0, paste1);
            let do_reverse = reverse < direct;
            let new = solution.problem.cost(cut0, cut1) + if do_reverse { reverse } else { direct };

            if new < old {
                Some((old - new, do_reverse, n_clip, n_cut, n_paste))
            } else {
                None
            }
        })
            .max_by_key(|(p, _, _, _, _)| *p)
            .map(|(_, do_reverse, n_clip, n_cut, n_paste)| (do_reverse, n_clip, n_cut, n_paste))
    }

    fn apply_neighbor(solution: &mut Solution, (do_reverse, n_clip, n_cut, n_paste): Self::Neighbor) {
        if do_reverse {
            solution.perm[n_clip as usize..n_cut as usize].reverse();
        }
        if n_clip < n_paste {
            solution.perm[n_clip as usize..n_paste as usize].rotate_left((n_cut - n_clip) as usize)
        } else {
            solution.perm[n_paste as usize..n_cut as usize].rotate_right((n_cut - n_clip) as usize)
        }
    }
}

const IMAGE_WIDTH: u32 = 256;
const TEXT_HEIGHT: f32 = 10.0;

fn main() {
    let font = FontCollection::from_bytes(include_bytes!("DejaVuSans.ttf") as &[u8]).unwrap().into_font().unwrap();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
    let points: Vec<(i32, i32)> = (0..100).map(|_| (rng.gen_range(0, IMAGE_WIDTH as i32), rng.gen_range(0, IMAGE_WIDTH as i32))).collect();

    let draw = |gofn: &Vec<u16>, i: u32| {
        let mut image: RgbImage = RgbImage::new(IMAGE_WIDTH, IMAGE_WIDTH);
        let city = Rgb([0u8, 0u8, 0u8]);
        let title = Rgb([0u8, 100u8, 200u8]);
        let road = Rgb([200u8, 100u8, 0u8]);
        let ground = Rgb([255u8, 255u8, 255u8]);
        draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(IMAGE_WIDTH, IMAGE_WIDTH), ground);
        let scale = Scale { x: TEXT_HEIGHT, y: TEXT_HEIGHT };
        for (i, p) in points.iter().enumerate() {
            draw_text_mut(&mut image, title, p.0 as u32 + 2, p.1 as u32, scale, &font, &i.to_string());
        }
        for i in 1..gofn.len() {
            draw_antialiased_line_segment_mut(&mut image, points[gofn[i - 1] as usize], points[gofn[i] as usize], road, interpolate);
        }
        for p in points.iter() {
            draw_filled_rect_mut(&mut image, Rect::at(p.0, p.1).of_size(2, 2), city);
        }
        image.save(format!("plot/{}.png", i)).unwrap();
    };

    let problem = Problem::new(&points);
    let mut solution = Solution::new(&problem);
    let start = Instant::now();
    let cost = solution.global(5, &mut rng);
    println!("{:?} {:?}", start.elapsed(), cost);
    draw(&solution.perm, 999);
}


