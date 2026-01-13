# Benchmarks

This directory contains performance benchmarks for beads-tui using Criterion.

## Running Benchmarks

Run all benchmarks:
```bash
cargo bench
```

Run specific benchmark:
```bash
cargo bench --bench common_operations
cargo bench --bench search_performance
```

Test benchmark compilation without running:
```bash
cargo bench --bench common_operations -- --test
cargo bench --bench search_performance -- --test
```

## Benchmarks

### common_operations

Benchmarks for common issue management operations:

- **filter_by_status**: Filter issues by single status (10, 100, 500, 1000 issues)
- **filter_multiple_criteria**: Filter by multiple criteria (status + priority + type)
- **search_by_title**: Case-sensitive title search
- **search_case_insensitive**: Case-insensitive search
- **issue_list_sorting**: Sort by priority, title, and updated date
- **parse_json_list**: Parse JSON output from bd CLI (10, 100, 500 issues)
- **issue_list_navigation**: Navigate through issue list with cursor movement

### search_performance

Comprehensive benchmarks for search functionality on large datasets:

- **search_all_fields_substring**: Substring search across all issue fields (100, 500, 1000, 5000 issues)
- **search_fuzzy_matching**: Fuzzy search with typo tolerance (100, 500, 1000, 5000 issues)
- **search_regex_matching**: Regex pattern matching (100, 500, 1000, 5000 issues)
- **search_with_query_cache**: Tests query caching optimization effectiveness
- **search_with_status_filter**: Combined search + filter operations
- **search_case_insensitive**: Case-insensitive search performance
- **search_large_datasets**: Stress testing with 5000 and 10000 issues
- **search_different_scopes**: Compare performance across Title, Description, and All fields scopes

These benchmarks help identify performance bottlenecks and validate optimizations for search operations on large issue datasets.

## Results

Benchmark results are stored in `target/criterion/` and can be viewed with the generated HTML reports.

## Adding New Benchmarks

1. Create a new file in `benches/` (e.g., `benches/rendering.rs`)
2. Add benchmark configuration to `Cargo.toml`:
   ```toml
   [[bench]]
   name = "rendering"
   harness = false
   ```
3. Use Criterion's API to define benchmarks
4. Run `cargo bench --bench rendering` to test
