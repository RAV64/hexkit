//! Sync boundary traits.

/// Immutable boundary interaction.
pub trait Handle<I> {
    /// Response shape for this handler implementation.
    type Output<'a>
    where
        Self: 'a;

    /// Handles an input using immutable access.
    fn handle(&self, input: I) -> Self::Output<'_>;
}

/// Mutable boundary interaction.
pub trait HandleMut<I> {
    /// Response shape for this handler implementation.
    type Output<'a>
    where
        Self: 'a;

    /// Handles an input using mutable access.
    fn handle_mut(&mut self, input: I) -> Self::Output<'_>;
}
