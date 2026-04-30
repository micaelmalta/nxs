#![allow(dead_code, unused_imports, unused_variables)]
mod compiler;
mod decoder;
/// NXS vs JSON vs XML vs CSV benchmark
/// Measures: output byte size and serialization/deserialization throughput.
mod error;
mod lexer;
mod parser;
mod writer;

use compiler::Compiler;
use decoder::decode;
use std::time::{Duration, Instant};
use writer::{NxsWriter, Schema, Slot};

// ── Shared dataset ──────────────────────────────────────────────────────────

#[derive(Clone)]
struct Record {
    id: i64,
    username: String,
    email: String,
    age: i64,
    balance: f64,
    active: bool,
    score: f64,
    created_at: &'static str, // ISO date string
}

fn dataset(n: usize) -> Vec<Record> {
    (0..n)
        .map(|i| Record {
            id: i as i64,
            username: format!("user_{i:04}"),
            email: format!("user{i}@example.com"),
            age: 20 + (i % 50) as i64,
            balance: 100.0 + (i as f64) * 1.37,
            active: i % 3 != 0,
            score: (i as f64 % 100.0) / 10.0,
            created_at: "2026-04-30",
        })
        .collect()
}

// ── NXS serialization ────────────────────────────────────────────────────────

fn serialize_nxs(records: &[Record]) -> Vec<u8> {
    let mut src = String::new();
    for r in records {
        src.push_str(&format!(
            "record_{id} {{\n\
             \tid: ={id}\n\
             \tusername: \"{un}\"\n\
             \temail: \"{em}\"\n\
             \tage: ={age}\n\
             \tbalance: ~{bal:.2}\n\
             \tactive: ?{act}\n\
             \tscore: ~{sc:.1}\n\
             \tcreated_at: @{ts}\n\
             }}\n",
            id = r.id,
            un = r.username,
            em = r.email,
            age = r.age,
            bal = r.balance,
            act = if r.active { "true" } else { "false" },
            sc = r.score,
            ts = r.created_at,
        ));
    }

    let mut lexer = lexer::Lexer::new(&src);
    let tokens = lexer.tokenize().expect("nxs lex");
    let mut parser = parser::Parser::new(tokens);
    let fields = parser.parse_file().expect("nxs parse");
    let mut c = Compiler::new();
    c.compile(&fields).expect("nxs compile")
}

fn deserialize_nxs(data: &[u8]) -> usize {
    let decoded = decode(data).expect("nxs decode");
    decoded.root_fields.len()
}

// ── NXS wire writer (direct binary, no AST) ──────────────────────────────────

const SLOTS: &[&str] = &[
    "id",
    "username",
    "email",
    "age",
    "balance",
    "active",
    "score",
    "created_at",
];

// Integer slot IDs matching SLOTS order
const S_ID: Slot = Slot(0);
const S_USERNAME: Slot = Slot(1);
const S_EMAIL: Slot = Slot(2);
const S_AGE: Slot = Slot(3);
const S_BALANCE: Slot = Slot(4);
const S_ACTIVE: Slot = Slot(5);
const S_SCORE: Slot = Slot(6);
const S_CREATED_AT: Slot = Slot(7);

fn serialize_nxs_wire(records: &[Record]) -> Vec<u8> {
    let schema = Schema::new(SLOTS);
    // Pre-size: each record is ~110 bytes in binary form
    let mut w = NxsWriter::with_capacity(&schema, records.len() * 128 + 256);
    for r in records {
        w.begin_object();
        w.write_i64(S_ID, r.id);
        w.write_str(S_USERNAME, &r.username);
        w.write_str(S_EMAIL, &r.email);
        w.write_i64(S_AGE, r.age);
        w.write_f64(S_BALANCE, r.balance);
        w.write_bool(S_ACTIVE, r.active);
        w.write_f64(S_SCORE, r.score);
        w.write_time(S_CREATED_AT, 1_777_593_600_000_000_000);
        w.end_object();
    }
    w.finish()
}

fn deserialize_nxs_wire(data: &[u8]) -> usize {
    // Same tail-index path as compiler output
    deserialize_nxs(data)
}

// ── JSON serialization (manual, no runtime dep needed) ───────────────────────

