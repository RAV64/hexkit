//! Purpose: reference folder layout for a hexagonal application (`core/`, `driving/`, `driven/`).

mod core;
mod driven;
mod driving;

use crate::core::UserCore;
use crate::core::UserId;
use crate::driven::InMemoryUsers;
use crate::driving::App;
use crate::driving::CliCreateUser;
use crate::driving::CliGetUser;
use crate::driving::HttpCreateUser;
use crate::driving::HttpGetUser;
use hexkit::{Handle, HandleMut};

fn main() {
    let users = InMemoryUsers::default();
    let core = UserCore::new(users);
    let mut app = App::new(core);

    let http_id: UserId = app
        .handle_mut(HttpCreateUser {
            display_name: String::from("lea"),
        })
        .expect("http create should succeed");

    let cli_id: UserId = app
        .handle_mut(CliCreateUser {
            name: String::from("sam"),
        })
        .expect("cli create should succeed");

    let http_name = app
        .handle(HttpGetUser { id: &http_id })
        .expect("http read should succeed")
        .expect("http user should exist");
    let cli_name = app
        .handle(CliGetUser { id: &cli_id })
        .expect("cli read should succeed")
        .expect("cli user should exist");

    println!(
        "canonical-app-skeleton: first={} ({}) second={} ({})",
        http_id.0, http_name.0, cli_id.0, cli_name.0
    );
}
