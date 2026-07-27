//! CLI integration tests — native **RETURNED-CLOSURE** support (safe-tier arena
//! stage-4, increment **a**: the invoke-plumbing prerequisite).
//!
//! # What this increment unblocks
//!
//! Before it, a function that RETURNS a closure and a caller that INVOKES a
//! call-returned closure both skipped native: `native_signature_eligibility` refused
//! a `fn(...)` return type, and `indirect_callable_sig` could not resolve a
//! call-returned `fn` local — so the whole function demoted to the interpreters. This
//! increment makes them compile:
//!
//! - a `fn(...)` RETURN is admitted as a single `I64` block-pointer word (`rax`),
//!   gated tightly to a **locally-created closure literal** (fresh, flat,
//!   scalar-capture) — a returned fn PARAMETER (aliases a caller's env) or a
//!   call-returned closure (increment b) stays refused;
//! - a `let g fn(...) = make_adder(5)` local becomes an **indirect callable**, so
//!   `g(3)` lowers through the existing closure indirect-call ABI (env pointer in
//!   `rcx`, `mov rax,[rcx]; call rax`).
//!
//! # Soundness (increment a does NOT touch arena reclamation)
//!
//! The returning factory stays **off the arena** (it returns a `fn`/heap value, so
//! `arena_eligible_functions` refuses it), so its return-edge never rewinds and the
//! returned `[code_ptr][captures…]` block stays live on the growing heap — the caller
//! can invoke it with no dangling. `native_returned_closure_survives_heap_churn` pins
//! that: a factory-captured value read back after the caller allocates heavily still
//! reads correctly, which it could not if the block had been reclaimed.
//!
//! Every tier agrees or refuses. The refusal-boundary tests pin that a returned fn
//! parameter, a heap-capturing returned closure, and a stored (aliased) closure each
//! skip cleanly (`L0339`) while the interpreters still compute the answer — correct-or-
//! refuse, never a miscompile.

use super::{ScratchDir, lullaby, stderr, stdout};

/// Compile `source` to a native `.exe` in `dir` and return its real (32-bit) exit
/// code. Asserts the program COMPILED natively — an emitted exe file must exist, so a
/// regression that makes a supported returned-closure shape silently SKIP is a
/// failure, not a vacuous pass. `run.status.code()` reads the true exit code (never
/// the shell's 8-bit-masked view).
fn native_exit(dir: &ScratchDir, tag: &str, source: &str) -> i32 {
    let src = dir.join(format!("{tag}.lby"));
    let exe = dir.join(format!("{tag}.exe"));
    std::fs::write(&src, source).expect("write source");
    let _ = std::fs::remove_file(&exe);
    let emit = lullaby()
        .args([
            "native",
            "-o",
            exe.to_str().expect("exe path"),
            src.to_str().expect("src path"),
        ])
        .output()
        .expect("run native");
    assert!(
        emit.status.success(),
        "native emit failed for {tag}:\n{source}\n{}",
        stderr(&emit)
    );
    assert!(
        exe.is_file(),
        "no native exe produced for {tag} — a supported returned-closure shape must \
         COMPILE, not skip:\n{source}\n{}{}",
        stdout(&emit),
        stderr(&emit)
    );
    let run = std::process::Command::new(&exe)
        .output()
        .expect("run native exe");
    run.status.code().expect("native exit code")
}

/// Run `source` on the three interpreter tiers, asserting each succeeds and prints the
/// same integer; returns that integer (the ground truth the native tier must match).
fn interp_value(dir: &ScratchDir, tag: &str, source: &str) -> i64 {
    let src = dir.join(format!("{tag}.lby"));
    std::fs::write(&src, source).expect("write source");
    let mut value: Option<i64> = None;
    for backend in [None, Some("ir"), Some("bytecode")] {
        let mut args = vec!["run".to_string()];
        if let Some(b) = backend {
            args.push("--backend".to_string());
            args.push(b.to_string());
        }
        args.push(src.to_str().expect("src path").to_string());
        let out = lullaby().args(&args).output().expect("run cli");
        assert!(
            out.status.success(),
            "interpreter {backend:?} failed for {tag}:\n{source}\n{}",
            stderr(&out)
        );
        let v: i64 = stdout(&out)
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("interp {backend:?} did not print an i64 for {tag}"));
        match value {
            Some(prev) => assert_eq!(
                prev, v,
                "interpreter {backend:?} disagrees with an earlier tier for {tag}"
            ),
            None => value = Some(v),
        }
    }
    value.expect("at least one interpreter tier")
}

