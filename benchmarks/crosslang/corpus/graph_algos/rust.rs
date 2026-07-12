// Cross-language graph-algorithm suite (Rust). A directed graph on n vertices is a
// flat row-major adjacency matrix g (length n*n): g[r*n + c] == 1 means an edge
// r -> c. Mirrors ../lullaby.lby. See ../SPEC.md.
#![allow(dead_code)]

fn has_edge(g: &[i64], n: i64, r: i64, c: i64) -> i64 {
    g[(r * n + c) as usize]
}

fn out_degree(g: &[i64], n: i64, v: i64) -> i64 {
    (0..n).map(|c| g[(v * n + c) as usize]).sum()
}

fn in_degree(g: &[i64], n: i64, v: i64) -> i64 {
    (0..n).map(|r| g[(r * n + v) as usize]).sum()
}

fn total_degree(g: &[i64], n: i64, v: i64) -> i64 {
    out_degree(g, n, v) + in_degree(g, n, v)
}

fn count_self_loops(g: &[i64], n: i64) -> i64 {
    (0..n).map(|i| g[(i * n + i) as usize]).sum()
}

fn edge_count_directed(g: &[i64], n: i64) -> i64 {
    (0..n * n).map(|i| g[i as usize]).sum()
}

fn edge_count_undirected(g: &[i64], n: i64) -> i64 {
    let mut count = 0;
    for i in 0..n {
        for j in i + 1..n {
            if g[(i * n + j) as usize] == 1 {
                count += 1;
            }
        }
    }
    count
}

fn is_complete_graph(g: &[i64], n: i64) -> i64 {
    for i in 0..n {
        for j in 0..n {
            if i != j && g[(i * n + j) as usize] != 1 {
                return 0;
            }
        }
    }
    1
}

fn is_symmetric(g: &[i64], n: i64) -> i64 {
    for i in 0..n {
        for j in 0..n {
            if g[(i * n + j) as usize] != g[(j * n + i) as usize] {
                return 0;
            }
        }
    }
    1
}

fn count_triangles(g: &[i64], n: i64) -> i64 {
    let mut count = 0;
    for i in 0..n {
        for j in i + 1..n {
            for k in j + 1..n {
                if g[(i * n + j) as usize] == 1
                    && g[(j * n + k) as usize] == 1
                    && g[(i * n + k) as usize] == 1
                {
                    count += 1;
                }
            }
        }
    }
    count
}

fn max_degree_vertex(g: &[i64], n: i64) -> i64 {
    let mut best_v = 0;
    let mut best_d = out_degree(g, n, 0);
    for v in 1..n {
        let d = out_degree(g, n, v);
        if d > best_d {
            best_d = d;
            best_v = v;
        }
    }
    best_v
}

fn count_isolated_vertices(g: &[i64], n: i64) -> i64 {
    (0..n).filter(|&v| total_degree(g, n, v) == 0).count() as i64
}

fn count_leaves(g: &[i64], n: i64) -> i64 {
    (0..n).filter(|&v| out_degree(g, n, v) == 1).count() as i64
}

fn path_exists_len2(g: &[i64], n: i64, s: i64, t: i64) -> i64 {
    for k in 0..n {
        if g[(s * n + k) as usize] == 1 && g[(k * n + t) as usize] == 1 {
            return 1;
        }
    }
    0
}

fn common_neighbors_count(g: &[i64], n: i64, a: i64, b: i64) -> i64 {
    let mut count = 0;
    for k in 0..n {
        if g[(a * n + k) as usize] == 1 && g[(b * n + k) as usize] == 1 {
            count += 1;
        }
    }
    count
}

fn is_regular(g: &[i64], n: i64) -> i64 {
    let d0 = out_degree(g, n, 0);
    for v in 1..n {
        if out_degree(g, n, v) != d0 {
            return 0;
        }
    }
    1
}

fn reachable_count_bfs(g: &[i64], n: i64, start: i64) -> i64 {
    let mut visited = [0i64; 64];
    let mut queue = [0i64; 64];
    let mut head = 0;
    let mut tail = 0;
    visited[start as usize] = 1;
    queue[tail] = start;
    tail += 1;
    let mut count = 0;
    while head < tail {
        let v = queue[head];
        head += 1;
        count += 1;
        for c in 0..n {
            if g[(v * n + c) as usize] == 1 && visited[c as usize] == 0 {
                visited[c as usize] = 1;
                queue[tail] = c;
                tail += 1;
            }
        }
    }
    count
}

fn connected_components_count(g: &[i64], n: i64) -> i64 {
    let mut visited = [0i64; 64];
    let mut queue = [0i64; 64];
    let mut components = 0;
    for s in 0..n {
        if visited[s as usize] == 0 {
            components += 1;
            let mut head = 0;
            let mut tail = 1;
            visited[s as usize] = 1;
            queue[0] = s;
            while head < tail {
                let v = queue[head];
                head += 1;
                for c in 0..n {
                    if g[(v * n + c) as usize] == 1 && visited[c as usize] == 0 {
                        visited[c as usize] = 1;
                        queue[tail] = c;
                        tail += 1;
                    }
                }
            }
        }
    }
    components
}

fn main() {
    let g: [i64; 36] = [0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1];
    let sym: [i64; 36] = [0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
    let comp: [i64; 16] = [0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0];
    println!(
        "{}",
        has_edge(&g, 6, 0, 2) + out_degree(&g, 6, 0) + edge_count_directed(&g, 6)
            + count_triangles(&sym, 6) + reachable_count_bfs(&sym, 6, 0)
            + connected_components_count(&sym, 6) + is_complete_graph(&comp, 4)
            + is_regular(&comp, 4)
    );
}
