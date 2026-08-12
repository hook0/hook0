/// Ceilings applied to every snapshot the generator accepts.
///
/// Nothing is ever truncated: a snapshot that crosses one of these ceilings is rejected with the
/// count it reached and the ceiling it crossed. Strings that are not identifiers (paths, summaries,
/// descriptions) are bounded transitively by [`Limits::max_snapshot_bytes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Largest snapshot accepted, in bytes, checked before any parsing happens.
    pub max_snapshot_bytes: usize,
    /// Longest chain of `$ref` hops followed before giving up. Also what stops a reference cycle.
    pub max_reference_depth: usize,
    /// Largest number of operations a snapshot may declare, counted before the public tag filter.
    pub max_operations: usize,
    /// Largest number of entities a model may carry.
    pub max_entities: usize,
    /// Largest number of methods a single entity may carry.
    pub max_methods_per_entity: usize,
    /// Largest number of parameters a single operation may carry, path-level ones included.
    pub max_parameters_per_operation: usize,
    /// Longest operation id or parameter name accepted, in bytes.
    pub max_identifier_bytes: usize,
}

impl Limits {
    /// Ceilings used when the caller has no reason to pick its own.
    ///
    /// Each one sits an order of magnitude above what the Hook0 API declares today, so a snapshot
    /// crossing one signals a spec that grew out of shape rather than a normal release.
    pub const DEFAULT: Self = Self {
        max_snapshot_bytes: 8 * 1024 * 1024,
        max_reference_depth: 8,
        max_operations: 1024,
        max_entities: 256,
        max_methods_per_entity: 64,
        max_parameters_per_operation: 64,
        max_identifier_bytes: 128,
    };
}

impl Default for Limits {
    fn default() -> Self {
        Self::DEFAULT
    }
}
