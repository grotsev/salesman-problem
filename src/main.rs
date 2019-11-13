extern crate rand;
extern crate rand_chacha;
extern crate plotters;

use rand::prelude::*;
use plotters::prelude::*;
use std::time::Instant;
use std::collections::BTreeSet;

type Coord = (i32, i32);
type P = u16;


/*fn near(cost: &Array2<u32>) -> Array2<P> {
    let res = Array2::from_shape_fn((N, N), |(a, pos)| pos);
    res.into_i
}*/

fn main() {
    let start = Instant::now();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);

    let size: P = 1000;
    let index = |a: P, b: P| a as usize * size as usize + b as usize;
    let points: Vec<Coord> = (0..size).map(|_| (rng.gen_range(-100, 100), rng.gen_range(-100, 100))).collect();

    let draw = |visits: &Vec<P>| {
        let root = BitMapBackend::new("plot.png", (600, 600)).into_drawing_area()
            .apply_coord_spec(RangedCoord::<RangedCoordi32, RangedCoordi32>::new(-100..100, -100..100, (0..600, 0..600)));
        root.fill(&WHITE).unwrap();
        root.draw(&Path::new(visits.iter().map(|i| points[*i as usize]).collect::<Vec<Coord>>(), &BLACK)
        ).unwrap();
    };

    let cost_matrix: Vec<u32> = {
        let mut r = Vec::with_capacity(size as usize * size as usize);
        for a in 0..size {
            for b in 0..size {
                r.push(((points[a as usize].0 - points[b as usize].0) as f64)
                    .hypot((points[a as usize].1 - points[b as usize].1) as f64)
                    .ceil() as u32)
            }
        }
        r
    };

    let cost = |a: P, b: P| cost_matrix[index(a, b)];

    let mut solution = { // greedy
        let mut r: Vec<P> = Vec::with_capacity(size as usize);
        let mut prev = 0;
        r.push(prev);
        let mut rest: BTreeSet<P> = (1..size).collect();
        while !rest.is_empty() {
            prev = *rest.iter().min_by_key(|next| cost(prev, **next)).unwrap();
            r.push(prev);
            rest.remove(&prev);
        }
        r.push(0);
        r
    };

    let mut solution_cost =
        (1..solution.len()).map(|i| cost(solution[i - 1], solution[i])).sum::<u32>();

    for _ in 0..2000000 {
        let a = rng.gen_range(1, size - 1);
        let b = rng.gen_range(a + 1, size);

        let estimate = |a: P, b: P| {
            let az = solution[a as usize];
            let am = solution[a as usize - 1];
            let bz = solution[b as usize];
            let bp = solution[b as usize + 1];
            (cost(am, az) + cost(bz, bp),
             cost(am, bz) + cost(az, bp))
        };
        let e = estimate(a, b);

        if e.1 <= e.0 {
            solution[a as usize..=b as usize].reverse();
            solution_cost = solution_cost - e.0 + e.1;
        }
    }
    println!("{:?} {:?}", start.elapsed(), solution_cost);
    draw(&solution);
}

