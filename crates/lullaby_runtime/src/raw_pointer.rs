//! Freestanding-tier raw-pointer *addressing* model for the interpreters — the
//! shared byte-addressed backing that makes `addr_of` / `ptr_offset` / `ptr_cast`
//! behave identically on the AST, IR, and bytecode interpreters (freestanding
//! tier stage 2, `documents/freestanding_tier_design.md` §2.2).
//!
//! The delivered raw-pointer builtins (`ptr_read`/`ptr_write`/`int_to_ptr`/
//! `ptr_to_int`/`volatile_*`) model a `ptr<T>` as an abstract *heap-slot handle*:
//! a single slot holds one `Value`, and `ptr_to_int` returns the slot index. That
//! model has no notion of *adjacent* addresses, so it cannot express pointer
//! arithmetic (`ptr_offset`) or the address-of an array element that a kernel
//! walks. This module adds a second, *byte-addressed* address space that lives
//! **above** [`RAW_POINTER_BASE`] so it never collides with a small heap-slot
//! handle: an `addr_of` snapshots the addressed place into a contiguous region of
//! typed cells with a fixed element `stride` (the C-natural `size_of` of the
//! element), and returns the region's byte base. `ptr_offset(p, n)` advances the
//! byte address by `n * stride`, and a read/write maps the byte address back to
//! `cell (addr - base) / stride`. This makes the observable **size law**
//! `ptr_to_int(ptr_offset(p, 1)) - ptr_to_int(p) == size_of(T)` hold on the
//! interpreters exactly as it does in real native addressing.
//!
//! The region snapshots the place by value, so a write *through* an `addr_of`
//! pointer mutates the snapshot, not the original binding — the interpreters model
//! raw reads/walks (the freestanding fixtures), while true byte-exact aliasing is a
//! native-codegen concern (as it already is for `volatile_*`). The native backend
//! does not yet compile the raw-pointer surface at all (a function using it skips
//! to the interpreters), so there is no native/interpreter divergence to reconcile
//! for these builtins today.

use crate::Value;

/// Base of the interpreters' freestanding raw-pointer byte-address space. Kept far
/// above any plausible heap-slot index (`Vec` indices are small) so an
/// `addr_of`-derived byte address is never confused with an `alloc`/`int_to_ptr`
/// heap-slot handle. `1 << 44` leaves 44 bits of low address space for handles and
/// ample room above for many regions.
pub const RAW_POINTER_BASE: usize = 1usize << 44;

/// One contiguous, byte-addressed snapshot region produced by an `addr_of`. Cell
/// `k` occupies bytes `[base + k*stride, base + (k+1)*stride)`.
struct RawRegion {
    base: usize,
    stride: usize,
    cells: Vec<Value>,
}

impl RawRegion {
    /// The exclusive end byte address of the region.
    fn end(&self) -> usize {
        self.base + self.stride.saturating_mul(self.cells.len())
    }

    /// The cell index a byte address maps to, if the address lies within the
    /// region's occupied span. Floors to the containing cell (a raw reinterpret of
    /// an unaligned interior address).
    fn cell_index(&self, addr: usize) -> Option<usize> {
        if self.stride == 0 || addr < self.base || addr >= self.end() {
            return None;
        }
        Some((addr - self.base) / self.stride)
    }
}

/// The interpreters' raw-pointer byte-address space. One instance per interpreter,
/// disjoint from the abstract heap. Empty until the first `addr_of`.
pub struct RawPointerMemory {
    regions: Vec<RawRegion>,
    next_base: usize,
}

impl Default for RawPointerMemory {
    fn default() -> Self {
        Self {
            regions: Vec::new(),
            next_base: RAW_POINTER_BASE,
        }
    }
}

/// The C-natural element stride of an array snapshot: `size_of(T)` rounded up to
/// `align_of(T)` for the first element, matching [`Value::layout_size`]'s array
/// formula. An empty array has no element type, so it falls back to a pointer-word
/// stride (its cells are never read). Returns `None` only for an element with no
/// defined layout (a heap/growable value), which the type checker already forbids
/// as an `addr_of` element.
fn array_stride(cells: &[Value]) -> Option<usize> {
    match cells.first() {
        Some(element) => {
            let size = element.layout_size()?;
            let align = element.layout_align()?;
            Some((((size + align - 1) / align) * align) as usize)
        }
        None => Some(8),
    }
}

/// The stride of a single scalar/pointer/struct snapshot cell — its `size_of`.
/// Falls back to a pointer word for a value with no defined layout (defensive; the
/// checker forbids such `addr_of` targets).
fn scalar_stride(value: &Value) -> usize {
    value.layout_size().unwrap_or(8) as usize
}

impl RawPointerMemory {
    /// Whether a byte address belongs to this raw-pointer space (vs. an abstract
    /// heap-slot handle). Used by `ptr_read`/`ptr_write`/`volatile_*` to route a
    /// pointer to the byte-addressed region model instead of the heap.
    pub fn is_raw(addr: usize) -> bool {
        addr >= RAW_POINTER_BASE
    }

    /// Reserve a fresh region for `cells` with element `stride`, returning its byte
    /// base. Regions are laid out consecutively with a one-stride guard gap so two
    /// regions never share an address.
    fn push_region(&mut self, cells: Vec<Value>, stride: usize) -> usize {
        let base = self.next_base;
        let span = stride.saturating_mul(cells.len()).max(stride).max(8);
        self.next_base = base + span + stride.max(8);
        self.regions.push(RawRegion {
            base,
            stride,
            cells,
        });
        base
    }

