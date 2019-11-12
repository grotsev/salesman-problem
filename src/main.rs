extern crate rand;
extern crate rand_chacha;
extern crate plotters;
extern crate ndarray;

use rand::prelude::*;
use std::cmp::Ordering;
use plotters::prelude::*;
use std::time::Instant;
use ndarray::Array2;

type Point = (i32, i32);
type P = u16;

const N: P = 1000;

fn annealing<R>(rng: &mut R, solution: &mut Vec<P>, cost_matrix: Array2<u32>) -> u32 where R: Rng {
    let cost = |a: P, b: P| { cost_matrix[(a as usize, b as usize)] };
    let mut solution_cost = (1..solution.len())
        .map(|i| cost(solution[i - 1], solution[i])).sum::<u32>();

    for _ in 0..2000000 {
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
            solution_cost = solution_cost - e.0 + e.1;
        }
    }

    solution_cost
}

fn draw(visits: &Vec<u16>, points: Vec<Point>) {
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

    let ds: Array2<u32> = Array2::from_shape_fn(
        (N as usize, N as usize),
        |(a, b)| ((points[a].0 - points[b].0) as f64)
            .hypot((points[a].1 - points[b].1) as f64)
            .ceil() as u32);

    let mut solution: Vec<P> = (0..N).collect();
    solution.push(0);

    let solution_cost = annealing(&mut rng, &mut solution, ds);

    println!("{:?} {:?}", start.elapsed(), solution_cost);
    draw(&solution, points);
}

