# NXS — Kotlin Reader

Zero-copy `.nxb` reader for Kotlin/JVM. Uses only `java.nio.ByteBuffer` plus
`org.json` for the test JSON parsing.

## Requirements

- JDK 17+
- Gradle 8+ (`gradle wrapper` or system Gradle)

## Build & Test

```bash
cd kotlin
gradle run --args="../js/fixtures"    # smoke tests
```

## API

```kotlin
import nxs.NxsReader
import nxs.NxsError

val data = File("data.nxb").readBytes()
val reader = NxsReader(data)

println(reader.recordCount)   // Int
println(reader.keys)          // List<String>

val obj = reader.record(42)
val id:     Long    = obj.getI64("id")
val score:  Double  = obj.getF64("score")
val active: Boolean = obj.getBool("active")
val name:   String  = obj.getStr("username")

// Slot optimisation
val scoreSlot = reader.slot("score")
val s: Double = obj.getF64BySlot(scoreSlot)

// Bulk reducers
val sum:  Double  = reader.sumF64("score")
val sumi: Long    = reader.sumI64("id")
val mn:   Double? = reader.minF64("score")
val mx:   Double? = reader.maxF64("score")
```
