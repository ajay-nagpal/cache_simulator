A Rust-based program designed to simulate the behavior of cache memory. The simulator models fundamental cache operations: hits, misses, and evictions.

What is cache memory?
A small but very fast storage area that sits between the CPU and the main memory (RAM). The cache reduces this waiting time by keeping recently used data close to the processor. Programs often reuse the same data again (temporal locality) or use data stored near the data they just accessed (spatial locality). The cache takes advantage of these patterns to improve performance.

This structure represents the entire cache. It stores an array of Set
structures along with key parameters including:
s: the number of set index bits
b: the number of block offset bits
E: the number of lines per set (associativity)
hits, miss, evicts: counters for cache hits, misses, and evictions

Assumptions:

Consider all three levels of cache memory as a single unit.
Tracefile will always contain memory accesses in valid format
operation address, size. example :"M 7ff000384,4"

Valid flags : `-s <s> -E <E> -b <b> -t <tracefile>`

The trace files may contain several additional fields, but we are only using memory addresses.
The values of s, E, and b are assumed to be valid. Example: s,E and b cannot be zero, also s+b should be less than 64
We will focus only on data operations.
