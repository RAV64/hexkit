//! Async API surface.
//!
//! Available only with feature `async`.
#![allow(async_fn_in_trait)]

/// Immutable async boundary interaction.
pub trait Handle<I> {
    /// Response shape for this handler implementation.
    type Output<'a>
    where
        Self: 'a;

    /// Handles an input asynchronously using immutable access.
    async fn handle(&self, input: I) -> Self::Output<'_>;
}

/// Mutable async boundary interaction.
pub trait HandleMut<I> {
    /// Response shape for this handler implementation.
    type Output<'a>
    where
        Self: 'a;

    /// Handles an input asynchronously using mutable access.
    async fn handle_mut(&mut self, input: I) -> Self::Output<'_>;
}
