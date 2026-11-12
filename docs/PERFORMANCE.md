# Performance measurement

SJML includes a small diagnostic microbenchmark runner for the portable
compiled-WASM core. It reports the Java version, operating system, architecture,
available processors, and median/minimum/maximum nanoseconds per operation.
Run it from the repository root:

```sh
./gradlew :sjml-core:benchmark --args="20000 7"
```

The two arguments are measured iterations per sample and sample rounds. The
runner measures uncached compile plus instantiation, an authenticated disk
compiler-cache hit plus instantiation, a numeric guest export, a scalar
WASM-to-Java import, direct versus batched numeric and memory imports (4, 16,
and 64 operations), a Java-to-WASM byte callback with a four-byte payload,
structured task submission/dispatch/completion, synchronous and asynchronous
local service routing, and payload copying at 4 B, 256 B, 4 KiB, and 64 KiB.
When available, JVM per-thread allocation counters add allocated bytes per
operation to the payload-copy results. The service cases exercise the portable
router and a deterministic in-process lane scheduler; they exclude Minecraft
networking and real executor contention.
Payload-copy cases clone the array and consume up to 64 evenly spaced bytes so
the copy remains observable to the JVM. Their throughput is a microbenchmark
measurement, not a storage or network bandwidth result.
The batch cases use a pre-encoded request in guest memory, so they measure the
Java dispatcher and exclude Rust request construction, guest allocation, and
copying. Compile measurements use one compile per sample because compilation
is much slower than a warmed guest call. The authenticated cache creates a private,
temporary key and cache under `build/tmp`, verifies that a miss writes an entry,
then verifies each measured compilation actually hits it. Temporary data is
deleted after a successful run.

These figures describe only the machine and JVM that produced them. The runner
does not use a JMH fork or control CPU frequency, thermal state, background
load, or JVM flags. For comparisons, use the same hardware, JDK distribution,
power profile, Gradle version, and arguments; close other workloads and repeat
the command several times. Report the raw output with those conditions. SJML
does not claim a speedup from these measurements.

One smoke run in the managed cloud workspace on 2026-11-10 used Temurin
21.0.13+9 on Linux/amd64 with four available processors and `1000 3` arguments.
It reported 17.9 ms median uncached compile/instantiate (16.8 ms minimum,
298.6 ms maximum), 4.9 ms median authenticated cache hit/instantiate
(4.2–16.3 ms), 0.91 µs per guest export (0.73–1.64 µs), 0.50 µs per numeric
host import (0.42–0.80 µs), and 1.32 µs per four-byte callback (1.29–2.24 µs).
The guest-call measurements include per-export diagnostic timing
instrumentation. The large compile maximum shows why this small smoke run
should not be used as a performance claim or release comparison.

A `20000 7` run on the same machine measured these four-operation groups:

| Operations | Direct imports | One batch import |
| --- | ---: | ---: |
| Numeric return-only host calls | 255 ns | 489 ns |
| Four-byte memory copy host calls | 310 ns | 620 ns |

The same run measured 16 numeric calls at 411 ns direct versus 1.39 µs batched,
16 memory copies at 1.07 µs versus 3.10 µs, 64 numeric calls at 1.39 µs versus
4.92 µs, and 64 memory copies at 3.54 µs versus 6.53 µs. In this microbenchmark
the generic batch dispatcher is slower for trivial host operations, and the
measurement omits guest-side batch encoding and allocation, so it does not
establish a general speedup. Real Minecraft reads have not been benchmarked.
Prefer an operation-specific bulk API such as `world::block_states` when one
exists; use the generic batch API only after measuring the actual workload.

An additional `1000 3` smoke run on 2026-11-11 with Temurin 21.0.13+9-LTS on
Linux/amd64 measured deterministic task-group submit/dispatch/
completion at 2.88 µs/op, local synchronous service routing at 2.05 µs/op, and
local asynchronous service routing at 3.90 µs/op for a 16-byte request. Payload
copy cases measured 58 ns/op and 24 B/op for 4 B, 203 ns/op and 272 B/op for
256 B, 322 ns/op and 4,112 B/op for 4 KiB, and 2.28 µs/op and 65,552 B/op for
64 KiB. The 64 KiB case reported 27,353 MiB/s in this short run. These values
are diagnostic; the small iteration count, non-JMH harness, JVM warmup, and
uncontrolled machine load make them unsuitable as release claims or direct
comparisons with real game workloads.

The Rust CLI integration suite also verifies reproducible package output by
building the same archive twice and comparing the complete bytes, sorted entry
order, and SHA-256 declarations:

```sh
cargo test --manifest-path sjml-cli/Cargo.toml -p sjml-cli --test cli \
  packages_are_reproducible_sorted_and_checksum_every_payload --locked
```

These deterministic task and service cases are portable microbenchmarks, not
measurements of a live server queue. There is still no benchmark for live
Minecraft tick impact, client FPS, GPU driver cost, multiplayer routing,
storage throughput, contended async queue latency, callback fan-out, or peak
process/native allocation. Those measurements require representative
dedicated-server and display-backed client runs; results must be collected
before publishing performance claims for those paths. The rendering and live
network integrations have not been exercised in those environments yet.

Administrators can inspect cumulative compiled-export timings with
`/sjml profiler`. It reports calls, total elapsed time, current linear-memory
reservation, active asynchronous operations, maximum time, and the five
highest-cost exports for each active module. Each module records aggregate
timings for at most 128 distinct export names; total call/time counters continue
to include additional names. The profiler uses `System.nanoTime` around each
guest export invocation, includes ABI allocator/deallocator exports used by byte
callbacks, and excludes Java-side work between guest calls. Counters are
instance-local and reset when that runtime instance is recreated. The memory
figure describes current WASM pages rather than peak process/native memory; the
async count covers structured tasks owned by the module. The profiler is
diagnostic and does not replace the controlled benchmark runner above.