    /// `addr_of(place)` for a scalar/struct place or a whole array: snapshot the
    /// value into a region and return a pointer to its first byte. A `Value::Array`
    /// becomes a multi-cell region whose base points at element 0 (so `ptr_offset`
    /// walks it); any other value becomes a single-cell region.
    pub fn addr_of_value(&mut self, value: Value) -> Option<Value> {
        let addr = match value {
            Value::Array(cells) => {
                let cells = cells.into_vec();
                let stride = array_stride(&cells)?;
                self.push_region(cells, stride)
            }
            scalar => {
                let stride = scalar_stride(&scalar);
                self.push_region(vec![scalar], stride)
            }
        };
        Some(Value::Ptr(addr))
    }

    /// `addr_of(array[index])`: snapshot the whole `cells` array into a region and
    /// return a pointer to element `index` (`base + index*stride`). A negative or
    /// out-of-range index still yields a byte address (raw arithmetic is unchecked
    /// in `unsafe`); a later read of an out-of-range address fails.
    pub fn addr_of_element(&mut self, cells: Vec<Value>, index: i64) -> Option<Value> {
        let stride = array_stride(&cells)?;
        let base = self.push_region(cells, stride);
        let addr = base as i64 + index * stride as i64;
        Some(Value::Ptr(addr as usize))
    }

    /// `ptr_offset(p, n)`: advance the byte address by `n * size_of(T)`, where the
    /// element stride is taken from the region `p` points into. Returns `None` when
    /// `p` is not an `addr_of`-derived raw pointer (e.g. an `alloc`/`int_to_ptr`
    /// heap-slot handle), which the abstract model cannot walk.
    pub fn offset(&self, addr: usize, n: i64) -> Option<usize> {
        let stride = self.stride_of(addr)?;
        Some((addr as i64 + n * stride as i64) as usize)
    }

    /// The element stride of the region containing `addr`, if any.
    fn stride_of(&self, addr: usize) -> Option<usize> {
        self.regions
            .iter()
            .find(|region| addr >= region.base && addr < region.end())
            .map(|region| region.stride)
    }

    /// Read the cell a raw byte address maps to (`ptr_read`/`volatile_load` for a
    /// raw-space pointer). `None` when the address is not inside any region.
    pub fn load(&self, addr: usize) -> Option<Value> {
        self.regions.iter().find_map(|region| {
            region
                .cell_index(addr)
                .and_then(|index| region.cells.get(index).cloned())
        })
    }

    /// Write the cell a raw byte address maps to (`ptr_write`/`volatile_store` for a
    /// raw-space pointer), mutating the region snapshot. `false` when the address is
    /// not inside any region.
    pub fn store(&mut self, addr: usize, value: Value) -> bool {
        for region in &mut self.regions {
            if let Some(index) = region.cell_index(addr)
                && let Some(slot) = region.cells.get_mut(index)
            {
                *slot = value;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntKind;

    #[test]
    fn array_walk_and_size_law_hold() {
        let mut mem = RawPointerMemory::default();
        let cells = vec![Value::I64(10), Value::I64(20), Value::I64(30)];
        let Value::Ptr(base) = mem.addr_of_element(cells, 0).expect("addr_of") else {
            panic!("addr_of should yield a pointer");
        };
        // The address lives in the raw-pointer space, not the heap-slot space.
        assert!(RawPointerMemory::is_raw(base));
        // Walking reads consecutive elements.
        for (i, expected) in [10i64, 20, 30].iter().enumerate() {
            let addr = mem.offset(base, i as i64).expect("offset");
            assert_eq!(mem.load(addr), Some(Value::I64(*expected)));
        }
        // The size law: (base+1) - base == size_of(i64) == 8.
        let one = mem.offset(base, 1).expect("offset");
        assert_eq!(one - base, 8);
    }

    #[test]
    fn stride_matches_element_size_for_narrow_scalars() {
        let mut mem = RawPointerMemory::default();
        let cells = vec![Value::int(1, IntKind::I32), Value::int(2, IntKind::I32)];
        let Value::Ptr(base) = mem.addr_of_element(cells, 0).expect("addr_of") else {
            panic!("addr_of should yield a pointer");
        };
        // i32 stride is 4, so the size law reports 4, and offset 1 reads element 1.
        assert_eq!(mem.offset(base, 1).expect("offset") - base, 4);
        let one = mem.offset(base, 1).expect("offset");
        assert_eq!(mem.load(one), Some(Value::int(2, IntKind::I32)));
    }

    #[test]
    fn offset_on_a_non_region_pointer_is_none() {
        let mem = RawPointerMemory::default();
        // A heap-slot handle (small index) is not a walkable region.
        assert_eq!(mem.offset(3, 1), None);
        assert!(!RawPointerMemory::is_raw(3));
    }

    #[test]
    fn store_then_load_round_trips_within_a_region() {
        let mut mem = RawPointerMemory::default();
        let Value::Ptr(base) = mem.addr_of_value(Value::I64(41)).expect("addr_of scalar") else {
            panic!("addr_of should yield a pointer");
        };
        assert!(mem.store(base, Value::I64(99)));
        assert_eq!(mem.load(base), Some(Value::I64(99)));
    }
}
