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
use plotters::style::RGBAColor;

type Point = (i32, i32);
type P = u16;

const N: P = 1000;

fn annealing(solution: &mut Vec<P>, costMatrix: Array2<u32>) -> u32 {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
    // local u16 pointers

    let cost = |a: P, b: P| {costMatrix[(a as usize, b as usize)]};
    let mut solutionCost = solution.iter()
        .tuple_windows::<(&P, &P)>()
        .map(|(a, b)| cost(*a, *b)).sum::<u32>();

    for i in 0..2000000 {
        let a = rng.gen_range(1, N);
        let b = rng.gen_range(1, N);
        let (a, b) = match a.cmp(&b) {
            Ordering::Less => { (a, b) }
            Ordering::Greater => { (b, a) }
            Ordering::Equal => { continue; }
        };

        let estimate = |a: P, b: P| {
            let az = solution[a as usize];
            let am = solution[a as usize - 1];
            let bz = solution[b as usize];
            let bp = solution[b as usize + 1];
            (cost(am, az) + cost(bz, bp),
             cost(am, bz) + cost(az, bp))
        };

        let e = estimate(a, b);

        if e.1 <= e.0 + 1 {
            solution[a as usize..=b as usize].reverse();
            solutionCost -= e.0;
            solutionCost += e.1;
        }
    }

    solutionCost
}

fn draw(visits: &Vec<u16>, points: Vec<Point>) {
    let root = BitMapBackend::new("plot.png", (600, 600))
        .into_drawing_area()
        .apply_coord_spec(RangedCoord::<RangedCoordi32, RangedCoordi32>::new(-100..100, -100..100, (0..600, 0..600)));
    root.fill(&WHITE);
    root.draw(
        &Path::new(visits.iter()
                       .map(|i| points[*i as usize]).collect::<Vec<Point>>(), &BLACK)
    );
}


fn main() {
    let start = Instant::now();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);

    let points: Vec<Point> = (0..N).map(|_| (rng.gen_range(-100, 100), rng.gen_range(-100, 100)))
        .collect();

    let ds: Array2<u32> = Array2::from_shape_fn(
        (N as usize, N as usize),
        |(a, b)| ((points[a as usize].0 - points[b as usize].0) as f64)
            .hypot((points[a as usize].1 - points[b as usize].1) as f64)
            .ceil() as u32);

    let mut solution: Vec<P> = (0..N).collect();
    solution.push(0);

    let solutionCost = annealing(&mut solution, ds);

    println!("{:?} {:?}", start.elapsed(), solutionCost);
    draw(&solution, points);
}