/// Assert all four tiers agree that `source` returns `expected`: the three
/// interpreters via stdout, native via a real `.exe` exit code.
fn assert_four_tiers(tag: &str, source: &str, expected: i64) {
    let dir = ScratchDir::new(tag);
    let interp = interp_value(&dir, tag, source);
    assert_eq!(
        interp, expected,
        "interpreters must return {expected} for {tag}"
    );
    let native = native_exit(&dir, tag, source);
    assert_eq!(
        native as i64, expected,
        "native must agree with the interpreters ({expected}) for {tag}"
    );
}

/// Assert native SKIPS `source` cleanly (`L0339`, no exe run) while the interpreters
/// still compute `expected` — the correct-or-refuse boundary.
fn assert_native_skips(tag: &str, source: &str, expected: i64) {
    let dir = ScratchDir::new(tag);
    let interp = interp_value(&dir, tag, source);
    assert_eq!(
        interp, expected,
        "the skipped program must still run on the interpreters for {tag}"
    );
    let src = dir.join(format!("{tag}.lby"));
    let native = lullaby()
        .args(["native", "--verbose", src.to_str().expect("src path")])
        .output()
        .expect("run native");
    assert!(
        !native.status.success(),
        "native must refuse {tag} (an escaping/non-fresh returned closure)"
    );
    let rendered = format!("{}{}", stdout(&native), stderr(&native));
    assert!(
        rendered.contains("L0339"),
        "expected a clean L0339 skip for {tag}: {rendered}"
    );
}

// -- Supported shapes: four-tier parity on a real exe exit code ---------------

/// The canonical factory: `make_adder(5)` returns `fn x -> x + n`, invoked `g(3)` = 8.
/// The closure captures the factory's parameter `n`; the returned block pointer is
/// bound to `g` and called through the indirect ABI.
#[test]
fn native_returned_adder_factory() {
    let source = "\
fn make_adder n i64 -> fn(i64) -> i64
    fn x i64 -> x + n
fn main -> i64
    let g fn(i64) -> i64 = make_adder(5)
    g(3)
";
    assert_four_tiers("ret_adder", source, 8);
}

/// A NON-capturing returned closure: the factory takes no arguments and returns a
/// capture-free `fn x -> x * 2`. The block is a bare `[code_ptr]` word.
#[test]
fn native_returned_noncapturing_closure() {
    let source = "\
fn make_doubler -> fn(i64) -> i64
    fn x i64 -> x * 2
fn main -> i64
    let g fn(i64) -> i64 = make_doubler()
    g(21)
";
    assert_four_tiers("ret_noncap", source, 42);
}

/// A MULTI-capture returned closure: `make_affine(3, 4)` returns `fn x -> a*x + b`,
/// so the env block carries two captured words read in order when invoked.
#[test]
fn native_returned_multicapture_closure() {
    let source = "\
fn make_affine a i64 b i64 -> fn(i64) -> i64
    fn x i64 -> a * x + b
fn main -> i64
    let g fn(i64) -> i64 = make_affine(3, 4)
    g(10)
";
    assert_four_tiers("ret_multicap", source, 34);
}

/// Shape B — the factory binds the closure to a LOCAL literal and returns the local
/// (an implicit tail `f`). `closure_local_ok`'s return relaxation admits it.
#[test]
fn native_returned_local_literal_closure() {
    let source = "\
fn make_adder n i64 -> fn(i64) -> i64
    let f fn(i64) -> i64 = fn x i64 -> x + n
    f
fn main -> i64
    let g fn(i64) -> i64 = make_adder(100)
    g(23)
";
    assert_four_tiers("ret_local", source, 123);
}

