# Performance Tuning Guide

This document provides guidelines for optimizing beads-tui performance, understanding performance characteristics, and troubleshooting performance issues.

## Performance Characteristics

### Benchmarked Operations

beads-tui includes comprehensive benchmarks for common operations. Run benchmarks with:

```bash
cargo bench
```

Key benchmarked operations:
- **Filter operations**: ~100-500µs for 1000 issues
- **Search operations**: ~1-5ms for substring search on 1000 issues
- **JSON parsing**: ~4ms for 500 issues from bd CLI output
- **List navigation**: ~1µs per navigation operation

See `benches/README.md` for detailed benchmark information.

### Scalability Targets

beads-tui is designed to handle:
- **1,000 issues**: Excellent performance, no tuning needed
- **10,000 issues**: Good performance with default settings
- **100,000+ issues**: May require configuration tuning

## Performance Optimizations

### Built-in Optimizations

beads-tui includes several automatic optimizations:

#### 1. Hierarchy Caching
Issue dependency hierarchies are cached to avoid recomputation:
- Automatic cache invalidation on data changes
- ~10x speedup for repeated tree rendering
- Memory usage: ~1KB per 100 issues in hierarchy

#### 2. Query Caching
Filter and search queries are cached:
- LRU cache with configurable size
- Cache hit rate typically >80% for repeated queries
- Automatic invalidation on issue modifications

#### 3. Incremental Rendering
UI updates use incremental rendering where possible:
- Only changed portions of the screen are redrawn
- Reduces CPU usage during interactive operations
- Particularly effective for cursor movement and scrolling

### Configuration Options

#### Table Row Height

Adjust row height in table configuration (affects rendering performance):

```rust
// In config.toml or via Column Manager UI
[table]
row_height = 1  # Faster rendering, less content visible
row_height = 3  # Slower rendering, more content visible
```

**Recommendation**: Use `row_height = 1` for large issue lists (>1000 issues).

#### Column Visibility

Disable unused columns to improve rendering performance:

```bash
# Access Column Manager with 'c' key in Issues view
# Hide expensive columns like Description, Labels, Notes
```

**Impact**: Each hidden column saves ~50-100µs per row render.

#### Filter Complexity

Simpler filters are faster:

```rust
// Fast: Single field filters
status:open

// Medium: Multiple field filters with AND
status:open priority:P0

// Slower: Complex text searches
~database AND (description OR title)
```

**Recommendation**: Use status/priority/type filters before adding text search.

## Troubleshooting Performance Issues

### Slow List Rendering

**Symptoms**: Lag when scrolling through issue lists

**Causes**:
- Large number of visible columns
- Multi-line row heights
- Very large issue counts (>10,000)

**Solutions**:
1. Reduce visible columns (hide Description, Labels if not needed)
2. Use single-line row height (`row_height = 1`)
3. Apply filters to reduce visible issue count
4. Consider pagination (issue lists auto-paginate at 1000 items)

### Slow Search Operations

**Symptoms**: Delay when typing in search box

**Causes**:
- Full-text search on large datasets
- Regex patterns
- Fuzzy matching enabled

**Solutions**:
1. Use simpler search patterns (exact match vs regex)
2. Add filters before searching to reduce search space
3. Search specific fields (title:pattern) instead of all fields
4. Disable fuzzy matching if not needed

### High Memory Usage

**Symptoms**: Memory usage >500MB

**Causes**:
- Very large issue lists loaded
- Deep dependency trees
- Large cache sizes

**Solutions**:
1. Close unused tabs to free memory
2. Clear filter cache manually if needed
3. Restart application periodically for very long sessions
4. Use pagination for large lists

### Slow Startup

**Symptoms**: Application takes >5s to start

**Causes**:
- Large issue database (>10,000 issues)
- Slow beads CLI performance
- Git repository issues

**Solutions**:
1. Run `bd doctor` to check for repository issues
2. Ensure git repository is not corrupted
3. Consider using `bd sync` to clean up issue database
4. Check disk I/O performance

## Performance Monitoring

### Built-in Performance Stats

beads-tui tracks performance metrics internally:

```bash
# Toggle performance stats overlay (if enabled in build)
# Press 'P' (Shift+p) in the UI
```

Metrics displayed:
- Frame render time
- Event processing time
- Cache hit rates
- Memory usage estimates

### External Profiling

For detailed performance analysis:

#### CPU Profiling

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Run with profiling
cargo flamegraph --bin beads-tui

# Open flamegraph.svg in browser
```

#### Memory Profiling

```bash
# Install heaptrack (Linux)
sudo apt install heaptrack

# Profile memory usage
heaptrack target/debug/beads-tui

# Analyze results
heaptrack_gui heaptrack.beads-tui.*.gz
```

#### Benchmarking Specific Operations

```bash
# Run specific benchmark
cargo bench --bench common_operations -- filter_by_status

# Run with different dataset sizes
cargo bench --bench search_performance -- search_all_fields/5000

# Generate HTML reports
cargo bench
open target/criterion/report/index.html
```

## Best Practices

### For Small Repositories (<1,000 issues)

- Use default settings
- No tuning required
- All features enabled by default

### For Medium Repositories (1,000-10,000 issues)

- Consider hiding unused columns
- Use filters to focus on relevant issues
- Monitor memory usage periodically

### For Large Repositories (>10,000 issues)

- Use single-line row height
- Hide expensive columns (Description, Labels)
- Use filters aggressively to reduce visible set
- Consider pagination for search results
- Restart application periodically

### For CI/Automated Usage

- Use `bd` CLI directly instead of TUI for automation
- TUI is optimized for interactive use
- CLI is faster for scripted operations

## Performance Regression Testing

beads-tui includes performance regression tests in CI:

```bash
# Run locally
cargo bench --bench common_operations
cargo bench --bench search_performance

# Compare with baseline
cargo bench -- --baseline main
```

Benchmarks track:
- Filter operation performance
- Search operation performance
- Rendering performance
- Memory allocation patterns

## Future Optimizations

Planned performance improvements (see issue tracker):

- **Background task system**: Async operations without UI blocking
- **Comprehensive caching**: Cache more expensive computations
- **Resource management**: Better memory limits and monitoring
- **Progressive loading**: Load large datasets incrementally
- **Virtual scrolling**: Render only visible items

See the Performance & Optimization epic (beads-tui-9dor) for details.

## Getting Help

If you experience performance issues not covered here:

1. Run benchmarks to establish baseline: `cargo bench`
2. Profile the application to identify bottlenecks
3. Check issue tracker for known performance issues
4. File a performance bug with benchmark results

Include in bug reports:
- Repository size (issue count)
- Hardware specs (RAM, CPU)
- Specific operation that's slow
- Benchmark results if available
- Steps to reproduce