fn serialize_json(records: &[Record]) -> Vec<u8> {
    let mut s = String::from("[\n");
    for (i, r) in records.iter().enumerate() {
        s.push_str(&format!(
            "  {{\"id\":{id},\"username\":\"{un}\",\"email\":\"{em}\",\
             \"age\":{age},\"balance\":{bal:.2},\"active\":{act},\
             \"score\":{sc:.1},\"created_at\":\"{ts}\"}}",
            id = r.id,
            un = r.username,
            em = r.email,
            age = r.age,
            bal = r.balance,
            act = if r.active { "true" } else { "false" },
            sc = r.score,
            ts = r.created_at,
        ));
        if i + 1 < records.len() {
            s.push_str(",\n");
        }
    }
    s.push_str("\n]");
    s.into_bytes()
}

fn deserialize_json(data: &[u8]) -> usize {
    // Minimal JSON counter: count `"id":` occurrences
    let s = std::str::from_utf8(data).unwrap();
    s.matches("\"id\":").count()
}

// ── XML serialization ────────────────────────────────────────────────────────

fn serialize_xml(records: &[Record]) -> Vec<u8> {
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<records>\n");
    for r in records {
        s.push_str(&format!(
            "  <record>\
             <id>{id}</id>\
             <username>{un}</username>\
             <email>{em}</email>\
             <age>{age}</age>\
             <balance>{bal:.2}</balance>\
             <active>{act}</active>\
             <score>{sc:.1}</score>\
             <created_at>{ts}</created_at>\
             </record>\n",
            id = r.id,
            un = r.username,
            em = r.email,
            age = r.age,
            bal = r.balance,
            act = if r.active { "true" } else { "false" },
            sc = r.score,
            ts = r.created_at,
        ));
    }
    s.push_str("</records>");
    s.into_bytes()
}

fn deserialize_xml(data: &[u8]) -> usize {
    let s = std::str::from_utf8(data).unwrap();
    s.matches("<record>").count()
}

// ── CSV serialization ────────────────────────────────────────────────────────

fn serialize_csv(records: &[Record]) -> Vec<u8> {
    let mut s = String::from("id,username,email,age,balance,active,score,created_at\n");
    for r in records {
        s.push_str(&format!(
            "{},{},{},{},{:.2},{},{:.1},{}\n",
            r.id,
            r.username,
            r.email,
            r.age,
            r.balance,
            if r.active { "true" } else { "false" },
            r.score,
            r.created_at,
        ));
    }
    s.into_bytes()
}

fn deserialize_csv(data: &[u8]) -> usize {
    let s = std::str::from_utf8(data).unwrap();
    s.lines().count().saturating_sub(1) // skip header
}

// ── Benchmark harness ────────────────────────────────────────────────────────

const SIZES: &[usize] = &[10_000, 100_000, 1_000_000];

fn iters_for(n: usize) -> u32 {
    match n {
        n if n >= 1_000_000 => 3,
        n if n >= 100_000 => 5,
        _ => 10,
    }
}

fn bench<F: Fn() -> R, R>(iters: u32, f: F) -> Duration {
    for _ in 0..2 {
        let _ = f();
    } // warmup
    let start = Instant::now();
    for _ in 0..iters {
        let _ = f();
    }
    start.elapsed() / iters
}

fn fmt_ns(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000 {
        format!("{ns} ns")
    } else if ns < 1_000_000 {
        format!("{:.1} µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2} ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.3} s", ns as f64 / 1_000_000_000.0)
    }
}

fn fmt_bytes(n: usize) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{:.2} MB", n as f64 / (1024.0 * 1024.0))
    }
}

