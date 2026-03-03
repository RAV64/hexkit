#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Boundary traits for hexagonal architecture.
//!
//! `hexkit` is intentionally small:
//! - `Handle<I>` for read interactions
//! - `HandleMut<I>` for write interactions
//! - optional async variants in `hexkit::r#async` (`async` feature)
//!
//! This crate provides only boundary traits.
//! Define your own interaction types and adapter structs in your application.
//!
//! # Hello World
//!
//! ```rust
//! use hexkit::{Handle, HandleMut};
//!
//! struct CreateUser {
//!     name: String,
//! }
//!
//! struct ReadUser {
//!     id: u64,
//! }
//!
//! #[derive(Default)]
//! struct UserCore {
//!     next_id: u64,
//!     rows: Vec<(u64, String)>,
//! }
//!
//! impl HandleMut<CreateUser> for UserCore {
//!     type Output<'a> = u64;
//!
//!     fn handle_mut(&mut self, input: CreateUser) -> Self::Output<'_> {
//!         self.next_id += 1;
//!         self.rows.push((self.next_id, input.name));
//!         self.next_id
//!     }
//! }
//!
//! impl Handle<ReadUser> for UserCore {
//!     type Output<'a> = Option<&'a str>;
//!
//!     fn handle(&self, input: ReadUser) -> Self::Output<'_> {
//!         self.rows
//!             .iter()
//!             .find(|(id, _)| *id == input.id)
//!             .map(|(_, name)| name.as_str())
//!     }
//! }
//!
//! let mut core = UserCore::default();
//! let id = core.handle_mut(CreateUser { name: String::from("lea") });
//! let name = core.handle(ReadUser { id }).expect("user should exist");
//! assert_eq!(name, "lea");
//! ```
//!
//! # Design Notes
//!
//! - Define `Output<'a>` once per interaction.
//! - Use an owned type for regular outputs.
//! - Use a borrowed type (for example `&'a T`) for zero-copy outputs.
//! - Keep driving adapters and driven adapters as separate structs/modules.

mod api;
#[cfg(feature = "async")]
pub mod r#async;

pub use api::{Handle, HandleMut};
