//! Core domain interactions and core-level driven trait definitions.

use hexkit::{Handle, HandleMut};

#[derive(Debug, PartialEq, Eq)]
pub struct UserId(pub u64);

#[derive(Debug, PartialEq, Eq)]
pub struct UserName(pub String);

pub struct CreateUser {
    pub name: UserName,
}

pub struct FindUser<'i> {
    pub id: &'i UserId,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CoreError {
    EmptyName,
    Duplicate,
}

pub trait UserStore {
    fn create(&mut self, input: CreateUser) -> Result<UserId, CoreError>;
    fn find<'i>(&'i self, id: &UserId) -> Result<Option<&'i UserName>, CoreError>;
}

pub struct UserCore<S> {
    store: S,
}

impl<S> UserCore<S> {
    pub const fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> HandleMut<CreateUser> for UserCore<S>
where
    S: UserStore + 'static,
{
    type Output<'a> = Result<UserId, CoreError>;

    fn handle_mut(&mut self, input: CreateUser) -> Self::Output<'_> {
        if input.name.0.trim().is_empty() {
            return Err(CoreError::EmptyName);
        }

        self.store.create(input)
    }
}

impl<S> Handle<FindUser<'_>> for UserCore<S>
where
    S: UserStore + 'static,
{
    type Output<'a> = Result<Option<&'a UserName>, CoreError>;

    fn handle(&self, input: FindUser<'_>) -> Self::Output<'_> {
        self.store.find(input.id)
    }
}
