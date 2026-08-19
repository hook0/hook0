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
    /// Largest number of operations a snapshot may declare, counted before the SDK tag filter.
    pub max_operations: usize,
    /// Largest number of entities a model may carry.
    pub max_entities: usize,
    /// Largest number of methods a single entity may carry.
    pub max_methods_per_entity: usize,
    /// Largest number of parameters a single operation may carry, path-level ones included.
    pub max_parameters_per_operation: usize,
    /// Longest operation id or parameter name accepted, in bytes.
    pub max_identifier_bytes: usize,
    /// Largest number of words a name may split into. A name past this is not one an emitter could
    /// spell out under any casing, since every word costs at least one byte of the identifier.
    pub max_words_per_identifier: usize,
    /// Largest number of object types a model may carry, the ones derived from inline schemas
    /// included.
    pub max_schemas: usize,
    /// Largest number of fields a single object type may carry.
    pub max_fields_per_object: usize,
    /// Largest number of values a single closed enum may carry.
    pub max_enum_values: usize,
    /// Deepest a schema may nest before the reader gives up, which is also what stops an
    /// arbitrarily nested document from growing the stack.
    pub max_shape_depth: usize,
    /// Longest path an emitted file may sit at under its target root, in bytes.
    pub max_path_bytes: usize,
    /// Deepest an emitted file may sit under its target root, counted in segments. Also what
    /// bounds the walk of a target root.
    pub max_path_depth: usize,
    /// Largest number of files a single emission may carry.
    pub max_emitted_files: usize,
    /// Largest number of bytes a single emission may carry, all files taken together.
    pub max_emitted_bytes: usize,
    /// Largest number of entries walked under a target root before the walk gives up. A root
    /// holding more than this is not a generated tree.
    pub max_target_entries: usize,
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
        max_words_per_identifier: 64,
        max_schemas: 512,
        max_fields_per_object: 256,
        max_enum_values: 512,
        max_shape_depth: 16,
        max_path_bytes: 256,
        max_path_depth: 16,
        max_emitted_files: 4096,
        max_emitted_bytes: 32 * 1024 * 1024,
        max_target_entries: 20_000,
    };
}

impl Default for Limits {
    fn default() -> Self {
        Self::DEFAULT
    }
}
