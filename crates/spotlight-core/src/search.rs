use crate::model::{Entry, Kind, SearchResponse};
use std::{cmp::Reverse, collections::BinaryHeap, time::Instant};

struct Searchable {
    entry: Entry,
    name: String,
    context: String,
}

#[derive(Default)]
pub struct Index {
    items: Vec<Searchable>,
}

impl Index {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self {
            items: entries
                .into_iter()
                .map(|entry| Searchable {
                    name: entry.name.to_lowercase(),
                    context: format!("{} {}", entry.path, entry.keywords).to_lowercase(),
                    entry,
                })
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.items
            .iter()
            .find(|item| item.entry.id == id)
            .map(|item| &item.entry)
    }

    pub fn search(&self, query: &str, kind: Option<Kind>, limit: usize) -> SearchResponse {
        let start = Instant::now();
        let query = query.trim().to_lowercase();
        let tokens: Vec<_> = query.split_whitespace().collect();
        let mut best = BinaryHeap::new();
        let limit = limit.clamp(1, 100);
        for (position, item) in self.items.iter().enumerate() {
            if kind.is_some_and(|kind| kind != item.entry.kind) {
                continue;
            }
            let score = if tokens.is_empty() {
                Some(if item.entry.kind == Kind::App {
                    200
                } else if item.entry.kind == Kind::Folder {
                    100
                } else {
                    0
                })
            } else {
                tokens.iter().try_fold(0, |total, token| {
                    name_score(&item.name, token)
                        .or_else(|| item.context.contains(token).then_some(100))
                        .map(|score| total + score)
                })
            };
            if let Some(score) = score {
                let score = score + i32::from(item.entry.kind == Kind::App) * 20
                    - item.name.chars().count().min(100) as i32;
                best.push(Reverse((score, Reverse(position))));
                if best.len() > limit {
                    best.pop();
                }
            }
        }
        let mut scored: Vec<_> = best
            .into_iter()
            .map(|Reverse((score, Reverse(i)))| (score, &self.items[i].entry))
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
        SearchResponse {
            results: scored.into_iter().map(|(_, entry)| entry.clone()).collect(),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            total: self.len(),
        }
    }
}

fn name_score(name: &str, query: &str) -> Option<i32> {
    if name == query {
        return Some(2000);
    }
    if name.starts_with(query) {
        return Some(1500);
    }
    if let Some(position) = name.find(query) {
        return Some(1100 - position.min(100) as i32);
    }
    let mut letters = query.chars();
    let mut wanted = letters.next()?;
    let mut gap = 0;
    for letter in name.chars() {
        if letter == wanted {
            match letters.next() {
                Some(next) => wanted = next,
                None => return Some(500 - gap.min(300)),
            }
        } else {
            gap += 1;
        }
    }
    None
}
