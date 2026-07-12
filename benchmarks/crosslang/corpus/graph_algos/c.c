/* Cross-language graph-algorithm suite (C). A directed graph on n vertices is a
 * flat row-major adjacency matrix g (length n*n): g[r*n + c] == 1 means an edge
 * r -> c. Mirrors ../lullaby.lby. See ../SPEC.md. */
#include <stdio.h>

static long long has_edge(const long long *g, long long n, long long r, long long c) {
    return g[r * n + c];
}

static long long out_degree(const long long *g, long long n, long long v) {
    long long sum = 0;
    for (long long c = 0; c < n; c++) sum += g[v * n + c];
    return sum;
}

static long long in_degree(const long long *g, long long n, long long v) {
    long long sum = 0;
    for (long long r = 0; r < n; r++) sum += g[r * n + v];
    return sum;
}

static long long total_degree(const long long *g, long long n, long long v) {
    return out_degree(g, n, v) + in_degree(g, n, v);
}

static long long count_self_loops(const long long *g, long long n) {
    long long count = 0;
    for (long long i = 0; i < n; i++) count += g[i * n + i];
    return count;
}

static long long edge_count_directed(const long long *g, long long n) {
    long long count = 0;
    for (long long i = 0; i < n * n; i++) count += g[i];
    return count;
}

static long long edge_count_undirected(const long long *g, long long n) {
    long long count = 0;
    for (long long i = 0; i < n; i++)
        for (long long j = i + 1; j < n; j++)
            if (g[i * n + j] == 1) count += 1;
    return count;
}

static long long is_complete_graph(const long long *g, long long n) {
    for (long long i = 0; i < n; i++)
        for (long long j = 0; j < n; j++)
            if (i != j && g[i * n + j] != 1) return 0;
    return 1;
}

static long long is_symmetric(const long long *g, long long n) {
    for (long long i = 0; i < n; i++)
        for (long long j = 0; j < n; j++)
            if (g[i * n + j] != g[j * n + i]) return 0;
    return 1;
}

static long long count_triangles(const long long *g, long long n) {
    long long count = 0;
    for (long long i = 0; i < n; i++)
        for (long long j = i + 1; j < n; j++)
            for (long long k = j + 1; k < n; k++)
                if (g[i * n + j] == 1 && g[j * n + k] == 1 && g[i * n + k] == 1) count += 1;
    return count;
}

static long long max_degree_vertex(const long long *g, long long n) {
    long long best_v = 0, best_d = out_degree(g, n, 0);
    for (long long v = 1; v < n; v++) {
        long long d = out_degree(g, n, v);
        if (d > best_d) { best_d = d; best_v = v; }
    }
    return best_v;
}

static long long count_isolated_vertices(const long long *g, long long n) {
    long long count = 0;
    for (long long v = 0; v < n; v++) if (total_degree(g, n, v) == 0) count += 1;
    return count;
}

static long long count_leaves(const long long *g, long long n) {
    long long count = 0;
    for (long long v = 0; v < n; v++) if (out_degree(g, n, v) == 1) count += 1;
    return count;
}

static long long path_exists_len2(const long long *g, long long n, long long s, long long t) {
    for (long long k = 0; k < n; k++)
        if (g[s * n + k] == 1 && g[k * n + t] == 1) return 1;
    return 0;
}

static long long common_neighbors_count(const long long *g, long long n, long long a, long long b) {
    long long count = 0;
    for (long long k = 0; k < n; k++)
        if (g[a * n + k] == 1 && g[b * n + k] == 1) count += 1;
    return count;
}

static long long is_regular(const long long *g, long long n) {
    long long d0 = out_degree(g, n, 0);
    for (long long v = 1; v < n; v++) if (out_degree(g, n, v) != d0) return 0;
    return 1;
}

static long long reachable_count_bfs(const long long *g, long long n, long long start) {
    long long visited[64] = {0}, queue[64] = {0};
    long long head = 0, tail = 0;
    visited[start] = 1;
    queue[tail++] = start;
    long long count = 0;
    while (head < tail) {
        long long v = queue[head++];
        count += 1;
        for (long long c = 0; c < n; c++)
            if (g[v * n + c] == 1 && visited[c] == 0) { visited[c] = 1; queue[tail++] = c; }
    }
    return count;
}

static long long connected_components_count(const long long *g, long long n) {
    long long visited[64] = {0}, queue[64] = {0};
    long long components = 0;
    for (long long s = 0; s < n; s++) {
        if (visited[s] == 0) {
            components += 1;
            long long head = 0, tail = 1;
            visited[s] = 1;
            queue[0] = s;
            while (head < tail) {
                long long v = queue[head++];
                for (long long c = 0; c < n; c++)
                    if (g[v * n + c] == 1 && visited[c] == 0) { visited[c] = 1; queue[tail++] = c; }
            }
        }
    }
    return components;
}

int main(void) {
    long long g[] = {0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1};
    long long sym[] = {0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0};
    long long comp[] = {0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0};
    printf("%lld\n", has_edge(g, 6, 0, 2) + out_degree(g, 6, 0) + edge_count_directed(g, 6)
        + count_triangles(sym, 6) + reachable_count_bfs(sym, 6, 0) + connected_components_count(sym, 6)
        + is_complete_graph(comp, 4) + is_regular(comp, 4));
    return 0;
}
