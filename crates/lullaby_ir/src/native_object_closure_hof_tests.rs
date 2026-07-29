//! Unit tests for the **higher-order sink index** (`build_hof_index` in
//! `native_object_closure_escape.rs`) — the module-wide sweep that decides which `fn(...)`-typed
//! parameters are non-escaping sinks and therefore which closure argument positions the
//! native backend may lower.
//!
//! These assert the CLASSIFICATION directly, one rule per test, because the end-to-end
//! fixtures cannot always separate the layers: a function returning its `fn` parameter,
//! for instance, is refused by BOTH `native_fn_return_eligibility` and the sink rule, so
//! an `L0339` alone does not prove the sink rule saw it. Asserting the index shows each
//! rule independently.
//!
//! The end-to-end proofs — a 2- and 3-level chain compiling to a real `.exe` whose exit
//! matches all three interpreters, the escape probes skipping with `L0339`, and the
//! bounded 100 001-iteration chain loop — live in `crates/lullaby_cli/tests/cli/suite3.rs`,
//! which can build and run the binary.

use super::*;
use crate::{lower, lower_to_bytecode};
use lullaby_lexer::lex;
use lullaby_parser::parse;
use lullaby_semantics::validate_executable;
use std::collections::HashMap;

fn module_for(source: &str) -> BytecodeModule {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    let checked = validate_executable(&program).expect("semantic");
    let ir = lower(&checked).expect("lower");
    lower_to_bytecode(&ir)
}

/// Whether `(function, param_index)` is a proven sink in the module built from `source`.
fn is_sink(source: &str, function: &str, param_index: usize) -> bool {
    let module = module_for(source);
    let index = build_hof_index(&module);
    hof_params_of(&index, function)
        .iter()
        .any(|p| p.index == param_index)
}

/// The sink parameter NAMES the sweep proves for `function`, sorted for a stable assert.
fn sink_names(source: &str, function: &str) -> Vec<String> {
    let module = module_for(source);
    let index = build_hof_index(&module);
    let mut names: Vec<String> = hof_params_of(&index, function)
        .iter()
        .map(|p| p.name.clone())
        .collect();
    names.sort();
    names
}

// -- Rule (a): a call-only parameter is a sink (stage 3a, unchanged) ----------

#[test]
fn call_only_fn_parameter_is_a_sink() {
    let source = concat!(
        "fn apply f fn(i64) -> i64 v i64 -> i64\n    f(v)\n\n",
        "fn main -> i64\n    let sq fn(i64) -> i64 = fn x i64 -> x * x\n    apply(sq, 6)\n",
    );
    assert!(
        is_sink(source, "apply", 0),
        "a call-only fn param is a sink"
    );
    assert_eq!(sink_names(source, "apply"), vec!["f".to_string()]);
}

#[test]
fn a_parameter_called_several_times_is_still_a_sink() {
    let source = concat!(
        "fn twice f fn(i64) -> i64 v i64 -> i64\n    f(v) + f(v + 1)\n\n",
        "fn main -> i64\n    let c fn(i64) -> i64 = fn x i64 -> x + 1\n    twice(c, 1)\n",
    );
    assert!(is_sink(source, "twice", 0));
}

// -- Rule (b): a bare pass ONWARD to another sink position (stage 3c) ---------

#[test]
fn two_level_pass_onward_makes_both_levels_sinks() {
    let source = concat!(
        "fn inner f fn(i64) -> i64 v i64 -> i64\n    f(v)\n\n",
        "fn outer f fn(i64) -> i64 v i64 -> i64\n    inner(f, v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 2\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    outer(c, 40)\n",
    );
    assert!(is_sink(source, "inner", 0), "the terminal sink");
    assert!(
        is_sink(source, "outer", 0),
        "a bare pass onward to a sink position is itself a sink (stage 3c)"
    );
}

