extern crate rand;
extern crate rand_chacha;
extern crate itertools;
extern crate plotters;
extern crate ndarray;

use rand::prelude::*;
use rand::distributions::Standard;
use std::cmp::Ordering;
use itertools::Itertools;
use plotters::prelude::*;
use std::time::{Duration, Instant};
use ndarray::{Array, Array2};

type Point = (i32, i32);
type Distance = Array2<u32>;


#[derive(Debug, Clone)]
struct Route {
    visits: Vec<u16>,
    length: u32,
}

impl Route {
    fn new(visits: Vec<u16>, d: &Array2<u32>) -> Self {
        Route {
            length: visits.iter().tuple_windows::<(&u16, &u16)>()
                .map(|(a, b)| d[(*a as usize, *b as usize)]).sum::<u32>() + d[(visits.len() - 1, 0)],
            visits,
        }
    }

    fn opt2(&mut self, d: &Array2<u32>, from: u16, to: u16) {
        let (f, t) = match from.cmp(&to) {
            Ordering::Less => { (from, to) }
            Ordering::Greater => { (to, from) }
            Ordering::Equal => { return; }
        };
        if t - f + 2 >= self.visits.len() as u16 { return; }
        self.visits[f as usize..=t as usize].reverse();
        let bf = self.visits[if f > 0 { f - 1 } else { self.visits.len() as u16 - 1 } as usize] as usize;
        let at = self.visits[if t as usize + 1 < self.visits.len() { t + 1 } else { 0 } as usize] as usize;
        let xf = self.visits[f as usize] as usize;
        let xt = self.visits[t as usize] as usize;
        self.length = self.length - d[(bf, xt)] - d[(xf, at)] + d[(bf, xf)] + d[(xt, at)];
    }

    fn draw(&self, points: Vec<Point>) {
        let root = BitMapBackend::new("plot.png", (600, 600))
            .into_drawing_area()
            .apply_coord_spec(RangedCoord::<RangedCoordi32, RangedCoordi32>::new(-100..100, -100..100, (0..600, 0..600)));
        root.fill(&WHITE);
        root.draw(
            &Path::new(self.visits.iter()
                           .map(|i| points[*i as usize]).collect::<Vec<Point>>(), &BLACK)
        );
    }
}

fn main() {
    let start = Instant::now();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);

    let n = 1000_u16;
    let points: Vec<Point> = (0..n).map(|_| (rng.gen_range(-100, 100), rng.gen_range(-100, 100)))
        .collect();

    let ds: Array2<u32> = Array2::from_shape_fn(
        (n as usize, n as usize),
        |(a, b)| ((points[a as usize].0 - points[b as usize].0) as f64)
            .hypot((points[a as usize].1 - points[b as usize].1) as f64)
            .ceil() as u32);

    let mut best = Route::new((0..n).collect(), &ds);
    let mut current = best.clone();

    for i in 0..2000000 {
        let from = rng.gen_range(0, n);
        let to = rng.gen_range(0, n);
        if from == to { continue; }

        current.opt2(&ds, from, to);

        if current.length <= best.length + 1 {
            best = current.clone();
        } else {
            current = best.clone();
        }
    }

    println!("{:?} {:?}", start.elapsed(), best.length);
    best.draw(points);
}

