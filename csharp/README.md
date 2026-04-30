# NXS — C# Reader

Zero-copy `.nxb` reader for C# (.NET 8). Uses only BCL types; no NuGet
dependencies.

## Requirements

- .NET 8 SDK (`dotnet` CLI)

## Build & Test

```bash
cd csharp
dotnet run -- ../js/fixtures          # smoke tests
dotnet run -c Release -- ../js/fixtures ../js/fixtures  # pass dir twice to also bench
```

## API

```csharp
using Nxs;

byte[] data = File.ReadAllBytes("data.nxb");
var reader  = new NxsReader(data);

Console.WriteLine(reader.RecordCount);   // int
Console.WriteLine(string.Join(", ", reader.Keys));

var obj = reader.Record(42);
long   id     = obj.GetI64("id");
double score  = obj.GetF64("score");
bool   active = obj.GetBool("active");
string name   = obj.GetStr("username");

// Slot optimisation
int    scoreSlot = reader.Slot("score");
double s         = obj.GetF64BySlot(scoreSlot);

// Bulk reducers
double  sum  = reader.SumF64("score");
long    sumi = reader.SumI64("id");
double? mn   = reader.MinF64("score");
double? mx   = reader.MaxF64("score");
```
