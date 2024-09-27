pub const BLOCKS_PER_SECOND: u64 = 1;

/// This represents how big we want to allow a contract size to be in multiples of 24kb
/// 2 = 48kb
/// 3 = 72kb
pub const DEFAULT_MULTIPLY_VAL_FOR_CODE_SIZE: usize = 2;

/// Maximum bytecode to permit for a contract.
pub const MAX_CODE_BYTE_SIZE: usize = 24576 * DEFAULT_MULTIPLY_VAL_FOR_CODE_SIZE;

/// Default max request size in MB.
pub const DEFAULT_MAX_REQUEST_SIZE_MB: u32 = 37;
