# Rust Game Engine

A modular, data-oriented game engine written in Rust.

## Phase 1 foundation

- engine-core: monotonic frame timing, fixed/variable update orchestration, bounded catch-up, cross-thread events, and structured tracing.
- engine-input: keyboard, mouse, cursor, wheel, and action/axis mapping over winit.
- engine-renderer: wgpu 30 surface backend, resize handling, and a clear pass.
- sandbox: native winit application wiring the systems together.

The remaining crates establish documented workspace boundaries. Their production implementations are staged for later phases rather than represented by placeholder systems.

## Layout

~~~text
.
├── apps/sandbox
└── crates
    ├── engine-assets
    ├── engine-audio
    ├── engine-concurrency
    ├── engine-core
    ├── engine-ecs
    ├── engine-editor
    ├── engine-input
    ├── engine-math
    ├── engine-memory
    ├── engine-networking
    ├── engine-physics
    ├── engine-renderer
    └── engine-scripting
~~~

## Requirements

- Rust stable 1.88 or newer.
- A GPU driver supporting an enabled wgpu backend.
- On Linux, winit X11/Wayland development libraries.

## Run

From the repository root:

~~~bash
cargo run --bin sandbox
~~~

Development checks:

~~~bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets
~~~

PowerShell logging:

~~~powershell
$env:RUST_LOG = "debug"
cargo run --bin sandbox
~~~

## Design notes

- Rust 2024 workspace with centralized dependency versions.
- Safe Rust by default; unsafe_code is forbidden at workspace level.
- Simulation timing uses Duration; float seconds are exposed only at API boundaries.
- Fixed simulation updates are independent from variable-rate presentation updates.
- A configurable catch-up budget prevents the spiral of death and accounts for discarded time.
- The renderer passes an owned Arc window into wgpu, so its surface safely owns the native handle source.
- Phase 1 clears the swapchain. Mesh pipelines, shaders, and render-graph passes are subsequent work.

## Planned phases

1. Foundation — workspace, timing, window/input, initial wgpu surface. (Current)
2. Math, memory, concurrency, ECS — coordinate conventions, storage/query layout, scheduling, profiling.
3. Renderer — render graph, resource lifetime, meshes/materials, PBR, shadows, post-processing, compute.
4. Physics and audio — simulation integration, spatial audio, effects.
5. Assets and scripting — VFS, async loading, cooked formats, hot reload, gameplay scripts.
6. Networking — transport, replication, prediction, interpolation.
7. Editor — scene authoring, inspector, viewport, asset browser, profiling.

Dual licensed under MIT or Apache-2.0.
