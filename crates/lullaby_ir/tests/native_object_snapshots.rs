use std::fs;
use std::path::PathBuf;

use lullaby_ir::native_object::{emit_alpha1_coff_object, snapshot_native_object};
use lullaby_ir::{lower, lower_to_bytecode};
use lullaby_lexer::lex;
use lullaby_parser::parse;
use lullaby_semantics::validate_executable;

const UPDATE_ENV: &str = "LULLABY_UPDATE_NATIVE_OBJECT_SNAPSHOTS";
const SNAPSHOT: &str = "tests/snapshots/alpha1_return_42.coff.json";
const SOURCE: &str = "fn main -> i64\n    return 42\n";

#[test]
fn alpha1_literal_return_coff_object_matches_checked_in_snapshot() {
    let snapshot_path = ir_crate_root().join(SNAPSHOT);
    let actual = snapshot_for(SOURCE);

    if std::env::var_os(UPDATE_ENV).is_some() {
        if let Some(parent) = snapshot_path.parent() {
            fs::create_dir_all(parent).expect("create native object snapshot directory");
        }
        fs::write(&snapshot_path, &actual).expect("write native object snapshot");
        return;
    }

    let expected = fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", snapshot_path.display()));
    assert_eq!(
        expected, actual,
        "native object snapshot changed.\nReview the object-emission change, then refresh the checked-in golden file with PowerShell: `$env:LULLABY_UPDATE_NATIVE_OBJECT_SNAPSHOTS='1'; cargo test -p lullaby_ir --test native_object_snapshots; Remove-Item Env:LULLABY_UPDATE_NATIVE_OBJECT_SNAPSHOTS`."
    );
}

fn snapshot_for(source: &str) -> String {
    let tokens = lex(source).expect("lex native object snapshot source");
    let program = parse(&tokens).expect("parse native object snapshot source");
    let checked = validate_executable(&program).expect("validate native object snapshot source");
    let ir = lower(&checked).expect("lower native object snapshot source");
    let bytecode = lower_to_bytecode(&ir);
    let object = emit_alpha1_coff_object(&bytecode).expect("emit native object snapshot");
    let mut json =
        serde_json::to_string_pretty(&snapshot_native_object(&object)).expect("serialize snapshot");
    json.push('\n');
    json
}

fn ir_crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
