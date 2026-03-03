//! Purpose: minimal async boundaries with separate command and query interactions.

use core::future::Future;
use core::task::{Context, Poll, Waker};
use hexkit::r#async::{Handle, HandleMut};
use std::thread;
use std::time::Duration;

struct SendEmail {
    address: &'static str,
}

struct HealthCheck;
struct ReadSentCount;

#[derive(Default)]
struct FakeEmailGateway {
    sent: usize,
}

impl HandleMut<SendEmail> for FakeEmailGateway {
    type Output<'a> = Result<(), ()>;

    async fn handle_mut(&mut self, input: SendEmail) -> Self::Output<'_> {
        let _ = input.address;
        simulated_io().await;
        self.sent += 1;
        Ok(())
    }
}

impl Handle<ReadSentCount> for FakeEmailGateway {
    type Output<'a> = usize;

    async fn handle(&self, _input: ReadSentCount) -> Self::Output<'_> {
        self.sent
    }
}

struct HealthProbe;

impl Handle<HealthCheck> for HealthProbe {
    type Output<'a> = &'static str;

    async fn handle(&self, _input: HealthCheck) -> Self::Output<'_> {
        simulated_io().await;
        "ok"
    }
}

struct AsyncDeps {
    gateway: FakeEmailGateway,
    probe: HealthProbe,
}

impl Handle<HealthCheck> for AsyncDeps {
    type Output<'a> = &'static str;

    async fn handle(&self, input: HealthCheck) -> Self::Output<'_> {
        self.probe.handle(input).await
    }
}

impl HandleMut<SendEmail> for AsyncDeps {
    type Output<'a> = Result<(), ()>;

    async fn handle_mut(&mut self, input: SendEmail) -> Self::Output<'_> {
        self.gateway.handle_mut(input).await
    }
}

impl Handle<ReadSentCount> for AsyncDeps {
    type Output<'a> = usize;

    async fn handle(&self, input: ReadSentCount) -> Self::Output<'_> {
        self.gateway.handle(input).await
    }
}

async fn simulated_io() {
    core::future::ready(()).await;
    thread::sleep(Duration::from_millis(10));
}

fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let mut future = std::pin::pin!(future);

    loop {
        match Future::poll(future.as_mut(), &mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::yield_now(),
        }
    }
}

fn main() {
    let mut app = AsyncDeps {
        gateway: FakeEmailGateway::default(),
        probe: HealthProbe,
    };

    let health = block_on(app.handle(HealthCheck));
    block_on(app.handle_mut(SendEmail {
        address: "team@example.com",
    }))
    .expect("email send should succeed");
    let sent = block_on(app.handle(ReadSentCount));

    println!("async-basic-email-flow: health={health} sent_count={sent}");
}
