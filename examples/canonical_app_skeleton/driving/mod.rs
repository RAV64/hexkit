//! Driving adapters that translate transport-specific requests into core interactions.

use crate::core::{CoreError, CreateUser, FindUser, UserCore, UserId, UserName};
use crate::driven::InMemoryUsers;
use hexkit::{Handle, HandleMut};

pub struct HttpCreateUser {
    pub display_name: String,
}

pub struct CliCreateUser {
    pub name: String,
}

pub struct HttpGetUser<'i> {
    pub id: &'i UserId,
}

pub struct CliGetUser<'i> {
    pub id: &'i UserId,
}

pub struct App {
    core: UserCore<InMemoryUsers>,
}

impl App {
    pub const fn new(core: UserCore<InMemoryUsers>) -> Self {
        Self { core }
    }
}

impl HandleMut<HttpCreateUser> for App {
    type Output<'a> = Result<UserId, CoreError>;

    fn handle_mut(&mut self, input: HttpCreateUser) -> Self::Output<'_> {
        self.core.handle_mut(CreateUser {
            name: UserName(input.display_name),
        })
    }
}

impl HandleMut<CliCreateUser> for App {
    type Output<'a> = Result<UserId, CoreError>;

    fn handle_mut(&mut self, input: CliCreateUser) -> Self::Output<'_> {
        self.core.handle_mut(CreateUser {
            name: UserName(input.name),
        })
    }
}

impl Handle<HttpGetUser<'_>> for App {
    type Output<'a> = Result<Option<&'a UserName>, CoreError>;

    fn handle(&self, input: HttpGetUser<'_>) -> Self::Output<'_> {
        self.core.handle(FindUser { id: input.id })
    }
}

impl Handle<CliGetUser<'_>> for App {
    type Output<'a> = Result<Option<&'a UserName>, CoreError>;

    fn handle(&self, input: CliGetUser<'_>) -> Self::Output<'_> {
        self.core.handle(FindUser { id: input.id })
    }
}
