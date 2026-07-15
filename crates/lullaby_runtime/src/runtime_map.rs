//! The insertion-ordered `map<K, V>` runtime value and its argument extractor.
//! Split out of `lib.rs` as a behavior-preserving code move; `Value` and
//! `RuntimeError` (in `lib.rs`) are reached through `crate::` paths.

use std::collections::HashMap;
use std::fmt;

use crate::{RuntimeError, Value};

/// A hashable projection of a `map<K, V>` key. The type system restricts map
/// keys to `i64` or `string` (both hashable) — see `map_key_ok` in
/// `lullaby_semantics` — so this enum captures exactly those two kinds and lets
/// [`OrderedMap`] index entries by key for O(1) lookup. Any other value kind
/// (never produced for a well-typed program) has no projection; the map falls
/// back to a linear scan so `Value` equality semantics are preserved exactly.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MapKey {
    I64(i64),
    Str(String),
}

impl MapKey {
    fn from_value(value: &Value) -> Option<MapKey> {
        match value {
            Value::I64(n) => Some(MapKey::I64(*n)),
            Value::String(s) => Some(MapKey::Str(s.to_string())),
            _ => None,
        }
    }
}

/// An insertion-ordered `map<K, V>` value.
///
/// `entries` is the single source of truth for iteration order and value
/// equality — it is byte-for-byte the old `Vec<(Value, Value)>` representation,
/// so `map_keys`/`map_values` iterate in insertion order and `==` compares the
/// entries element-wise in order, unchanged. `index` maps each hashable key to
/// its position in `entries`, turning `map_get`/`map_has`/`map_set`-of-an-
/// existing-key from an O(n) linear scan into an O(1) hash probe.
///
/// Only `entries` is observable, so `PartialEq` compares just that (`index` is a
/// derived acceleration structure), and `Clone`/`Debug` are manual for the same
/// reason — `Debug` renders as the entries vector so `Value`'s derived `Debug`
/// output is identical to the previous `Map([..])` form.
pub struct OrderedMap {
    entries: Vec<(Value, Value)>,
    index: HashMap<MapKey, usize>,
}

impl OrderedMap {
    /// A fresh empty map.
    pub fn new() -> Self {
        OrderedMap {
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the map has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The value mapped to `key`, if present. O(1) for the guaranteed
    /// `i64`/`string` keys, with a linear-scan fallback for any other kind.
    pub fn get(&self, key: &Value) -> Option<&Value> {
        match MapKey::from_value(key) {
            Some(mk) => self.index.get(&mk).map(|&i| &self.entries[i].1),
            None => self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        }
    }

    /// Whether `key` is present. O(1) for `i64`/`string` keys.
    pub fn contains_key(&self, key: &Value) -> bool {
        match MapKey::from_value(key) {
            Some(mk) => self.index.contains_key(&mk),
            None => self.entries.iter().any(|(k, _)| k == key),
        }
    }

    /// Insert or overwrite `key -> value`, preserving the position of an
    /// existing key. O(1) for `i64`/`string` keys.
    pub fn insert(&mut self, key: Value, value: Value) {
        match MapKey::from_value(&key) {
            Some(mk) => match self.index.get(&mk) {
                Some(&i) => self.entries[i].1 = value,
                None => {
                    let position = self.entries.len();
                    self.entries.push((key, value));
                    self.index.insert(mk, position);
                }
            },
            None => match self.entries.iter_mut().find(|(k, _)| *k == key) {
                Some(entry) => entry.1 = value,
                None => self.entries.push((key, value)),
            },
        }
    }

    /// Remove `key` if present, preserving the order of the remaining entries.
    /// O(n) (rare operation): the vector shift plus an index rebuild.
    pub fn remove(&mut self, key: &Value) {
        let position = match MapKey::from_value(key) {
            Some(mk) => self.index.get(&mk).copied(),
            None => self.entries.iter().position(|(k, _)| k == key),
        };
        if let Some(i) = position {
            self.entries.remove(i);
            self.reindex();
        }
    }

    /// Rebuild `index` from `entries` after positions shift.
    fn reindex(&mut self) {
        self.index.clear();
        self.index.reserve(self.entries.len());
        for (i, (key, _)) in self.entries.iter().enumerate() {
            if let Some(mk) = MapKey::from_value(key) {
                self.index.insert(mk, i);
            }
        }
    }

    /// Iterate the entries in insertion order.
    pub fn iter(&self) -> std::slice::Iter<'_, (Value, Value)> {
        self.entries.iter()
    }

    /// Consume the map, yielding the entries in insertion order.
    pub fn into_entries(self) -> Vec<(Value, Value)> {
        self.entries
    }
}

impl Default for OrderedMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for OrderedMap {
    fn clone(&self) -> Self {
        OrderedMap {
            entries: self.entries.clone(),
            index: self.index.clone(),
        }
    }
}

// Only the entries are observable; the index is a derived acceleration
// structure, so equality compares entries alone. This is byte-for-byte the old
// `Vec<(Value, Value)>` element-wise, in-order comparison.
impl PartialEq for OrderedMap {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

// Render as the entries vector so `Value`'s derived `Debug` output stays
// identical to the previous `Map([(k, v), ..])` representation.
impl fmt::Debug for OrderedMap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.entries.fmt(formatter)
    }
}

/// Unwrap a runtime `Value` expected to be a map, reporting `L0417` otherwise.
pub fn expect_map(name: &str, value: Value) -> Result<OrderedMap, RuntimeError> {
    match value {
        Value::Map(entries) => Ok(*entries),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a map but got `{other}`"),
        )),
    }
}
