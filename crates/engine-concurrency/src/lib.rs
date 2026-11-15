#![forbid(unsafe_code)]
//! Parallel execution primitives and synchronization policy. The planned job system uses work-stealing for independent tasks and low-contention synchronization at subsystem boundaries.
