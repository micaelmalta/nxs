# NXS — Swift Reader

Zero-copy `.nxb` reader in Swift 5.9+. Uses `Foundation.Data` for memory
mapping; no third-party dependencies.

## Build & Test

Requires Swift 5.9+ (Xcode 15+ or swift.org toolchain).

```bash
swift run nxs-test ../js/fixtures     # smoke tests
swift run -c release nxs-bench ../js/fixtures   # benchmark
```

## API

```swift
import NXS

let data = try Data(contentsOf: URL(fileURLWithPath: "data.nxb"))
let reader = try NXSReader(data)

print(reader.recordCount)   // Int
print(reader.keys)          // [String]

let obj = try reader.record(42)
let id:     Int64  = try obj.getI64("id")
let score:  Double = try obj.getF64("score")
let active: Bool   = try obj.getBool("active")
let name:   String = try obj.getStr("username")

// Slot optimisation
let scoreSlot = try reader.slot("score")
let s: Double = try obj.getF64BySlot(scoreSlot)

// Bulk reducers
let sum:  Double  = try reader.sumF64("score")
let sumi: Int64   = try reader.sumI64("id")
let mn:   Double? = try reader.minF64("score")
let mx:   Double? = try reader.maxF64("score")
```