#[test]
fn three_level_pass_onward_chain_is_admitted_end_to_end() {
    let source = concat!(
        "fn level3 f fn(i64) -> i64 v i64 -> i64\n    f(v) + f(v + 1)\n\n",
        "fn level2 f fn(i64) -> i64 v i64 -> i64\n    level3(f, v)\n\n",
        "fn level1 f fn(i64) -> i64 v i64 -> i64\n    level2(f, v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 5\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x * n\n",
        "    level1(c, 1)\n",
    );
    for level in ["level1", "level2", "level3"] {
        assert!(is_sink(source, level, 0), "{level} must be a sink");
    }
}

#[test]
fn onward_pass_at_a_non_zero_argument_position_is_matched_by_index() {
    // `relay` hands its parameter to `inner`'s SECOND parameter. The sweep must match
    // the callee position by index, not merely by callee name.
    let source = concat!(
        "fn inner v i64 f fn(i64) -> i64 -> i64\n    f(v)\n\n",
        "fn relay f fn(i64) -> i64 v i64 -> i64\n    inner(v, f)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 3\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    relay(c, 4)\n",
    );
    assert!(is_sink(source, "inner", 1), "inner's param 1 is call-only");
    assert!(!is_sink(source, "inner", 0), "param 0 is not fn-typed");
    assert!(is_sink(source, "relay", 0));
}

// -- Default-deny: every non-sink use ----------------------------------------

#[test]
fn a_stored_fn_parameter_is_not_a_sink() {
    let source = concat!(
        "fn leaky f fn(i64) -> i64 v i64 -> i64\n",
        "    let saved fn(i64) -> i64 = f\n    saved(v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 7\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    leaky(c, 5)\n",
    );
    assert!(!is_sink(source, "leaky", 0), "a stored fn param escapes");
}

#[test]
fn a_returned_fn_parameter_is_not_a_sink() {
    // A returned fn parameter aliases a CALLER's env block, so it is never a sanctioned
    // escape. `native_fn_return_eligibility` independently refuses this function's
    // `fn(...)` return, which is why the classification is asserted here rather than
    // being inferred from the end-to-end `L0339`.
    let source = concat!(
        "fn giveback f fn(i64) -> i64 -> fn(i64) -> i64\n    f\n\n",
        "fn main -> i64\n",
        "    let n i64 = 4\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    let g fn(i64) -> i64 = giveback(c)\n",
        "    g(6)\n",
    );
    assert!(
        !is_sink(source, "giveback", 0),
        "a returned fn param aliases the caller's env and is not a sink"
    );
}

#[test]
fn a_bare_read_of_a_fn_parameter_is_not_a_sink() {
    let source = concat!(
        "fn touch f fn(i64) -> i64 v i64 -> i64\n    f\n    f(v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 8\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    touch(c, 20)\n",
    );
    assert!(
        !is_sink(source, "touch", 0),
        "a bare value read of a fn param is an escape"
    );
}

#[test]
fn passing_onward_to_a_non_sink_position_is_not_a_sink() {
    // `mid` stores its parameter, so `mid`'s position 0 is not a sink — and therefore
    // `outer`, which passes onward INTO that position, is not one either. This is the
    // recursive half of the rule: the denial has to propagate BACK up the chain.
    let source = concat!(
        "fn mid f fn(i64) -> i64 v i64 -> i64\n",
        "    let saved fn(i64) -> i64 = f\n    saved(v)\n\n",
        "fn outer f fn(i64) -> i64 v i64 -> i64\n    mid(f, v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 6\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    outer(c, 30)\n",
    );
    assert!(!is_sink(source, "mid", 0), "mid stores its parameter");
    assert!(
        !is_sink(source, "outer", 0),
        "passing onward to a NON-sink position must deny the passer too"
    );
}

#[test]
fn a_non_native_scalar_fn_signature_is_never_a_candidate() {
    // A `fn(string) -> i64` parameter has no native call signature, so it cannot be a
    // sink even though its only use is a direct call.
    let source = concat!(
        "fn apply f fn(string) -> i64 s string -> i64\n    f(s)\n\n",
        "fn main -> i64\n",
        "    let c fn(string) -> i64 = fn t string -> len(t)\n",
        "    apply(c, \"abcd\")\n",
    );
    assert!(
        !is_sink(source, "apply", 0),
        "a non-native-scalar fn signature is not even a candidate"
    );
}

