# NXS — C Reader

Zero-copy `.nxb` reader in C99. No dependencies beyond `libc`/`libm`.

## Build & Test

```bash
make test        # compile test binary
./test ../js/fixtures

make bench       # compile benchmark binary
./bench ../js/fixtures
```

## API

```c
#include "nxs.h"

uint8_t *data = ...;   // mmap or malloc+read
size_t   size = ...;

nxs_reader_t r;
nxs_open(&r, data, size);

// Schema
printf("%d records, %d keys\n", r.record_count, r.key_count);

// O(1) record access
nxs_object_t obj;
nxs_record(&r, 42, &obj);

int64_t  id;     nxs_get_i64 (&obj, "id",       &id);
double   score;  nxs_get_f64 (&obj, "score",     &score);
int      active; nxs_get_bool(&obj, "active",    &active);
char     uname[64]; nxs_get_str(&obj, "username", uname, sizeof(uname));

// Slot optimisation — resolve key once, reuse per record
int slot = nxs_slot(&r, "score");
nxs_get_f64_slot(&obj, slot, &score);

// Bulk reducers
double  sum = nxs_sum_f64(&r, "score");
int64_t ids = nxs_sum_i64(&r, "id");
double  mn, mx;
nxs_min_f64(&r, "score", &mn);
nxs_max_f64(&r, "score", &mx);

nxs_close(&r);
```