/// FLOAT captures, parameters, and return through the positional-XMM ABI:
/// `make_lin(2.0, 3.0)` returns `fn x y -> a*x + b*y` (two float captures, two float
/// params, float return), sampled at three argument vectors and counted into an i64
/// so a wrong XMM register would change the exit code.
#[test]
fn native_returned_float_closure() {
    let source = "\
fn make_lin a f64 b f64 -> fn(f64, f64) -> f64
    fn x f64 y f64 -> a * x + b * y
fn main -> i64
    let g fn(f64, f64) -> f64 = make_lin(2.0, 3.0)
    let total i64 = 0
    if g(10.0, 5.0) > 30.0
        total = total + 1
    if g(1.0, 1.0) > 4.0
        total = total + 2
    if g(0.0, 10.0) > 25.0
        total = total + 4
    total
";
    // g(10,5)=35>30 ✓(+1); g(1,1)=5>4 ✓(+2); g(0,10)=30>25 ✓(+4) → 7.
    assert_four_tiers("ret_float", source, 7);
}

/// The returned closure INVOKED MULTIPLE TIMES from one stored local — each call
/// re-reads the same env block, so a call that clobbered the env pointer would drift.
#[test]
fn native_returned_closure_invoked_multiple_times() {
    let source = "\
fn make_adder n i64 -> fn(i64) -> i64
    fn x i64 -> x + n
fn main -> i64
    let g fn(i64) -> i64 = make_adder(1)
    let a i64 = g(10)
    let b i64 = g(20)
    let c i64 = g(30)
    a + b + c
";
    // (10+1) + (20+1) + (30+1) = 63.
    assert_four_tiers("ret_multi_invoke", source, 63);
}

/// A factory with MULTIPLE RETURN EDGES returning DIFFERENT closures: each edge is its
/// own fresh literal (a distinct synthesized body), and the caller invokes whichever
/// the factory produced.
#[test]
fn native_returned_multiple_return_edges() {
    let source = "\
fn pick c bool -> fn(i64) -> i64
    if c
        return fn x i64 -> x + 1
    fn x i64 -> x * 100
fn main -> i64
    let g fn(i64) -> i64 = pick(false)
    let h fn(i64) -> i64 = pick(true)
    g(5) + h(5)
";
    // pick(false) → x*100 → 500; pick(true) → x+1 → 6; 500 + 6 = 506.
    assert_four_tiers("ret_multi_edge", source, 506);
}

/// **Soundness pin — the factory stays off the arena, so the returned block is never
/// reclaimed.** The factory captures `222`; after obtaining the closure the caller
/// runs a heap-churning helper that would reuse a freed region if the factory had
/// rewound its heap, then invokes the closure. Reading back exactly `222` proves the
/// block survived — a dangling/reclaimed block would read garbage.
#[test]
fn native_returned_closure_survives_heap_churn() {
    let source = "\
fn make_const n i64 -> fn(i64) -> i64
    fn x i64 -> x + n
fn churn -> i64
    let total i64 = 0
    let i i64 = 0
    while i < 50
        let s string = \"reuse-the-heap-region-aggressively\"
        total = total + len(s)
        i = i + 1
    total
fn main -> i64
    let g fn(i64) -> i64 = make_const(222)
    let junk i64 = churn()
    g(junk - junk)
";
    assert_four_tiers("ret_no_dangle", source, 222);
}

// -- Refusal boundary: native skips cleanly, interpreters still answer --------

/// A returned fn PARAMETER aliases a caller's env — never this increment's fresh-block
/// case — so `returns_only_local_closure_literals` refuses `identity_fn` while the
/// interpreters run it. This is the alias hazard the admit-fn-return gate must reject.
/// `base` is a direct literal (not a factory result), so the ONLY reason native
/// declines is the returned parameter — isolating that guard, whose teeth are proven by
/// injection in the closure fuzzer / this module's history.
#[test]
fn native_returned_fn_parameter_skips() {
    let source = "\
fn identity_fn f fn(i64) -> i64 -> fn(i64) -> i64
    return f
fn main -> i64
    let base fn(i64) -> i64 = fn x i64 -> x + 3
    let g fn(i64) -> i64 = identity_fn(base)
    g(7)
";
    assert_native_skips("ret_param_skip", source, 10);
}

