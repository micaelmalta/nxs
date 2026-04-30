/*
 * nxs_reducers.c — freestanding reducers for the NXS format.
 *
 * Compiled to WebAssembly with no libc, no allocator, no imports.
 *
 *   clang --target=wasm32 -O3 -nostdlib -fno-builtin \
 *         -Wl,--no-entry -Wl,--export-dynamic -Wl,--allow-undefined \
 *         -o nxs_reducers.wasm nxs_reducers.c
 *
 * JS calls conventions:
 *   - Buffer base address passed as a uint32 offset into WASM linear memory.
 *   - All reads are little-endian (WASM is LE natively).
 *   - Scanner walks the tail-index, dereferences each object, and walks its
 *     LEB128 bitmask inline per-record to find the slot's value.
 */

#include <stdint.h>

/* Unaligned little-endian reads — WASM loads are already LE. */
static inline uint16_t rd_u16(const uint8_t *p) {
    return (uint16_t)p[0] | ((uint16_t)p[1] << 8);
}

static inline uint32_t rd_u32(const uint8_t *p) {
    return (uint32_t)p[0]       | ((uint32_t)p[1] << 8)
         | ((uint32_t)p[2] << 16)| ((uint32_t)p[3] << 24);
}

static inline uint64_t rd_u64(const uint8_t *p) {
    return (uint64_t)rd_u32(p) | ((uint64_t)rd_u32(p + 4) << 32);
}

static inline int64_t rd_i64(const uint8_t *p) {
    return (int64_t)rd_u64(p);
}

static inline double rd_f64(const uint8_t *p) {
    union { uint64_t u; double d; } u;
    u.u = rd_u64(p);
    return u.d;
}

/*
 * Locate the byte offset of `slot`'s value within the object at `obj_offset`.
 * Returns -1 on absent. Inlines the LEB128 bitmask walk and offset-table index.
 */
static int64_t field_offset(const uint8_t *data, uint32_t size,
                            uint32_t obj_offset, uint32_t slot) {
    uint32_t p = obj_offset + 8; /* skip NXSO magic + length */
    if (p > size) return -1;

    uint32_t cur_slot = 0;
    uint32_t table_idx = 0;
    int found = 0;
    uint8_t byte = 0;
    do {
        if (p >= size) return -1;
        byte = data[p++];
        uint8_t data_bits = byte & 0x7F;
        for (int b = 0; b < 7; b++) {
            if (cur_slot == slot) {
                if ((data_bits >> b) & 1) {
                    found = 1;
                } else {
                    return -1;
                }
            } else if (cur_slot < slot && ((data_bits >> b) & 1)) {
                table_idx++;
            }
            cur_slot++;
        }
        if (found && (byte & 0x80) == 0) break;
        if (cur_slot > slot && found) break;
    } while (byte & 0x80);

    if (!found) return -1;

    /* Skip any remaining continuation bytes to reach offset-table start */
    while (byte & 0x80) {
        if (p >= size) return -1;
        byte = data[p++];
    }

    uint32_t ofpos = p + table_idx * 2;
    if (ofpos + 2 > size) return -1;
    uint16_t rel = rd_u16(data + ofpos);
    return (int64_t)obj_offset + rel;
}

/*
 * Sum all f64 values at `slot` across every record.
 *
 * base: absolute WASM-memory offset of the .nxb buffer
 * size: total byte length of that buffer
 * tail_start: absolute offset of the first tail-index record entry
 *             (= tailPtr + 4, i.e. already past EntryCount)
 * record_count: total number of records
 * slot: target field slot index
 */
__attribute__((export_name("sum_f64")))
double sum_f64(uint32_t base, uint32_t size, uint32_t tail_start,
               uint32_t record_count, uint32_t slot) {
    const uint8_t *data = (const uint8_t *)(uintptr_t)base;
    (void)size; /* bounds-checked via field_offset */
    double sum = 0.0;
    for (uint32_t i = 0; i < record_count; i++) {
        const uint8_t *entry = data + tail_start + (uint64_t)i * 10;
        uint32_t abs = (uint32_t)rd_u64(entry + 2);
        int64_t off = field_offset(data, 0xFFFFFFFFu, abs, slot);
        if (off < 0) continue;
        sum += rd_f64(data + off);
    }
    return sum;
}

__attribute__((export_name("sum_i64")))
int64_t sum_i64(uint32_t base, uint32_t size, uint32_t tail_start,
                uint32_t record_count, uint32_t slot) {
    const uint8_t *data = (const uint8_t *)(uintptr_t)base;
    (void)size;
    int64_t sum = 0;
    for (uint32_t i = 0; i < record_count; i++) {
        const uint8_t *entry = data + tail_start + (uint64_t)i * 10;
        uint32_t abs = (uint32_t)rd_u64(entry + 2);
        int64_t off = field_offset(data, 0xFFFFFFFFu, abs, slot);
        if (off < 0) continue;
        sum += rd_i64(data + off);
    }
    return sum;
}

/*
 * min_f64 / max_f64 need to signal "no records matched" to JS.
 * Convention: return 0.0 and set a module-local flag that JS retrieves via
 * `min_max_has_result()`.
 */
static int32_t _min_max_has_result = 0;

__attribute__((export_name("min_max_has_result")))
int32_t min_max_has_result(void) { return _min_max_has_result; }

__attribute__((export_name("min_f64")))
double min_f64(uint32_t base, uint32_t size, uint32_t tail_start,
               uint32_t record_count, uint32_t slot) {
    const uint8_t *data = (const uint8_t *)(uintptr_t)base;
    (void)size;
    double m = 0.0;
    int have = 0;
    for (uint32_t i = 0; i < record_count; i++) {
        const uint8_t *entry = data + tail_start + (uint64_t)i * 10;
        uint32_t abs = (uint32_t)rd_u64(entry + 2);
        int64_t off = field_offset(data, 0xFFFFFFFFu, abs, slot);
        if (off < 0) continue;
        double v = rd_f64(data + off);
        if (!have || v < m) { m = v; have = 1; }
    }
    _min_max_has_result = have;
    return m;
}

__attribute__((export_name("max_f64")))
double max_f64(uint32_t base, uint32_t size, uint32_t tail_start,
               uint32_t record_count, uint32_t slot) {
    const uint8_t *data = (const uint8_t *)(uintptr_t)base;
    (void)size;
    double m = 0.0;
    int have = 0;
    for (uint32_t i = 0; i < record_count; i++) {
        const uint8_t *entry = data + tail_start + (uint64_t)i * 10;
        uint32_t abs = (uint32_t)rd_u64(entry + 2);
        int64_t off = field_offset(data, 0xFFFFFFFFu, abs, slot);
        if (off < 0) continue;
        double v = rd_f64(data + off);
        if (!have || v > m) { m = v; have = 1; }
    }
    _min_max_has_result = have;
    return m;
}

/* Not needed: JS imports memory, chooses the data base itself. */
