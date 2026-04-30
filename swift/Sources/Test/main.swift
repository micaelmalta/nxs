// NXS Swift reader smoke tests
// Run: swift run nxs-test <fixtures_dir>
import Foundation
import NXS

let fixtureDir = CommandLine.arguments.count > 1
    ? CommandLine.arguments[1]
    : "../js/fixtures"

var passed = 0, failed = 0

func check(_ name: String, _ expr: Bool) {
    if expr { print("  ✓ \(name)"); passed += 1 }
    else     { print("  ✗ \(name)"); failed += 1 }
}
func checkThrows(_ name: String, _ body: () throws -> Void) {
    do { try body(); print("  ✗ \(name) — expected throw"); failed += 1 }
    catch { print("  ✓ \(name)"); passed += 1 }
}

print("\nNXS Swift Reader — Tests\n")

let nxbURL  = URL(fileURLWithPath: "\(fixtureDir)/records_1000.nxb")
let jsonURL = URL(fileURLWithPath: "\(fixtureDir)/records_1000.json")

guard let nxbData = try? Data(contentsOf: nxbURL) else {
    print("fixtures not found at \(fixtureDir)")
    print("generate them: cargo run --release --bin gen_fixtures -- js/fixtures")
    exit(1)
}
let jsonData = try! Data(contentsOf: jsonURL)
let json = try! JSONSerialization.jsonObject(with: jsonData) as! [[String: Any]]

do {
    let r = try NXSReader(nxbData)
    check("opens without error", true)
    check("reads correct record count", r.recordCount == 1000)
    check("reads schema keys", r.keys.contains("id") && r.keys.contains("username") && r.keys.contains("score"))

    let obj0 = try r.record(0)
    let id0 = try obj0.getI64("id")
    check("record(0) id matches JSON", id0 == (json[0]["id"] as! NSNumber).int64Value)

    let obj42 = try r.record(42)
    let u42 = try obj42.getStr("username")
    check("record(42) username matches JSON", u42 == (json[42]["username"] as! String))

    let obj500 = try r.record(500)
    let s500 = try obj500.getF64("score")
    let js500 = (json[500]["score"] as! NSNumber).doubleValue
    check("record(500) score close to JSON", abs(s500 - js500) < 0.001)

    let obj999 = try r.record(999)
    let a999 = try obj999.getBool("active")
    check("record(999) active matches JSON", a999 == (json[999]["active"] as! Bool))

    checkThrows("out-of-bounds record throws") { _ = try r.record(10000) }

    let sumNXS = try r.sumF64("score")
    let sumJSON = json.reduce(0.0) { $0 + ($1["score"] as! NSNumber).doubleValue }
    check("sum_f64 matches JSON sum", abs(sumNXS - sumJSON) < 0.01)

    let sumId = try r.sumI64("id")
    check("sum_i64(id) positive", sumId > 0)

    if let mn = try r.minF64("score"), let mx = try r.maxF64("score") {
        check("min_f64 <= max_f64", mn <= mx)
    }

} catch {
    print("  ✗ fatal: \(error)")
    failed += 1
}

print("\n\(passed) passed, \(failed) failed\n")
exit(failed > 0 ? 1 : 0)
