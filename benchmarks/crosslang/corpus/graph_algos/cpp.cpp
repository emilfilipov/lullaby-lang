// Cross-language graph-algorithm suite (C++). A directed graph on n vertices is a
// flat row-major adjacency matrix g (length n*n): g[r*n + c] == 1 means an edge
// r -> c. Mirrors ../lullaby.lby. See ../SPEC.md.
#include <array>
#include <cstdint>
#include <iostream>
#include <vector>

using Graph = std::vector<int64_t>;

static int64_t has_edge(const Graph &g, int64_t n, int64_t r, int64_t c) { return g[r * n + c]; }

static int64_t out_degree(const Graph &g, int64_t n, int64_t v) {
    int64_t sum = 0;
    for (int64_t c = 0; c < n; c++) sum += g[v * n + c];
    return sum;
}

static int64_t in_degree(const Graph &g, int64_t n, int64_t v) {
    int64_t sum = 0;
    for (int64_t r = 0; r < n; r++) sum += g[r * n + v];
    return sum;
}

static int64_t total_degree(const Graph &g, int64_t n, int64_t v) {
    return out_degree(g, n, v) + in_degree(g, n, v);
}

static int64_t count_self_loops(const Graph &g, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) count += g[i * n + i];
    return count;
}

static int64_t edge_count_directed(const Graph &g, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n * n; i++) count += g[i];
    return count;
}

static int64_t edge_count_undirected(const Graph &g, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = i + 1; j < n; j++)
            if (g[i * n + j] == 1) count += 1;
    return count;
}

static int64_t is_complete_graph(const Graph &g, int64_t n) {
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = 0; j < n; j++)
            if (i != j && g[i * n + j] != 1) return 0;
    return 1;
}

static int64_t is_symmetric(const Graph &g, int64_t n) {
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = 0; j < n; j++)
            if (g[i * n + j] != g[j * n + i]) return 0;
    return 1;
}

static int64_t count_triangles(const Graph &g, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = i + 1; j < n; j++)
            for (int64_t k = j + 1; k < n; k++)
                if (g[i * n + j] == 1 && g[j * n + k] == 1 && g[i * n + k] == 1) count += 1;
    return count;
}

static int64_t max_degree_vertex(const Graph &g, int64_t n) {
    int64_t best_v = 0, best_d = out_degree(g, n, 0);
    for (int64_t v = 1; v < n; v++) {
        int64_t d = out_degree(g, n, v);
        if (d > best_d) { best_d = d; best_v = v; }
    }
    return best_v;
}

static int64_t count_isolated_vertices(const Graph &g, int64_t n) {
    int64_t count = 0;
    for (int64_t v = 0; v < n; v++) if (total_degree(g, n, v) == 0) count += 1;
    return count;
}

static int64_t count_leaves(const Graph &g, int64_t n) {
    int64_t count = 0;
    for (int64_t v = 0; v < n; v++) if (out_degree(g, n, v) == 1) count += 1;
    return count;
}

static int64_t path_exists_len2(const Graph &g, int64_t n, int64_t s, int64_t t) {
    for (int64_t k = 0; k < n; k++)
        if (g[s * n + k] == 1 && g[k * n + t] == 1) return 1;
    return 0;
}

static int64_t common_neighbors_count(const Graph &g, int64_t n, int64_t a, int64_t b) {
    int64_t count = 0;
    for (int64_t k = 0; k < n; k++)
        if (g[a * n + k] == 1 && g[b * n + k] == 1) count += 1;
    return count;
}

static int64_t is_regular(const Graph &g, int64_t n) {
    int64_t d0 = out_degree(g, n, 0);
    for (int64_t v = 1; v < n; v++) if (out_degree(g, n, v) != d0) return 0;
    return 1;
}

static int64_t reachable_count_bfs(const Graph &g, int64_t n, int64_t start) {
    std::array<int64_t, 64> visited{}, queue{};
    int64_t head = 0, tail = 0;
    visited[start] = 1;
    queue[tail++] = start;
    int64_t count = 0;
    while (head < tail) {
        int64_t v = queue[head++];
        count += 1;
        for (int64_t c = 0; c < n; c++)
            if (g[v * n + c] == 1 && visited[c] == 0) { visited[c] = 1; queue[tail++] = c; }
    }
    return count;
}

static int64_t connected_components_count(const Graph &g, int64_t n) {
    std::array<int64_t, 64> visited{}, queue{};
    int64_t components = 0;
    for (int64_t s = 0; s < n; s++) {
        if (visited[s] == 0) {
            components += 1;
            int64_t head = 0, tail = 1;
            visited[s] = 1;
            queue[0] = s;
            while (head < tail) {
                int64_t v = queue[head++];
                for (int64_t c = 0; c < n; c++)
                    if (g[v * n + c] == 1 && visited[c] == 0) { visited[c] = 1; queue[tail++] = c; }
            }
        }
    }
    return components;
}

int main() {
    Graph g = {0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1};
    Graph sym = {0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0};
    Graph comp = {0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0};
    std::cout << (has_edge(g, 6, 0, 2) + out_degree(g, 6, 0) + edge_count_directed(g, 6)
        + count_triangles(sym, 6) + reachable_count_bfs(sym, 6, 0) + connected_components_count(sym, 6)
        + is_complete_graph(comp, 4) + is_regular(comp, 4)) << "\n";
    return 0;
}
