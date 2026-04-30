// NXS C# reader smoke tests + optional bench
// Run: dotnet run -- <fixtures_dir>
//      dotnet run -- <fixtures_dir> --bench
using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using Nxs;

string dir = args.Length > 0 ? args[0] : "../js/fixtures";
string nxbPath  = Path.Combine(dir, "records_1000.nxb");
string jsonPath = Path.Combine(dir, "records_1000.json");

if (!File.Exists(nxbPath))
{
    Console.WriteLine($"fixtures not found at {dir}");
    Console.WriteLine("generate them: cargo run --release --bin gen_fixtures -- js/fixtures");
    return 1;
}

byte[] nxbData  = File.ReadAllBytes(nxbPath);
var    jsonArr  = JsonNode.Parse(File.ReadAllText(jsonPath))!.AsArray();

int passed = 0, failed = 0;

void Check(string name, bool expr)
{
    if (expr) { Console.WriteLine($"  ✓ {name}"); passed++; }
    else      { Console.WriteLine($"  ✗ {name}"); failed++; }
}

Console.WriteLine("\nNXS C# Reader — Tests\n");

var r = new NxsReader(nxbData);
Check("opens without error", true);
Check("reads correct record count", r.RecordCount == 1000);
Check("reads schema keys",
    Array.IndexOf(r.Keys, "id")       >= 0 &&
    Array.IndexOf(r.Keys, "username") >= 0 &&
    Array.IndexOf(r.Keys, "score")    >= 0);

var obj0 = r.Record(0);
Check("record(0) id matches JSON",
    obj0.GetI64("id") == jsonArr[0]!["id"]!.GetValue<long>());

var obj42 = r.Record(42);
Check("record(42) username matches JSON",
    obj42.GetStr("username") == jsonArr[42]!["username"]!.GetValue<string>());

var obj500 = r.Record(500);
Check("record(500) score close to JSON",
    Math.Abs(obj500.GetF64("score") - jsonArr[500]!["score"]!.GetValue<double>()) < 0.001);

var obj999 = r.Record(999);
Check("record(999) active matches JSON",
    obj999.GetBool("active") == jsonArr[999]!["active"]!.GetValue<bool>());

bool threw = false;
try { r.Record(10000); } catch (NxsException) { threw = true; }
Check("out-of-bounds throws NxsException", threw);

double sumNXS  = r.SumF64("score");
double sumJSON = 0;
foreach (var rec in jsonArr) sumJSON += rec!["score"]!.GetValue<double>();
Check("sum_f64 matches JSON sum", Math.Abs(sumNXS - sumJSON) < 0.01);

Check("sum_i64(id) positive", r.SumI64("id") > 0);

double? mn = r.MinF64("score"), mx = r.MaxF64("score");
Check("min_f64 <= max_f64", mn.HasValue && mx.HasValue && mn.Value <= mx.Value);

Console.WriteLine($"\n{passed} passed, {failed} failed\n");

if (args.Length > 1 && args[1] == "--bench")
    Bench.Run(dir);

return failed > 0 ? 1 : 0;
