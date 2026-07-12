// Cross-language graph-algorithm suite (JavaScript). A directed graph on n vertices
// is a flat row-major adjacency matrix g (length n*n): g[r*n + c] === 1 means an
// edge r -> c. Mirrors ../lullaby.lby. See ../SPEC.md.

function has_edge(g, n, r, c) {
    return g[r * n + c];
}

function out_degree(g, n, v) {
    let sum = 0;
    for (let c = 0; c < n; c++) sum += g[v * n + c];
    return sum;
}

function in_degree(g, n, v) {
    let sum = 0;
    for (let r = 0; r < n; r++) sum += g[r * n + v];
    return sum;
}

function total_degree(g, n, v) {
    return out_degree(g, n, v) + in_degree(g, n, v);
}

function count_self_loops(g, n) {
    let count = 0;
    for (let i = 0; i < n; i++) count += g[i * n + i];
    return count;
}

function edge_count_directed(g, n) {
    let count = 0;
    for (let i = 0; i < n * n; i++) count += g[i];
    return count;
}

function edge_count_undirected(g, n) {
    let count = 0;
    for (let i = 0; i < n; i++)
        for (let j = i + 1; j < n; j++)
            if (g[i * n + j] === 1) count += 1;
    return count;
}

function is_complete_graph(g, n) {
    for (let i = 0; i < n; i++)
        for (let j = 0; j < n; j++)
            if (i !== j && g[i * n + j] !== 1) return 0;
    return 1;
}

function is_symmetric(g, n) {
    for (let i = 0; i < n; i++)
        for (let j = 0; j < n; j++)
            if (g[i * n + j] !== g[j * n + i]) return 0;
    return 1;
}

function count_triangles(g, n) {
    let count = 0;
    for (let i = 0; i < n; i++)
        for (let j = i + 1; j < n; j++)
            for (let k = j + 1; k < n; k++)
                if (g[i * n + j] === 1 && g[j * n + k] === 1 && g[i * n + k] === 1) count += 1;
    return count;
}

function max_degree_vertex(g, n) {
    let best_v = 0;
    let best_d = out_degree(g, n, 0);
    for (let v = 1; v < n; v++) {
        const d = out_degree(g, n, v);
        if (d > best_d) { best_d = d; best_v = v; }
    }
    return best_v;
}

function count_isolated_vertices(g, n) {
    let count = 0;
    for (let v = 0; v < n; v++) if (total_degree(g, n, v) === 0) count += 1;
    return count;
}

function count_leaves(g, n) {
    let count = 0;
    for (let v = 0; v < n; v++) if (out_degree(g, n, v) === 1) count += 1;
    return count;
}

function path_exists_len2(g, n, s, t) {
    for (let k = 0; k < n; k++)
        if (g[s * n + k] === 1 && g[k * n + t] === 1) return 1;
    return 0;
}

function common_neighbors_count(g, n, a, b) {
    let count = 0;
    for (let k = 0; k < n; k++)
        if (g[a * n + k] === 1 && g[b * n + k] === 1) count += 1;
    return count;
}

function is_regular(g, n) {
    const d0 = out_degree(g, n, 0);
    for (let v = 1; v < n; v++) if (out_degree(g, n, v) !== d0) return 0;
    return 1;
}

function reachable_count_bfs(g, n, start) {
    const visited = new Array(64).fill(0);
    const queue = new Array(64).fill(0);
    let head = 0;
    let tail = 0;
    visited[start] = 1;
    queue[tail++] = start;
    let count = 0;
    while (head < tail) {
        const v = queue[head++];
        count += 1;
        for (let c = 0; c < n; c++)
            if (g[v * n + c] === 1 && visited[c] === 0) { visited[c] = 1; queue[tail++] = c; }
    }
    return count;
}

function connected_components_count(g, n) {
    const visited = new Array(64).fill(0);
    const queue = new Array(64).fill(0);
    let components = 0;
    for (let s = 0; s < n; s++) {
        if (visited[s] === 0) {
            components += 1;
            let head = 0;
            let tail = 1;
            visited[s] = 1;
            queue[0] = s;
            while (head < tail) {
                const v = queue[head++];
                for (let c = 0; c < n; c++)
                    if (g[v * n + c] === 1 && visited[c] === 0) { visited[c] = 1; queue[tail++] = c; }
            }
        }
    }
    return components;
}

const g = [0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1];
const sym = [0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
const comp = [0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0];
console.log(has_edge(g, 6, 0, 2) + out_degree(g, 6, 0) + edge_count_directed(g, 6)
    + count_triangles(sym, 6) + reachable_count_bfs(sym, 6, 0) + connected_components_count(sym, 6)
    + is_complete_graph(comp, 4) + is_regular(comp, 4));
