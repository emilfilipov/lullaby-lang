# Cross-language graph-algorithm suite (Python). A directed graph on n vertices is
# a flat row-major adjacency matrix `g` (list of length n*n): g[r*n + c] == 1 means
# an edge r -> c. Mirrors ../lullaby.lby; results are ints for cross-language
# uniformity. See ../SPEC.md.


def has_edge(g, n, r, c):
    return g[r * n + c]


def out_degree(g, n, v):
    return sum(g[v * n + c] for c in range(n))


def in_degree(g, n, v):
    return sum(g[r * n + v] for r in range(n))


def total_degree(g, n, v):
    return out_degree(g, n, v) + in_degree(g, n, v)


def count_self_loops(g, n):
    return sum(g[i * n + i] for i in range(n))


def edge_count_directed(g, n):
    return sum(g[i] for i in range(n * n))


def edge_count_undirected(g, n):
    count = 0
    for i in range(n):
        for j in range(i + 1, n):
            if g[i * n + j] == 1:
                count += 1
    return count


def is_complete_graph(g, n):
    for i in range(n):
        for j in range(n):
            if i != j and g[i * n + j] != 1:
                return 0
    return 1


def is_symmetric(g, n):
    for i in range(n):
        for j in range(n):
            if g[i * n + j] != g[j * n + i]:
                return 0
    return 1


def count_triangles(g, n):
    count = 0
    for i in range(n):
        for j in range(i + 1, n):
            for k in range(j + 1, n):
                if g[i * n + j] == 1 and g[j * n + k] == 1 and g[i * n + k] == 1:
                    count += 1
    return count


def max_degree_vertex(g, n):
    best_v = 0
    best_d = out_degree(g, n, 0)
    for v in range(1, n):
        d = out_degree(g, n, v)
        if d > best_d:
            best_d = d
            best_v = v
    return best_v


def count_isolated_vertices(g, n):
    return sum(1 for v in range(n) if total_degree(g, n, v) == 0)


def count_leaves(g, n):
    return sum(1 for v in range(n) if out_degree(g, n, v) == 1)


def path_exists_len2(g, n, s, t):
    for k in range(n):
        if g[s * n + k] == 1 and g[k * n + t] == 1:
            return 1
    return 0


def common_neighbors_count(g, n, a, b):
    count = 0
    for k in range(n):
        if g[a * n + k] == 1 and g[b * n + k] == 1:
            count += 1
    return count


def is_regular(g, n):
    d0 = out_degree(g, n, 0)
    for v in range(1, n):
        if out_degree(g, n, v) != d0:
            return 0
    return 1


def reachable_count_bfs(g, n, start):
    visited = [0] * 64
    queue = [0] * 64
    head = 0
    tail = 0
    visited[start] = 1
    queue[tail] = start
    tail += 1
    count = 0
    while head < tail:
        v = queue[head]
        head += 1
        count += 1
        for c in range(n):
            if g[v * n + c] == 1 and visited[c] == 0:
                visited[c] = 1
                queue[tail] = c
                tail += 1
    return count


def connected_components_count(g, n):
    visited = [0] * 64
    queue = [0] * 64
    components = 0
    for s in range(n):
        if visited[s] == 0:
            components += 1
            head = 0
            visited[s] = 1
            queue[0] = s
            tail = 1
            while head < tail:
                v = queue[head]
                head += 1
                for c in range(n):
                    if g[v * n + c] == 1 and visited[c] == 0:
                        visited[c] = 1
                        queue[tail] = c
                        tail += 1
    return components


if __name__ == "__main__":
    g = [0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1]
    sym = [0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    comp = [0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0]
    assert has_edge(g, 6, 0, 2) == 1
    assert out_degree(g, 6, 0) == 2
    assert edge_count_directed(g, 6) == 8
    assert is_symmetric(g, 6) == 0
    assert count_triangles(sym, 6) == 1
    assert reachable_count_bfs(sym, 6, 0) == 3
    assert connected_components_count(sym, 6) == 3
    assert is_complete_graph(comp, 4) == 1
    assert is_regular(comp, 4) == 1
    print("ok")
