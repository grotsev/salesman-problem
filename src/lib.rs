/*extern crate ndarray;

use ndarray::Array2;

type VisitId = u16;
type ChainId = u16;

struct Cost(Array2<u32>);

impl Fn<> for Cost {
    fn get(&self, i: (VisitId, VisitId)) -> u32 {
        self.0[(a as usize, b as usize)]
    }
}

type Route = Vec<VisitId>;

fn estimate_swap(route: &Route, cost: Cost, a: usize, b: usize) -> (u32, u32) {
    let az = route[a] as usize;
    let am = route[a - 1] as usize;
    let ap = route[a + 1] as usize;
    let bz = route[b] as usize;
    let bm = route[b - 1] as usize;
    let bp = route[b + 1] as usize;
    (cost[(am, az)] + cost[(az, ap)] + cost[(bm, bz)] + cost[(bz, bp)],
     cost[(am, bz)] + cost[(bz, ap)] + cost[(bm, az)] + cost[(az, bp)])
}

fn estimate_remove_insert(route: &Route, cost: Cost, a: usize, b: usize) -> (u32, u32) {
    let az = route[a] as usize;
    let am = route[a - 1] as usize;
    let ap = route[a + 1] as usize;
    let bz = route[b] as usize;
    let bm = route[b - 1] as usize;
    (cost[(am, az)] + cost[(az, ap)] + cost[(bm, bz)],
     cost[(bm, az)] + cost[(az, bz)] + cost[(am, ap)])
}

fn estimate_reverse(route: &Route, cost: Cost, a: usize, b: usize) -> (u32, u32) {
    let az = route[a] as usize;
    let am = route[a - 1] as usize;
    let bz = route[b] as usize;
    let bp = route[b + 1] as usize;
    (cost[(am, az)] + cost[(bz, bp)],
     cost[(am, bz)] + cost[(az, bp)])
}

fn remove_insert(route: &mut Route, a: usize, b: usize) {
    let t = route[a];
    if a < b {
        route.copy_within(a + 1..=b, a);
    } else {
        route.copy_within(b..a, b + 1);
    }
    route[b] = t;
}

fn reverse(route: &mut Route, a: usize, b: usize) {
    route[a..=b].reverse();
}

// fn swap(&mut self, a: usize, b: usize)

//fn scramble() {}
//fn pfih() {}
*/