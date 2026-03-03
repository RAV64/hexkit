//! Purpose: sync multi-threaded access using `Arc<Mutex<...>>` with separate read/write interactions.

use hexkit::{Handle, HandleMut};
use std::sync::{Arc, Mutex};
use std::thread;

struct AddPoints {
    points: i64,
}

struct ReadTotal;

#[derive(Default)]
struct Ledger {
    total: i64,
}

impl HandleMut<AddPoints> for Ledger {
    type Output<'a> = i64;

    fn handle_mut(&mut self, input: AddPoints) -> Self::Output<'_> {
        self.total += input.points;
        self.total
    }
}

impl Handle<ReadTotal> for Ledger {
    type Output<'a> = i64;

    fn handle(&self, _input: ReadTotal) -> Self::Output<'_> {
        self.total
    }
}

fn main() {
    let app = Arc::new(Mutex::new(Ledger::default()));

    let mut handles = Vec::new();
    for _ in 0..4 {
        let app = Arc::clone(&app);
        handles.push(thread::spawn(move || {
            for _ in 0..25 {
                app.lock()
                    .expect("app lock should succeed")
                    .handle_mut(AddPoints { points: 1 });
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread should join successfully");
    }

    let total = app
        .lock()
        .expect("app lock should remain available")
        .handle(ReadTotal);
    println!("sync-multithread-arc-mutex: total points={total}");
}
