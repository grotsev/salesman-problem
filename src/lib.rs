/*use priority_queue::PriorityQueue;
use std::cmp::{Ordering, Reverse};
use std::collections::HashSet;

type P = usize;

type PP = (P, P);

fn ord(a: P, b: P) -> (P, P) {
    if a < b { (a, b) } else { (b, a) }
}

fn solve(size: P, cost: fn(P, P) -> u32) -> Vec<([P; 2], Merge)> {
    let mut cyc: Vec<HashSet<PP>> = (0..size).map(|a| {
        let mut set = HashSet::new();
        set.insert((a, a));
        set
    }).collect();

    let mut actual: HashSet<P> = (0..size).collect();
    let mut queue: PriorityQueue<PP, Reverse<u32>> = PriorityQueue::new();
    for a in 0..size - 1 {
        for b in a + 1..size {
            queue.push((a, b), Reverse(2 * cost(a, b)));
        }
    };

    let union = |a: &HashSet<PP>, b: &HashSet<PP>| -> HashSet<PP> {

    }

    let pot = |a: &HashSet<PP>, b: &HashSet<PP>| -> Reverse<u32> {

    }

    for m in size..2 * size - 1 {
        let ((a, b), _) = queue.pop().unwrap();
        cyc.push(union(&cyc[a], &cyc[b]));
        actual.remove(&a);
        actual.remove(&b);
        for other in actual {
            queue.change_priority(&ord(a, other), Reverse(0));
            queue.change_priority(&ord(b, other), Reverse(0));
            queue.pop().unwrap();
            queue.pop().unwrap();
            queue.push((other, m), pot(&cyc[other], &cyc[m]));
        }
        actual.push(m);
    };
    merge
}
*/