// -- SCC pre-poisoning: the teeth of the sweep -------------------------------

#[test]
fn a_mutually_recursive_sink_pair_is_poisoned() {
    // Without the on-stack back-edge poison, `ping` and `pong` would admit each other
    // (each is "call-only or passed onward to the other"), and the sweep would accept an
    // UNBOUNDED chain. Default-deny requires both to be refused.
    let source = concat!(
        "fn ping f fn(i64) -> i64 v i64 -> i64\n",
        "    if v <= 0\n        return f(0)\n    pong(f, v - 1)\n\n",
        "fn pong f fn(i64) -> i64 v i64 -> i64\n    ping(f, v - 1)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 9\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    ping(c, 4)\n",
    );
    assert!(!is_sink(source, "ping", 0), "ping sits on a cycle");
    assert!(!is_sink(source, "pong", 0), "pong sits on a cycle");
}

#[test]
fn a_self_recursive_pass_onward_is_poisoned() {
    let source = concat!(
        "fn spin f fn(i64) -> i64 v i64 -> i64\n",
        "    if v <= 0\n        return f(0)\n    spin(f, v - 1)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 2\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    spin(c, 3)\n",
    );
    assert!(
        !is_sink(source, "spin", 0),
        "a self-recursive pass onward is a cycle and must be poisoned"
    );
}

#[test]
fn a_cycle_poisons_the_callers_that_reach_it_but_not_an_independent_sink() {
    // `head` passes onward into the poisoned `ping`/`pong` cycle, so it inherits the
    // poison; `solo` is an independent call-only sink in the same module and must stay
    // admitted — the poison propagates along edges, it is not module-wide.
    let source = concat!(
        "fn ping f fn(i64) -> i64 v i64 -> i64\n",
        "    if v <= 0\n        return f(0)\n    pong(f, v - 1)\n\n",
        "fn pong f fn(i64) -> i64 v i64 -> i64\n    ping(f, v - 1)\n\n",
        "fn head f fn(i64) -> i64 v i64 -> i64\n    ping(f, v)\n\n",
        "fn solo f fn(i64) -> i64 v i64 -> i64\n    f(v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 1\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    head(c, 2) + solo(c, 3)\n",
    );
    assert!(!is_sink(source, "head", 0), "head reaches the cycle");
    assert!(is_sink(source, "solo", 0), "an independent sink survives");
}

// -- Unknown / indirect callees deny -----------------------------------------

#[test]
fn passing_a_fn_parameter_to_an_indirect_callee_is_not_a_sink() {
    // `g(f)` calls THROUGH another fn parameter. The callee is not a module function, so
    // the position cannot be a candidate sink and the pass denies (the default inversion
    // `classify_call` applies for extern/indirect targets).
    let source = concat!(
        "fn relay f fn(i64) -> i64 g fn(fn(i64) -> i64) -> i64 -> i64\n    g(f)\n\n",
        "fn main -> i64\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + 1\n",
        "    let h fn(fn(i64) -> i64) -> i64 = fn k fn(i64) -> i64 -> k(1)\n",
        "    relay(c, h)\n",
    );
    assert!(
        !is_sink(source, "relay", 0),
        "a pass through an indirect callee denies"
    );
}

#[test]
fn a_module_with_no_fn_parameters_produces_an_empty_index() {
    let module = module_for("fn main -> i64\n    let a i64 = 1\n    a + 1\n");
    assert!(
        build_hof_index(&module).is_empty(),
        "a module with no fn-typed parameter must produce no index entries, keeping \
         existing codegen byte-identical"
    );
}

// -- The load-bearing soundness link -----------------------------------------

