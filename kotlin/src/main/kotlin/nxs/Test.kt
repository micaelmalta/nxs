// NXS Kotlin reader smoke tests
// Run: gradle run --args="<fixtures_dir>"
package nxs

import java.io.File
import kotlin.math.abs

fun main(args: Array<String>) {
    val dir = args.firstOrNull() ?: "../js/fixtures"
    val nxbFile  = File("$dir/records_1000.nxb")
    val jsonFile = File("$dir/records_1000.json")

    if (!nxbFile.exists()) {
        println("fixtures not found at $dir")
        println("generate them: cargo run --release --bin gen_fixtures -- js/fixtures")
        return
    }

    val nxbData  = nxbFile.readBytes()
    @Suppress("UNCHECKED_CAST")
    val jsonList = org.json.JSONArray(jsonFile.readText()).let { arr ->
        (0 until arr.length()).map { arr.getJSONObject(it) }
    }

    var passed = 0; var failed = 0

    fun check(name: String, expr: Boolean) {
        if (expr) { println("  ✓ $name"); passed++ }
        else      { println("  ✗ $name"); failed++ }
    }

    println("\nNXS Kotlin Reader — Tests\n")

    val r = NxsReader(nxbData)
    check("opens without error", true)
    check("reads correct record count", r.recordCount == 1000)
    check("reads schema keys",
        r.keys.containsAll(listOf("id", "username", "email", "score", "active")))

    val obj0 = r.record(0)
    check("record(0) id matches JSON",
        obj0.getI64("id") == jsonList[0].getLong("id"))

    val obj42 = r.record(42)
    check("record(42) username matches JSON",
        obj42.getStr("username") == jsonList[42].getString("username"))

    val obj500 = r.record(500)
    check("record(500) score close to JSON",
        abs(obj500.getF64("score") - jsonList[500].getDouble("score")) < 0.001)

    val obj999 = r.record(999)
    check("record(999) active matches JSON",
        obj999.getBool("active") == jsonList[999].getBoolean("active"))

    var threw = false
    try { r.record(10000) } catch (e: NxsError) { threw = true }
    check("out-of-bounds throws NxsError", threw)

    val sumNXS  = r.sumF64("score")
    val sumJSON = jsonList.sumOf { it.getDouble("score") }
    check("sum_f64 matches JSON sum", abs(sumNXS - sumJSON) < 0.01)

    check("sum_i64(id) positive", r.sumI64("id") > 0)

    val mn = r.minF64("score"); val mx = r.maxF64("score")
    check("min_f64 <= max_f64", mn != null && mx != null && mn <= mx)

    println("\n$passed passed, $failed failed\n")
    if (failed > 0) System.exit(1)
}
