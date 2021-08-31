use rayon::prelude::*;
use sphere_geometry::adjacency::Adjacency;

fn main() {
    println!("run started, it usually takes ~9 s...");
    let start = std::time::Instant::now();
    (16..1024).into_par_iter().for_each(|count| {
        let mut adjacency = Adjacency::default();
        let adjacency = adjacency.get(count);
        for (node, adj) in adjacency.iter().enumerate() {
            for neighbour in adj {
                if neighbour > node {
                    let n_adj = adjacency[neighbour];
                    // all adjacent nodes share at least two neighbours
                    assert!(
                        adj.and(n_adj).len() >= 2,
                        "nodes: {},\n{}: {},\n{}: {},\n{}",
                        count,
                        node,
                        adjacency[node],
                        neighbour,
                        adjacency[neighbour],
                        n_adj.and(*adj)
                    );
                }
            }
        }
    });
    let end = std::time::Instant::now();
    let duration = end - start;
    println!("finished in {} s", duration.as_secs());
}

#[test]
fn test() {
    main();
}
