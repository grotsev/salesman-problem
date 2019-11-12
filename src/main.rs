extern crate rand;
extern crate rand_chacha;
extern crate plotters;
extern crate ndarray;

use rand::prelude::*;
use plotters::prelude::*;
use std::time::Instant;
use ndarray::Array2;
use std::collections::BTreeSet;

type Point = (i32, i32);
type P = u16;

const N: P = 1000;

fn arc(cost: &Array2<u32>, a: P, b: P) -> u32 {
    cost[(a as usize, b as usize)]
}

fn vec(cost: &Array2<u32>, solution: &Vec<P>) -> u32 {
    (1..solution.len()).map(|i| arc(cost, solution[i - 1], solution[i])).sum::<u32>()
}

fn greedy(cost: &Array2<u32>) -> Vec<P> {
    let mut prev = 0;
    let mut solution: Vec<P> = Vec::with_capacity(N as usize);
    solution.push(prev);
    let mut rest: BTreeSet<P> = (1..N).collect();
    while !rest.is_empty() {
        prev = *rest.iter().min_by_key(|next| arc(cost, prev, **next)).unwrap();
        solution.push(prev);
        rest.remove(&prev);
    }
    solution
}

fn annealing<R>(rng: &mut R, solution: &mut Vec<P>, cost: &Array2<u32>) -> u32 where R: Rng {
    let c = |a: P, b: P| { arc(cost, a, b) };
    let mut solution_cost = vec(cost, solution);

    for _ in 0..2000000 {
        let a = rng.gen_range(1, N-1);
        let b = rng.gen_range(a+1, N);

        let estimate = |a: P, b: P| {
            let az = solution[a as usize];
            let am = solution[a as usize - 1];
            let bz = solution[b as usize];
            let bp = solution[b as usize + 1];
            (c(am, az) + c(bz, bp),
             c(am, bz) + c(az, bp))
        };
        let e = estimate(a, b);

        if e.1 <= e.0 {
            solution[a as usize..=b as usize].reverse();
            solution_cost = solution_cost - e.0 + e.1;
        }
    }

    solution_cost
}

fn draw(visits: &Vec<P>, points: Vec<Point>) {
    let root = BitMapBackend::new("plot.png", (600, 600)).into_drawing_area()
        .apply_coord_spec(RangedCoord::<RangedCoordi32, RangedCoordi32>::new(-100..100, -100..100, (0..600, 0..600)));
    root.fill(&WHITE).unwrap();
    root.draw(&Path::new(visits.iter().map(|i| points[*i as usize]).collect::<Vec<Point>>(), &BLACK)
    ).unwrap();
}


fn main() {
    let start = Instant::now();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
    let points: Vec<Point> = (0..N)
        .map(|_| (rng.gen_range(-100, 100), rng.gen_range(-100, 100))).collect();

    let cost: Array2<u32> = Array2::from_shape_fn(
        (N as usize, N as usize),
        |(a, b)| ((points[a].0 - points[b].0) as f64)
            .hypot((points[a].1 - points[b].1) as f64)
            .ceil() as u32);

    //let mut solution: Vec<P> = (0..N).collect();
    let mut solution = greedy(&cost);
    solution.push(0);
    let solution_cost = annealing(&mut rng, &mut solution, &cost);
    println!("{:?} {:?}", start.elapsed(), solution_cost);
    let solution_cost = vec(&cost, &solution);
    println!("{:?} {:?}", start.elapsed(), solution_cost);
    draw(&solution, points);
}

