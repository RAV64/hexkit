# hexkit

Small boundary traits for hexagonal architecture in Rust.

## What This Crate Provides

- `Handle<I>`: immutable/read-style interaction boundary.
- `HandleMut<I>`: mutable/write-style interaction boundary.
- Optional async equivalents in `hexkit::r#async` behind feature flag `async`.

You define your own interaction input/output types and adapters.

## Hello World

```rust
use hexkit::{Handle, HandleMut};

struct CreateUser {
    name: String,
}

struct ReadUser {
    id: u64,
}

#[derive(Default)]
struct UserCore {
    next_id: u64,
    rows: Vec<(u64, String)>,
}

impl HandleMut<CreateUser> for UserCore {
    type Output<'a> = u64;

    fn handle_mut(&mut self, input: CreateUser) -> Self::Output<'_> {
        self.next_id += 1;
        self.rows.push((self.next_id, input.name));
        self.next_id
    }
}

impl Handle<ReadUser> for UserCore {
    type Output<'a> = Option<&'a str>;

    fn handle(&self, input: ReadUser) -> Self::Output<'_> {
        self.rows
            .iter()
            .find(|(id, _)| *id == input.id)
            .map(|(_, name)| name.as_str())
    }
}

fn main() {
    let mut core = UserCore::default();
    let id = core.handle_mut(CreateUser { name: String::from("lea") });
    let name = core.handle(ReadUser { id }).expect("user should exist");
    assert_eq!(name, "lea");
}
```

## Async

Enable the feature:

```toml
[dependencies]
hexkit = { version = "0.1", features = ["async"] }
```

Use:

```rust
use hexkit::r#async::{Handle, HandleMut};
```

## Examples

See [examples/README.md](examples/README.md) for an example map.

Quick start commands:

- `cargo run --example sync_basic_in_memory`
- `cargo run --example canonical_app_skeleton`
- `cargo run --example async_basic_email_flow --features async`
