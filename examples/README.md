# Examples Guide

Use this map to pick the right example quickly.

Run examples with:
- `cargo run --example <example_name>`
- async examples: `cargo run --example <example_name> --features async`

## Start Here

- `canonical_app_skeleton/`: canonical project shape (`core/`, `driving/`, `driven/`).
- `sync_basic_in_memory.rs`: minimal sync app with write + read interactions.

## Common Patterns

- `sync_owned_vs_borrowed_input.rs`: same app using owned and borrowed driving inputs.
- `sync_multithread_arc_mutex.rs`: thread-safe wiring with `Arc<Mutex<...>>` and separate read interaction.
- `sync_driven_file_persistence.rs`: driven adapter backed by file persistence.
- `sync_composition_root_trait_objects.rs`: composition root with trait-object driven dependencies.

## Advanced Flows

- `async_basic_email_flow.rs` (feature `async`): minimal async boundary usage.
- `async_retry_timeout_policy.rs` (feature `async`): retries, timeout policy, and error mapping.
- `sync_outbox_event_emission.rs`: state change plus outbound event emission.