/// The `[code_ptr][captures…]` block is allocated in the CREATOR's frame region and every
/// use in the chain is dynamically nested inside the creating call — but that only keeps
/// the block alive if no arena rewind can fall between its creation and its last use. The
/// link that guarantees it: `fn_typed_binding_names` puts every `fn`-typed PARAMETER into
/// the indirect-denied set, so a chain function that calls through its parameter is
/// R4-retaining, the poison propagates back up the chain through R4, and criterion 3
/// (`all_callees_non_retaining`) therefore fails for the creator — which gets neither a
/// function region nor a per-iteration loop sub-region.
///
/// This asserts that link DIRECTLY, at both chain depths and for the loop shape, because
/// it is the whole soundness argument for stage 3c and it is not visible in the emitted
/// bytes. If a future change ever made a chain function non-retaining, the creator would
/// gain a region and this test would fail before a rewind could reclaim a live closure.
#[test]
fn a_chain_creator_is_kept_off_the_arena() {
    let chain2 = concat!(
        "fn inner f fn(i64) -> i64 v i64 -> i64\n    f(v)\n\n",
        "fn outer f fn(i64) -> i64 v i64 -> i64\n    inner(f, v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 2\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    outer(c, 40)\n",
    );
    let chain3 = concat!(
        "fn level3 f fn(i64) -> i64 v i64 -> i64\n    f(v) + f(v + 1)\n\n",
        "fn level2 f fn(i64) -> i64 v i64 -> i64\n    level3(f, v)\n\n",
        "fn level1 f fn(i64) -> i64 v i64 -> i64\n    level2(f, v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 5\n",
        "    let c fn(i64) -> i64 = fn x i64 -> x * n\n",
        "    level1(c, 1)\n",
    );
    let chain_loop = concat!(
        "fn inner f fn(i64) -> i64 v i64 -> i64\n    f(v)\n\n",
        "fn outer f fn(i64) -> i64 v i64 -> i64\n    inner(f, v)\n\n",
        "fn main -> i64\n",
        "    let n i64 = 3\n",
        "    let total i64 = 0\n",
        "    for i from 0 to 100\n",
        "        let f fn(i64) -> i64 = fn x i64 -> x + n\n",
        "        total = (total + outer(f, 1)) % 251\n",
        "    total\n",
    );
    for (label, source) in [
        ("chain2", chain2),
        ("chain3", chain3),
        ("chain_loop", chain_loop),
    ] {
        let module = module_for(source);
        let heap_aggs = heap_carrying_aggregates(&module.structs, &module.enums);
        let closure_layouts = compute_module_closure_layouts(&module);
        let summary = retaining_summary(&module, &heap_aggs, &closure_layouts);
        // Every chain function is retaining: the terminal one calls THROUGH its `fn`
        // parameter (an indirect target, denied by R4), and each relay inherits that.
        for relay in ["inner", "outer", "level1", "level2", "level3"] {
            if let Some(&retaining) = summary.get(relay) {
                assert!(
                    retaining,
                    "{label}: chain function `{relay}` must be retaining so the creator \
                     is denied the arena"
                );
            }
        }
        let program = emit_native_program(&module).expect("emit native program");
        let eligible = program.compiled.clone();
        assert!(
            eligible.contains(&"main".to_string()),
            "{label}: the chain must actually compile natively, or this proves nothing"
        );
        let signatures = native_signatures_for(&module, &eligible);
        let arena = arena_eligible_functions(&module, &eligible, &signatures, &closure_layouts);
        assert!(
            !arena.contains("main"),
            "{label}: the closure-creating function must stay OFF the arena — with a \
             region or loop sub-region a rewind could fall between block creation and \
             the chain's last use"
        );
    }
}

/// The native signatures of `names`, built exactly as the program driver builds them, so
/// an `arena_eligible_functions` query in a test sees the same inputs the pipeline does.
fn native_signatures_for(
    module: &BytecodeModule,
    names: &[String],
) -> HashMap<String, NativeSignature> {
    let mut signatures = HashMap::new();
    for name in names {
        let Some(function) = module.functions.iter().find(|f| &f.name == name) else {
            continue;
        };
        let lengths = infer_array_lengths(function, module, names).expect("infer array lengths");
        let sig = compute_native_signature(function, &module.structs, &module.enums, &lengths)
            .expect("compute native signature");
        signatures.insert(name.clone(), sig);
    }
    signatures
}
