# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

NXS (Nexus Standard) is a proof-of-concept bi-modal serialization format. The text format (`.nxs`) is a sigil-typed human-readable source that a Rust compiler transforms into a zero-copy binary format (`.nxb`). Ten language implementations (Rust, JavaScript, Python, Go, Ruby, PHP, C, Swift, Kotlin, C#) can read the binary directly via memory mapping without a full parse pass.

## Commands

The `Makefile` is the primary interface for lint, test, conformance, and fixture generation across all ten languages. Run from the repo root.

```bash
make fixtures                  # generate js/fixtures/ (FIXTURE_COUNT=1000 default; use 1000000 for benchmarks)
make test                      # run all ten language test suites
make lint                      # strict lint for all languages
make fix                       # auto-fix all fixable issues
make conformance               # generate conformance vectors + run all runners
make fuzz                      # cargo-fuzz for 60s (requires nightly)
make all                       # fix + test + conformance
make install-git-hooks         # pre-commit hook → make lint (bypass: SKIP_HOOKS=1 git commit)
```

Per-language test targets follow the pattern `make test-<lang>` (e.g. `make test-rust`, `make test-js`). CI-specific targets (`make test-rust-ci`, `make test-py-ci`, `make test-ruby-ci`, `make test-php-ci`) also build and verify the C extensions.

### Rust (compiler + core)
```bash
cd rust && cargo build --release          # build all binaries
cd rust && cargo test --release           # run all Rust tests
cd rust && cargo test test_compile_basic  # run a single test by name
cd rust && cargo test --test e2e --test exit_codes --test json_import  # converter tests
./rust/target/release/nxs data.nxs       # compile .nxs → .nxb
cd rust && cargo +nightly fuzz run fuzz_decode -- -max_total_time=60
```

### Converter binaries (`rust/src/bin/`)
Three additional binaries handle format conversion:
```bash
echo '[{"id":1}]' | ./rust/target/release/nxs-import --from json - out.nxb
./rust/target/release/nxs-inspect out.nxb
./rust/target/release/nxs-export --to json --pretty out.nxb
```
Supported import formats: `json`, `csv`, `xml`. Export formats: `json`, `csv`.

### Per-language quick reference
| Language | Test | Lint/Fix |
|----------|------|----------|
| JavaScript | `node js/test.js` | `cd js && npm run lint` |
| Python | `cd py && python test_nxs.py` | `ruff check` / `ruff --fix` |
| Go | `cd go && go test ./...` | `gofmt -w .` |
| Ruby | `ruby ruby/test.rb js/fixtures` | `rubocop --config ruby/.rubocop.yml` |
| PHP | `php php/test.php js/fixtures` | `phpstan analyse Nxs.php --level=5` |
| C | `cd c && make test -s && ./test ../js/fixtures` | `cppcheck` |
| Swift | `cd swift && swift run nxs-test ../js/fixtures` | `swiftlint` |
| Kotlin | `cd kotlin && ./gradlew run --args="../js/fixtures"` | `./gradlew ktlintCheck` |
| C# | `cd csharp && dotnet run -- ../js/fixtures` | `dotnet format --verify-no-changes` |

C extensions (Python, Ruby, PHP) each have a `build_ext.sh` / `ruby/ext/build.sh` / `php/nxs_ext/build.sh` and separate bench/test scripts suffixed `_c`.

Browser WASM demo requires `cd js && python3 server.py` (COOP/COEP headers for SharedArrayBuffer).

## Architecture

### Rust Crate Layout (`rust/`)

The crate lives under `rust/` (not `src/`). It provides three binaries (`nxs`, `bench`, `gen_fixtures`) plus converter binaries under `rust/src/bin/`:

- `src/lexer.rs` — tokenizes `.nxs` source into sigils, keys, braces, brackets
- `src/parser.rs` — builds an AST of `Field { key, value }`; `Value` is an enum over all sigil types
- `src/compiler.rs` — two-pass: `collect_keys` builds a global dictionary (key → u16); second pass emits preamble, schema header, LEB128 bitmask object headers, and tail-index
- `src/writer.rs` — `NxsWriter` / `Schema` / `Slot` API for emitting `.nxb` directly without text; the hot path used by `gen_fixtures`
- `src/decoder.rs` — minimal decoder used by tests; reads schema header back out of `.nxb`
- `src/error.rs` — `NxsError` enum mapping to spec error codes
- `src/convert/` — format-conversion pipeline (`json_in`, `csv_in`, `xml_in`, `json_out`, `csv_out`, `infer`, `inspect`)
- `src/bin/` — `nxs_import.rs`, `nxs_export.rs`, `nxs_inspect.rs`
- `fuzz/` — two fuzz targets: `fuzz_decode` and `fuzz_writer_roundtrip`

### Binary Format Invariants

Every implementation must honour these constraints from the spec (`SPEC.md`):

| Constraint | Detail |
|:---|:---|
| Rule of 8 | All atomic values aligned to 8-byte boundaries; padding after Bools (1 + 7 pad) and after variable-length blobs |
| File magic | Bytes 0–3: `0x4E585342` ("NXSB"); final 4 bytes: `0x2153584E` ("NXS!") |
| Tail-index | Last 8 bytes hold `FooterPtr` + `NXS!`; index entries are `(KeyID u16, AbsoluteOffset u64)` |
| Object header | `0x4E58534F` ("NXSO") magic + 4-byte length + LEB128 bitmask + u16 offset table |
| List header | `0x4E58534C` ("NXSL") magic + uniform element sigil enforced |
| Schema hash | `DictHash` in preamble is MurmurHash3 of the schema header bytes |

### Cross-Language Reader Pattern

All ten readers share the same lookup strategy:
1. Seek to `EOF - 8`, read `FooterPtr` to find the tail-index
2. Walk the tail-index to locate a record's absolute offset — O(1) random access
3. Parse the object header (bitmask + offset table) to locate individual fields — no full-record scan needed

The "slot" optimisation (resolve key name → index once, reuse per record) is available in all implementations for hot-path column scans.

### Conformance Suite (`conformance/`)

`conformance/` contains language-agnostic test vectors (`.nxb` + `.expected.json` pairs) and per-language runner scripts (`run_<lang>.<ext>`). Vectors are generated by `rust/src/bin/gen_conformance`. Each runner exits 0 on full pass, 1 on any failure. Positive vectors assert decoded field values; negative vectors assert the correct error code (`ERR_BAD_MAGIC`, `ERR_DICT_MISMATCH`, `ERR_OUT_OF_BOUNDS`).

### Fixtures

`js/fixtures/` is the shared fixture directory used by all language benchmarks. Generate before benchmarking:

```bash
make fixtures FIXTURE_COUNT=1000000
```

## Key Documents

| Document | Purpose |
|:---|:---|
| `SPEC.md` | Ground-truth binary format specification (authoritative) |
| `RFC.md` | Motivation, security guidance, implementation notes |
| `GETTING_STARTED.md` | Code examples for all languages |
| `BENCHMARK.md` | Full benchmark results across all scenarios |
| `SCENARIOS.md` | Use-case scenarios and design rationale |
| `CONTRIBUTING.md` | Requirements for new language implementations |
| `examples/` | Sample `.nxs` files used by Rust integration tests |