/// A HEAP-capturing returned closure (`fn s -> p + s`, capturing a `string`) is
/// outside the scalar-only native closure subset, so native skips.
#[test]
fn native_returned_heap_capture_skips() {
    let source = "\
fn make_prefixer p string -> fn(string) -> string
    fn s string -> p + s
fn main -> i64
    let g fn(string) -> string = make_prefixer(\"hi-\")
    len(g(\"there\"))
";
    // "hi-" + "there" = "hi-there" → len 8.
    assert_native_skips("ret_heap_skip", source, 8);
}

/// A closure STORED (aliased) into another local rather than returned/called is an
/// escaping value read, so native skips — a different escape from a returned closure
/// and not unblocked by this increment.
#[test]
fn native_stored_closure_skips() {
    let source = "\
fn main -> i64
    let c fn(i64) -> i64 = fn x i64 -> x + 1
    let d fn(i64) -> i64 = c
    d(41)
";
    assert_native_skips("ret_stored_skip", source, 42);
}

// -- Arena stage-4b: MARK-ADVANCE PROMOTION of a returned closure --------------
//
// The factory is now ARENA-eligible (criterion 1b admits a promotable closure
// factory): its return-edge reset PROMOTES the survivor — relocates the returned
// `[code_ptr][captures…]` block DOWN to the region mark and advances `heap_next`
// past it (`heap_next = markF + size`, NOT `markF`) — so the factory reclaims its
// per-call scratch while the survivor lands in the caller's region and stays live
// until the caller's own rewind. These pins are four-tier (interpreters via stdout,
// native via a real `.exe` exit code); the teeth (a plain reset, a wrong size, an
// admitted non-fresh survivor) are proven by injection in this suite's history and
// the closure fuzzer.

/// Four-tier agreement WITHOUT a hardcoded expected: the interpreters establish the
/// ground truth and native must reproduce it. Used for loop-sum shapes whose exact
/// value is tedious to hand-compute but must be identical across tiers.
fn assert_native_matches_interp(tag: &str, source: &str) -> i64 {
    let dir = ScratchDir::new(tag);
    let interp = interp_value(&dir, tag, source);
    let native = native_exit(&dir, tag, source);
    assert_eq!(
        native as i64, interp,
        "native must agree with the interpreters ({interp}) for {tag}"
    );
    interp
}

/// **Bounded-heap proof that promotion + reclamation FIRES.** A hot loop calls a
/// factory 20 000 times; the factory allocates a ~130-byte scratch string per call and
/// returns a closure capturing an `i64` derived from it. WITH the promoting reset the
/// factory reclaims that scratch each call (only the 16-byte survivor accumulates:
/// 20 000 × 16 B ≈ 320 KB, under the 1 MB native heap), so native runs to completion
/// and agrees with the interpreters. WITHOUT reclamation the scratch would leak
/// (20 000 × ~150 B ≈ 3 MB) and the native allocator's exhaustion guard would `ud2`-trap
/// — a divergent (crashing) exit. Native matching the interpreters is therefore only
/// possible because the promoting reset reclaims the per-call scratch.
#[test]
fn native_promoting_factory_reclaims_scratch_bounded_heap() {
    let source = "\
fn make_val n i64 -> fn(i64) -> i64
    let pad string = \"PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING-PADDING\"
    let m i64 = n + len(pad)
    fn x i64 -> x + m
fn hot -> i64
    let acc i64 = 0
    let i i64 = 0
    while i < 20000
        let g fn(i64) -> i64 = make_val(i)
        acc = acc + g(0)
        i = i + 1
    acc
fn main -> i64
    hot()
";
    assert_native_matches_interp("promote_bounded", source);
}

/// The promoted survivor stays live in the caller's region until the caller's own
/// rewind — even after the caller does MORE heap allocation. The factory is now
/// arena-eligible (promoting); after obtaining the closure `main` runs a heap-churning
/// helper (`churn`, itself an arena function whose confined loop reclaims per
/// iteration) and only THEN invokes the closure. Reading back exactly `222` proves the
/// survivor was promoted into `main`'s region and never reclaimed by the factory's or
/// churn's rewinds.
#[test]
fn native_promoting_survivor_survives_more_allocation() {
    let source = "\
fn make_const n i64 -> fn(i64) -> i64
    let pad string = \"factory-scratch-reclaimed-by-the-promoting-reset\"
    let m i64 = n + len(pad) - len(pad)
    fn x i64 -> x + m
fn churn -> i64
    let total i64 = 0
    let i i64 = 0
    while i < 50
        let s string = \"reuse-the-heap-region-aggressively\"
        total = total + len(s)
        i = i + 1
    total
fn main -> i64
    let g fn(i64) -> i64 = make_const(222)
    let junk i64 = churn()
    g(junk - junk)
";
    assert_four_tiers("promote_survive_alloc", source, 222);
}

/// MULTIPLE return edges returning DIFFERENT-arity closures, so the promoting reset
/// sizes the survivor PER RETURN SITE: the `return` edge yields a 1-capture (2-word)
/// closure and the tail edge a 2-capture (3-word) closure. A single function-wide size
/// would mis-relocate one of them.
#[test]
fn native_promoting_multiple_arity_edges() {
    let source = "\
fn pick c bool a i64 b i64 -> fn(i64) -> i64
    if c
        return fn x i64 -> x + a
    fn x i64 -> x + a + b
fn main -> i64
    let g fn(i64) -> i64 = pick(false, 10, 20)
    let h fn(i64) -> i64 = pick(true, 10, 20)
    g(1) + h(1)
";
    // g = x+10+20 → g(1)=31; h = x+10 → h(1)=11; 31 + 11 = 42.
    assert_four_tiers("promote_multi_arity", source, 42);
}

/// FLOAT captures survive the relocation (the promoting reset copies raw words, so an
/// `f64` capture's bits are preserved), while the factory's per-call scratch is
/// reclaimed. A hot loop of 8 000 iterations over a float factory with scratch stays
/// heap-bounded (proving reclamation) AND computes the same float-threshold count on
/// every tier (proving the relocated float captures are read from the right words).
#[test]
fn native_promoting_float_captures_relocated() {
    let source = "\
fn make_lin a f64 b f64 -> fn(f64, f64) -> f64
    let pad string = \"float-factory-scratch-string-reclaimed-by-the-promoting-reset-each-call\"
    let n i64 = len(pad)
    fn x f64 y f64 -> a * x + b * y
fn run -> i64
    let total i64 = 0
    let i i64 = 0
    while i < 8000
        let g fn(f64, f64) -> f64 = make_lin(2.0, 3.0)
        if g(10.0, 5.0) > 30.0
            total = total + 1
        i = i + 1
    total
fn main -> i64
    run()
";
    // g(10,5) = 2*10 + 3*5 = 35 > 30 every iteration → total = 8000.
    assert_four_tiers("promote_float", source, 8000);
}

// -- Per-iteration closure-block reclamation: the LOOP-BOUNDEDNESS pins ---------
//
// Each fixture below is sized so it CANNOT pass without the reclamation it names:
// the native heap region is 1 MiB and a scalar-capture closure block costs
// 16 (RC header) + 8 (code ptr) + 8 (capture) = 32 bytes, so ≥ 32 768 unreclaimed
// blocks exhaust it and the allocator's guard traps (`0xC000001D`). Every fixture
// runs ≥ 100 000 iterations — a ~3.2 MB leak, > 3× the region — so a regression in
// either reclaim path turns the test from "native == interpreters" into a crash,
// never into a silently-passing weaker claim.
//
// This sizing is the lesson of `native_promoting_factory_reclaims_scratch_bounded_heap`
// above: at 20 000 iterations its 16-byte survivors total only ~320 KB, so it passed
// with AND without the loop reclamation it appeared to prove. Do not size a
// bounded-heap fixture below the region.

/// **Arena-DENIED creator, closure literal per iteration.** `main` calls the recursive
/// `fib`, so the retention summary pre-poisons that callee and `main` is refused the
/// arena (criterion 3) — no function region, no per-iteration sub-region. The only
/// thing that can reclaim the closure block allocated on each of the 100 000
/// iterations is the loop-body RC drop (`__lullaby_rc_dec` on the fallthrough and
/// early-exit edges). Without it the 3.2 MB of blocks exhaust the 1 MiB region and the
/// exe traps instead of agreeing with the interpreters.
#[test]
fn native_loop_closure_dropped_in_arena_denied_function() {
    let source = "\
fn fib k i64 -> i64
    if k < 2
        return k
    fib(k - 1) + fib(k - 2)
fn main -> i64
    let n i64 = 2
    let total i64 = 0
    for i from 0 to 100000
        let add_n fn(i64) -> i64 = fn x i64 -> x + n
        total = total + add_n(1)
    total % 7 + fib(5)
";
    assert_native_matches_interp("loop_closure_drop_denied", source);
}

/// The same leak through the shipped stage-3a **higher-order sink**: the closure is
/// passed to `apply(add_n, 1)` rather than called directly. Passing it to a HOF makes
/// `apply` an indirect-call (retaining) callee, which independently denies `main` the
/// arena — so this is a DOCUMENTED-SUPPORTED shape that reaches the same RC path. The
/// drop's use predicate must therefore accept a higher-order-sink argument, not only a
/// direct-call callee.
#[test]
fn native_loop_closure_dropped_through_hof_sink() {
    let source = "\
fn apply f fn(i64) -> i64 v i64 -> i64
    f(v)
fn main -> i64
    let n i64 = 2
    let total i64 = 0
    for i from 0 to 100000
        let add_n fn(i64) -> i64 = fn x i64 -> x + n
        total = total + apply(add_n, 1)
    total % 7
";
    assert_native_matches_interp("loop_closure_drop_hof", source);
}

/// **Arena-ELIGIBLE creator — the no-double-free control.** Identical loop shape, but
/// `main`'s only callee is the literal-bound closure itself, so `main` IS arena and its
/// loop gets a per-iteration sub-region. The RC drop is deliberately NOT emitted here
/// (see `collect_loop_body_drops`): the sub-region rewind reclaims the block, and
/// `__lullaby_rc_free` is a no-op in arena mode anyway. Agreement with the
/// interpreters at 100 000 iterations pins that the arena path still reclaims and that
/// adding the RC drop to the denied path did not introduce a second free here.
#[test]
fn native_loop_closure_arena_eligible_still_bounded() {
    let source = "\
fn main -> i64
    let n i64 = 2
    let total i64 = 0
    for i from 0 to 100000
        let add_n fn(i64) -> i64 = fn x i64 -> x + n
        total = total + add_n(1)
    total % 7
";
    assert_native_matches_interp("loop_closure_arena_ok", source);
}

/// **A call that RETURNS a closure is a heap touch.** `main`'s loop binds the result of
/// a promoting factory 200 000 times. The factory promotes a 16-byte survivor into the
/// caller's region on every call, so 3.2 MB accumulates unless `main` itself is an
/// arena region whose loop sub-region rewinds at each iteration edge. Two things must
/// hold for that: `expr_touches_heap` must see a `fn`-typed CALL as heap-touching
/// (else `main` fails arena criterion 2), and the retention summary must not treat
/// `g`, bound by a promotable-factory call, as an unknown indirect target (else `main`
/// fails criterion 3). Removing either makes this trap.
#[test]
fn native_factory_call_loop_is_heap_bounded() {
    let source = "\
fn make a i64 -> fn(i64) -> i64
    fn x i64 -> x + a
fn main -> i64
    let total i64 = 0
    for i from 0 to 200000
        let g fn(i64) -> i64 = make(i)
        total = total + g(1)
    total % 7
";
    assert_native_matches_interp("factory_loop_bounded", source);
}
