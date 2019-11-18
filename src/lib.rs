/*use priority_queue::PriorityQueue;
use std::cmp::Ordering;
use std::collections::HashSet;

type P = usize;

type PP = (P, P);

struct Merge {
    up: bool,
    cost: u32,
    sub: [PP; 2],
    add: [PP; 2],
}

impl Ord for Merge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.up.cmp(&other.up.cmp)
            .then(other.cost.cmp(&self.cost))
    }
}

fn sort(a: [P; 2]) -> [P; 2] {
    if a[0] < a[1] { a } else { [a[1], a[0]] }
}

fn solve(size: P, cost: fn(P, P) -> u32) -> Vec<([P; 2], Merge)> {
    let mut merge: Vec<([P; 2], Merge)> = (0..size).map(|a| [
        ([a, a], Merge { up: false, cost: 0, sub: [(a, a), (a, a)], add: [(a, a), (a, a)] }),
    ]).collect();

    let mut actual: HashSet<P> = (0..size).collect();
    let mut queue: PriorityQueue<[P; 2], Merge> = PriorityQueue::new();
    for a in 0..size - 1 {
        for b in a + 1..size {
            queue.push([a, b], Merge { up: false, cost: 2 * cost(a, b), sub: [(a, a), (b, b)], add: [(a, b), (a, b)] });
        }
    };
    for m in size..2 * size - 1 {
        let reify = queue.pop().unwrap();
        merge.push(reify);
        actual.remove(&reify.0[0]);
        actual.remove(&reify.0[1]);
        for other in actual {
            let trash: ([P; 2], Merge) = reify.0.map(|r| {
                queue.change_priority_by(&sort([r, other]), |mut p| {
                    p.up = true;
                    p
                });
                queue.pop().unwrap()
            }).min_by(|t| t.1.cost);
            queue.push([other, m], Merge {
                up: false,
                cost: 0,
                sub: [],
                add: [],
            });
        }
        actual.push(m);
    };
    merge
}

fn main() {

}
*/