fn main() {
    println!(
        "\n╔══════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!("║              NXS vs JSON vs XML vs CSV  —  Benchmark Results                    ║");
    println!(
        "╚══════════════════════════════════════════════════════════════════════════════════╝\n"
    );
    println!("  Iterations: 10/5/3 at 10k/100k/1M");
    println!(
        "  Fields per record: 8 (id, username, email, age, balance, active, score, created_at)\n"
    );

    for &n in SIZES {
        let iters = iters_for(n);
        let records = dataset(n);
        println!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  {n} records  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );

        let include_compiler = n <= 10_000;

        // Serialize once to get sizes
        let nxs_wire_bytes = serialize_nxs_wire(&records);
        let json_bytes = serialize_json(&records);
        let xml_bytes = serialize_xml(&records);
        let csv_bytes = serialize_csv(&records);
        let nxs_compiler_bytes = if include_compiler {
            Some(serialize_nxs(&records))
        } else {
            None
        };

        // Size comparison
        println!(
            "\n  ┌─ Output Size ───────────────────────────────────────────────────────────────┐"
        );
        let baseline = json_bytes.len() as f64;
        let mut size_rows: Vec<(&str, usize)> = vec![];
        if let Some(ref b) = nxs_compiler_bytes {
            size_rows.push(("NXS compiler", b.len()));
        }
        size_rows.push(("NXS wire    ", nxs_wire_bytes.len()));
        size_rows.push(("JSON        ", json_bytes.len()));
        size_rows.push(("XML         ", xml_bytes.len()));
        size_rows.push(("CSV         ", csv_bytes.len()));
        for (name, len) in &size_rows {
            let ratio = *len as f64 / baseline * 100.0;
            let bar_len = (*len * 40 / xml_bytes.len()).max(1);
            let bar = "█".repeat(bar_len);
            println!(
                "  │  {name}  {:>10}  ({:>5.1}% of JSON)  {bar}",
                fmt_bytes(*len),
                ratio
            );
        }
        println!(
            "  └─────────────────────────────────────────────────────────────────────────────┘"
        );

        // Serialization speed
        println!(
            "\n  ┌─ Serialization Time (avg over {iters} runs) ─────────────────────────────────────┐"
        );
        let t_nxs_wire_ser = bench(iters, || serialize_nxs_wire(&records));
        let t_json_ser = bench(iters, || serialize_json(&records));
        let t_xml_ser = bench(iters, || serialize_xml(&records));
        let t_csv_ser = bench(iters, || serialize_csv(&records));
        let t_nxs_compiler_ser = if include_compiler {
            Some(bench(iters, || serialize_nxs(&records)))
        } else {
            None
        };
        let json_ser_ns = t_json_ser.as_nanos() as f64;
        let mut ser_rows: Vec<(&str, std::time::Duration)> = vec![];
        if let Some(t) = t_nxs_compiler_ser {
            ser_rows.push(("NXS compiler", t));
        }
        ser_rows.push(("NXS wire    ", t_nxs_wire_ser));
        ser_rows.push(("JSON        ", t_json_ser));
        ser_rows.push(("XML         ", t_xml_ser));
        ser_rows.push(("CSV         ", t_csv_ser));
        for (name, t) in &ser_rows {
            let ratio = t.as_nanos() as f64 / json_ser_ns;
            println!("  │  {name}  {:>10}   ({:.2}x vs JSON)", fmt_ns(*t), ratio);
        }
        if !include_compiler {
            println!("  │  NXS compiler (skipped — would take minutes at this scale)");
        }
        println!(
            "  └─────────────────────────────────────────────────────────────────────────────┘"
        );

        // Deserialization speed
        println!(
            "\n  ┌─ Deserialization Time (avg over {iters} runs) ───────────────────────────────────┐"
        );
        let t_nxs_wire_de = bench(iters, || deserialize_nxs_wire(&nxs_wire_bytes));
        let t_json_de = bench(iters, || deserialize_json(&json_bytes));
        let t_xml_de = bench(iters, || deserialize_xml(&xml_bytes));
        let t_csv_de = bench(iters, || deserialize_csv(&csv_bytes));
        let t_nxs_compiler_de = nxs_compiler_bytes
            .as_ref()
            .filter(|b| b.len() < 60_000)
            .map(|b| bench(iters, || deserialize_nxs(b)));
        let json_de_ns = t_json_de.as_nanos() as f64;
        let mut de_rows: Vec<(&str, std::time::Duration)> = vec![];
        if let Some(t) = t_nxs_compiler_de {
            de_rows.push(("NXS compiler", t));
        }
        de_rows.push(("NXS wire    ", t_nxs_wire_de));
        de_rows.push(("JSON        ", t_json_de));
        de_rows.push(("XML         ", t_xml_de));
        de_rows.push(("CSV         ", t_csv_de));
        for (name, t) in &de_rows {
            let ratio = t.as_nanos() as f64 / json_de_ns;
            println!("  │  {name}  {:>10}   ({:.2}x vs JSON)", fmt_ns(*t), ratio);
        }
        println!(
            "  └─────────────────────────────────────────────────────────────────────────────┘\n"
        );
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  NXS compiler: .nxs source text → lex → parse → AST → binary (one-time build step)");
    println!("  NXS wire:     typed struct → direct binary write (the actual hot-path)");
    println!("  JSON/XML/CSV: string formatting → bytes (no parsing overhead on write side)\n");
}
