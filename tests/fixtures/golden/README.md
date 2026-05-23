# Golden Test Files

Pre-computed reference output files for deterministic encoder verification.

## Generation
Run with: `cargo test -- --generate-golden`
(requires the PHOTOPIPELINE_GENERATE_GOLDEN env var to be set)

## Files
Each file is the expected output of a specific pipeline with a known test pattern input.
Binary files in this directory are used by `assert_golden_bytes()` in the test harness.

## Adding New Golden Files
1. Add a generator function in `tests/test_harness/src/fixtures/golden_patterns.rs`
2. Run `PHOTOPIPELINE_GENERATE_GOLDEN=1 cargo test -p photopipeline-e2e-tests -- --generate-golden`
3. Verify the generated file manually
4. Commit the binary file
