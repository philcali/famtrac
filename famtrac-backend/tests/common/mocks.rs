// Re-export mock repositories from the main crate
// This allows integration tests to use the same mocks as unit tests

pub use famtrac_backend::test_utils::mocks::*;
