//! Purpose: async retry/timeout policy with transport-error to app-error mapping.

use core::future::Future;
use core::task::{Context, Poll, Waker};
use hexkit::r#async::HandleMut;
use std::thread;
use std::time::{Duration, Instant};

struct SendWelcome {
    address: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
enum MailGatewayError {
    Timeout,
    Transport,
}

#[derive(Debug, PartialEq, Eq)]
enum AppError {
    GatewayTimeout,
    DeliveryUnavailable,
}

trait Clock {
    fn now(&self) -> Instant;
}

struct StdClock;

impl Clock for StdClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

struct FlakyGateway {
    attempts: usize,
    fail_until: usize,
}

impl FlakyGateway {
    const fn new(fail_until: usize) -> Self {
        Self {
            attempts: 0,
            fail_until,
        }
    }
}

impl HandleMut<SendWelcome> for FlakyGateway {
    type Output<'a> = Result<(), MailGatewayError>;

    async fn handle_mut(&mut self, input: SendWelcome) -> Self::Output<'_> {
        let _ = input.address;
        self.attempts += 1;
        simulated_io().await;

        if self.attempts.is_multiple_of(5) {
            return Err(MailGatewayError::Timeout);
        }

        if self.attempts <= self.fail_until {
            return Err(MailGatewayError::Transport);
        }

        Ok(())
    }
}

struct NotificationCore<G, C> {
    gateway: G,
    clock: C,
    max_retries: usize,
    timeout: Duration,
}

impl<C> NotificationCore<FlakyGateway, C> {
    const fn new(gateway: FlakyGateway, clock: C, max_retries: usize, timeout: Duration) -> Self {
        Self {
            gateway,
            clock,
            max_retries,
            timeout,
        }
    }
}

impl<C: Clock + 'static> HandleMut<SendWelcome> for NotificationCore<FlakyGateway, C> {
    type Output<'a> = Result<(), AppError>;

    async fn handle_mut(&mut self, input: SendWelcome) -> Self::Output<'_> {
        let started = self.clock.now();

        for _ in 0..=self.max_retries {
            if started.elapsed() > self.timeout {
                return Err(AppError::GatewayTimeout);
            }

            let attempt = self.gateway.handle_mut(SendWelcome {
                address: input.address,
            });

            match attempt.await {
                Ok(()) => return Ok(()),
                Err(MailGatewayError::Timeout) => return Err(AppError::GatewayTimeout),
                Err(MailGatewayError::Transport) => {
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }

        Err(AppError::DeliveryUnavailable)
    }
}

async fn simulated_io() {
    core::future::ready(()).await;
    thread::sleep(Duration::from_millis(3));
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
    let gateway = FlakyGateway::new(2);
    let clock = StdClock;
    let mut app = NotificationCore::new(gateway, clock, 3, Duration::from_millis(200));

    let first = block_on(app.handle_mut(SendWelcome {
        address: "lea@example.com",
    }));
    assert_eq!(first, Ok(()));

    let bad_gateway = FlakyGateway::new(10);
    let mut failing = NotificationCore::new(bad_gateway, StdClock, 1, Duration::from_millis(40));
    let second = block_on(failing.handle_mut(SendWelcome {
        address: "sam@example.com",
    }));
    assert_eq!(second, Err(AppError::DeliveryUnavailable));

    println!("async-retry-timeout-policy: first={first:?} second={second:?}");
}
