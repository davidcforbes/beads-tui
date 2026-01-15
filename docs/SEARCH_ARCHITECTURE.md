# Search & Filter Architecture Overhaul

## Overview
This document outlines the design for the "Search Engine V2" upgrade in Beads-TUI. The goal is to move from simple substring/exact matching to a powerful, developer-centric query language with support for fuzzy and semantic search.

## 1. Unified Query Language (The "Omnibar")

Instead of separate UI widgets for status, priority, and text search, we will implement a unified query syntax similar to GitHub/Jira/Linear.

### Syntax Specification
- **Keywords:** `key:value` pairs (e.g., `status:open`, `is:blocked`, `priority:p1`, `assignee:me`).
- **Text:** Any non-keyword token is treated as a text search term.
- **Quotes:** `"exact phrase"` matching.
- **Negation:** `-key:value` or `-term` to exclude.
- **Operators:** Support for `OR` logic (e.g., `priority:p1,p2`).

### Parser Architecture (`src/search/parser.rs`)
We will use a custom recursive descent parser or a lightweight parser combinator (like `nom` or `chumsky` if needed, but simple string splitting might suffice for V1) to transform the raw input string into a structured `QueryAST`.

```rust
pub enum QueryToken {
    Filter { field: String, value: String, negate: bool },
    Text(String),
    Exact(String),
}

pub struct QueryAST {
    pub filters: Vec<QueryToken>,
    pub text_query: String,
}
```

## 2. Search Engine V2 (`src/search/engine.rs`)

The search engine will be decoupled from the UI state. It will accept a `QueryAST` and a list of `Issue`s, returning a ranked list of matches.

### Ranking Strategy (Hybrid Scoring)
Issues will be scored based on a weighted sum:
1.  **Filter Score:** Boolean pass/fail (acts as a hard filter).
2.  **Text Score:**
    *   **Exact Match:** High score.
    *   **Fuzzy Match:** Uses `skim` or `nucleo` logic for typo tolerance.
    *   **BM25/TF-IDF:** (Future) Frequency-based relevance for longer descriptions.
3.  **Recency Boost:** Slight score boost for recently updated issues.

### Interface
```rust
pub trait SearchProvider {
    fn search(&self, query: &QueryAST, issues: &[Issue]) -> Vec<ScoredIssue>;
}
```

## 3. Semantic Search (Future/Plugin)

To support "semantic" search (finding issues by meaning rather than keywords), we will design the `SearchProvider` to optionally allow vector-based retrieval.

*   **Technology:** `fastembed-rs` (Rust bindings for ONNX runtime) to generate embeddings locally.
*   **Storage:** Store document vectors in a local binary file (`.beads/vectors.bin`).
*   **Integration:** If the `semantic` feature is enabled, the search engine will compute the cosine similarity between the query vector and issue vectors, merging this score into the final ranking.

## 4. UI Changes

*   **Remove:** The "Quick Filter" side menu and separate specific filter widgets.
*   **Add:** A prominent "Command Bar" at the top (or accessible via `/`).
*   **Visuals:** Syntax highlighting for keys (e.g., coloring `status:` in blue).

## 5. Implementation Phases

1.  **Phase 1: Query Parser & Logic**
    *   Implement the parser for `key:value` syntax.
    *   Update `IssueFilter` to accept this structured query.
2.  **Phase 2: Hybrid Search Engine**
    *   Implement the scoring algorithm (Fuzzy + Recency).
    *   Replace existing simple `filter()` iterator.
3.  **Phase 3: UI Integration**
    *   Replace Search Bar with Syntax-highlighted Input.
    *   Remove legacy filter widgets.
4.  **Phase 4: Semantic Spike**
    *   Experiment with `fastembed` for vector generation.
