//! Reads a `.puffin` file and prints per-scope timing statistics.
//!
//! Usage: cargo run --bin profile_summary --features profiling -- profile.puffin

use std::collections::BTreeMap;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: profile_summary <file.puffin>");
        std::process::exit(1);
    });

    let mut file = std::fs::File::open(&path).expect("failed to open file");
    let view = puffin::FrameView::read(&mut file).expect("failed to read puffin data");

    let frames: Vec<_> = view.recent_frames().cloned().collect();
    if frames.is_empty() {
        eprintln!("No frames found in {path}");
        return;
    }

    println!("Frames: {}", frames.len());

    // Collect per-scope stats: (count, total_ns, min_ns, max_ns)
    let mut scope_stats: BTreeMap<String, (u64, i64, i64, i64)> = BTreeMap::new();
    let mut frame_durations_ns: Vec<i64> = Vec::new();

    let scope_collection = view.scope_collection();

    for frame in &frames {
        let unpacked = match frame.unpacked() {
            Ok(u) => u,
            Err(_) => continue,
        };

        let duration_ns = unpacked.duration_ns();
        frame_durations_ns.push(duration_ns);

        for stream_info in unpacked.thread_streams.values() {
            let Ok(top_scopes) =
                puffin::Reader::from_start(&stream_info.stream).read_top_scopes()
            else {
                continue;
            };
            collect_scopes(
                &top_scopes,
                &stream_info.stream,
                scope_collection,
                &mut scope_stats,
            );
        }
    }

    // Print frame duration stats.
    frame_durations_ns.sort();
    let n = frame_durations_ns.len();
    let avg_ms = frame_durations_ns.iter().copied().sum::<i64>() as f64 / n as f64 / 1e6;
    let median_ms = frame_durations_ns[n / 2] as f64 / 1e6;
    let p95_ms = frame_durations_ns[(n as f64 * 0.95) as usize] as f64 / 1e6;
    let p99_ms = frame_durations_ns[(n as f64 * 0.99) as usize] as f64 / 1e6;
    let max_ms = frame_durations_ns[n - 1] as f64 / 1e6;
    let min_ms = frame_durations_ns[0] as f64 / 1e6;

    println!("\n=== Frame Duration ===");
    println!(
        "  avg: {avg_ms:.2}ms  median: {median_ms:.2}ms  p95: {p95_ms:.2}ms  p99: {p99_ms:.2}ms"
    );
    println!("  min: {min_ms:.2}ms  max: {max_ms:.2}ms");
    println!(
        "  effective fps: {:.0} (avg)  {:.0} (p95)",
        1000.0 / avg_ms,
        1000.0 / p95_ms
    );

    // Print per-scope stats sorted by avg time descending.
    println!("\n=== Per-Scope Timing (sorted by avg time) ===");
    println!(
        "{:<35} {:>6} {:>10} {:>10} {:>10} {:>10}",
        "Scope", "Count", "Total(ms)", "Avg(ms)", "Min(ms)", "Max(ms)"
    );
    println!("{}", "-".repeat(91));

    let mut sorted: Vec<_> = scope_stats.iter().collect();
    sorted.sort_by(|a, b| {
        let avg_a = a.1 .1 as f64 / a.1 .0 as f64;
        let avg_b = b.1 .1 as f64 / b.1 .0 as f64;
        avg_b.partial_cmp(&avg_a).unwrap()
    });

    for (name, (count, total_ns, min_ns, max_ns)) in &sorted {
        let total_ms = *total_ns as f64 / 1e6;
        let avg_ms = *total_ns as f64 / *count as f64 / 1e6;
        let min_ms = *min_ns as f64 / 1e6;
        let max_ms = *max_ns as f64 / 1e6;
        println!(
            "{:<35} {:>6} {:>10.2} {:>10.3} {:>10.3} {:>10.3}",
            truncate(name, 35),
            count,
            total_ms,
            avg_ms,
            min_ms,
            max_ms
        );
    }

    // Print slowest frames.
    println!("\n=== 10 Slowest Frames ===");
    let mut indexed: Vec<_> = frame_durations_ns.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));
    for (i, dur_ns) in indexed.iter().take(10) {
        println!("  Frame {:>5}: {:.2}ms", i, *dur_ns as f64 / 1e6);
    }
}

fn collect_scopes(
    scopes: &[puffin::Scope<'_>],
    stream: &puffin::Stream,
    scope_collection: &puffin::ScopeCollection,
    stats: &mut BTreeMap<String, (u64, i64, i64, i64)>,
) {
    for scope in scopes {
        let name = scope_name(scope, scope_collection);

        let dur_ns = scope.record.duration_ns;
        let entry = stats.entry(name).or_insert((0, 0, i64::MAX, 0));
        entry.0 += 1;
        entry.1 += dur_ns;
        entry.2 = entry.2.min(dur_ns);
        entry.3 = entry.3.max(dur_ns);

        // Recurse into children.
        if scope.child_begin_position < scope.child_end_position {
            if let Ok(reader) =
                puffin::Reader::with_offset(stream, scope.child_begin_position)
            {
                if let Ok(children) = reader.read_top_scopes() {
                    let children: Vec<_> = children
                        .into_iter()
                        .filter(|c| {
                            c.record.start_ns + c.record.duration_ns
                                <= scope.record.start_ns + scope.record.duration_ns
                        })
                        .collect();
                    collect_scopes(&children, stream, scope_collection, stats);
                }
            }
        }
    }
}

fn scope_name(scope: &puffin::Scope<'_>, scope_collection: &puffin::ScopeCollection) -> String {
    scope_collection
        .fetch_by_id(&scope.id)
        .map(|d| {
            let func = if d.function_name.is_empty() {
                String::new()
            } else {
                short_fn(&d.function_name)
            };
            let scope_name = d.scope_name.as_deref().unwrap_or("");
            if func.is_empty() {
                scope_name.to_string()
            } else if scope_name.is_empty() || scope_name == func {
                func
            } else {
                format!("{func}::{scope_name}")
            }
        })
        .unwrap_or_else(|| format!("scope_{}", scope.id.0))
}

fn short_fn(full: &str) -> String {
    // "spout::main::Spout::render" -> "Spout::render"
    let parts: Vec<&str> = full.rsplitn(3, "::").collect();
    if parts.len() >= 2 {
        format!("{}::{}", parts[1], parts[0])
    } else {
        full.to_string()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
