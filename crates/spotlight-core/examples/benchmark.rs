use spotlight_core::{
    model::{Entry, Kind},
    search::Index,
};
use std::time::Instant;

fn main() {
    let count = 50_000;
    let entries = (0..count)
        .map(|i| Entry {
            id: format!("file:{i}"),
            name: format!("project-notes-{i:05}.md"),
            path: format!(
                "/home/benchmark/projects/project-{}/notes-{i:05}.md",
                i % 100
            ),
            kind: Kind::File,
            keywords: String::new(),
        })
        .collect();
    let start = Instant::now();
    let index = Index::new(entries);
    println!(
        "Synthetic entries: {count}; index construction: {:.2} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );
    let queries = [
        "project-notes-49999",
        "pjn499",
        "notes",
        "project 123",
        "no-matches",
        "",
    ];
    let mut timings = Vec::new();
    for i in 0..120 {
        let response = index.search(queries[i % queries.len()], None, 30);
        std::hint::black_box(&response.results);
        if i >= 20 {
            timings.push(response.elapsed_ms);
        }
    }
    timings.sort_by(f64::total_cmp);
    println!(
        "100 warm queries: p50 {:.2} ms; p95 {:.2} ms; max {:.2} ms",
        timings[49], timings[94], timings[99]
    );
    println!("Engine only; excludes IPC, rendering, filesystem indexing, and application startup.");
}
