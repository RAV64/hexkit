//! Driven adapter implementation (in-memory store).

use crate::core::{CoreError, CreateUser, UserId, UserName, UserStore};

#[derive(Default)]
pub struct InMemoryUsers {
    next_id: u64,
    rows: Vec<(u64, UserName)>,
}

impl UserStore for InMemoryUsers {
    fn create(&mut self, input: CreateUser) -> Result<UserId, CoreError> {
        if self
            .rows
            .iter()
            .any(|(_, existing)| existing.0 == input.name.0)
        {
            return Err(CoreError::Duplicate);
        }

        self.next_id += 1;
        self.rows.push((self.next_id, input.name));
        Ok(UserId(self.next_id))
    }

    fn find<'i>(&'i self, id: &UserId) -> Result<Option<&'i UserName>, CoreError> {
        Ok(self
            .rows
            .iter()
            .find(|(row_id, _)| *row_id == id.0)
            .map(|(_, name)| name))
    }
}
