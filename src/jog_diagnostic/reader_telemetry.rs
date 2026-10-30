use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Get1,
    Get3,
    InterruptRead,
    Get2Analog,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationTiming {
    pub operation: Operation,
    pub latest_started_us: Option<u64>,
    pub latest_ended_us: Option<u64>,
    pub latest_duration_us: Option<u64>,
    pub maximum_duration_us: u64,
    pub count: u64,
    pub failures: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub elapsed_us: u64,
    pub in_flight_operation: Option<Operation>,
    pub in_flight_started_us: Option<u64>,
    pub in_flight_wait: Option<InFlightWait>,
    pub waits: WaitTiming,
    pub operations: [OperationTiming; 4],
    pub latest_batch_duration_us: Option<u64>,
    pub maximum_batch_duration_us: u64,
    pub latest_publication_duration_us: Option<u64>,
    pub maximum_publication_duration_us: u64,
    pub last_publication_us: Option<u64>,
    pub publication_count: u64,
    pub queue_depth: usize,
    pub queue_high_water: usize,
    pub latest_poll_lateness_us: Option<u64>,
    pub maximum_poll_lateness_us: u64,
    pub latest_post_wait_us: Option<u64>,
    pub maximum_post_wait_us: u64,
    pub post_wait_count: u64,
    pub latest_poll_admitted_us: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct InFlightWait {
    pub requested: Duration,
    pub scheduled_deadline_us: u64,
    pub started_us: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct WaitTiming {
    pub latest_requested_us: Option<u64>,
    pub latest_scheduled_deadline_us: Option<u64>,
    pub latest_started_us: Option<u64>,
    pub latest_returned_us: Option<u64>,
    pub latest_elapsed_us: Option<u64>,
    pub latest_overshoot_us: Option<u64>,
    pub latest_cancelled: Option<bool>,
    pub maximum_elapsed_us: u64,
    pub maximum_overshoot_us: u64,
    pub count: u64,
    pub cancellations: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum PollEvent {
    WaitStarted {
        requested_us: u64,
        scheduled_deadline_us: u64,
        started_us: u64,
    },
    WaitReturned {
        returned_us: u64,
        cancelled: bool,
    },
    PollAdmitted {
        started_us: u64,
        lateness_us: u64,
        post_wait_us: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug)]
struct InFlight {
    operation: Operation,
    started_us: u64,
}

#[derive(Debug)]
pub struct Telemetry {
    started: Instant,
    in_flight: Option<InFlight>,
    in_flight_wait: Option<InFlightWait>,
    waits: WaitTiming,
    operations: [OperationTiming; 4],
    latest_batch_duration_us: Option<u64>,
    maximum_batch_duration_us: u64,
    latest_publication_duration_us: Option<u64>,
    maximum_publication_duration_us: u64,
    last_publication_us: Option<u64>,
    publication_count: u64,
    queue_high_water: usize,
    latest_poll_lateness_us: Option<u64>,
    maximum_poll_lateness_us: u64,
    latest_post_wait_us: Option<u64>,
    maximum_post_wait_us: u64,
    post_wait_count: u64,
    latest_poll_admitted_us: Option<u64>,
}

impl Telemetry {
    pub fn new(started: Instant) -> Self {
        Self {
            started,
            in_flight: None,
            in_flight_wait: None,
            waits: WaitTiming::default(),
            operations: [
                Operation::Get1,
                Operation::Get3,
                Operation::InterruptRead,
                Operation::Get2Analog,
            ]
            .map(|operation| OperationTiming {
                operation,
                latest_started_us: None,
                latest_ended_us: None,
                latest_duration_us: None,
                maximum_duration_us: 0,
                count: 0,
                failures: 0,
            }),
            latest_batch_duration_us: None,
            maximum_batch_duration_us: 0,
            latest_publication_duration_us: None,
            maximum_publication_duration_us: 0,
            last_publication_us: None,
            publication_count: 0,
            queue_high_water: 0,
            latest_poll_lateness_us: None,
            maximum_poll_lateness_us: 0,
            latest_post_wait_us: None,
            maximum_post_wait_us: 0,
            post_wait_count: 0,
            latest_poll_admitted_us: None,
        }
    }

    pub fn elapsed_us(&self) -> u64 {
        micros(self.started.elapsed())
    }

    pub fn begin(&mut self, operation: Operation) {
        self.in_flight = Some(InFlight {
            operation,
            started_us: self.elapsed_us(),
        });
    }

    pub fn end(&mut self, operation: Operation, duration: Duration, failed: bool) {
        let duration_us = micros(duration);
        let started_us = self
            .in_flight
            .filter(|flight| flight.operation == operation)
            .map(|flight| flight.started_us);
        let ended_us = self.elapsed_us();
        let timing = &mut self.operations[index(operation)];
        timing.latest_started_us = started_us;
        timing.latest_ended_us = Some(ended_us);
        timing.latest_duration_us = Some(duration_us);
        timing.maximum_duration_us = timing.maximum_duration_us.max(duration_us);
        timing.count = timing.count.saturating_add(1);
        if failed {
            timing.failures = timing.failures.saturating_add(1);
        }
        self.in_flight = None;
    }

    pub fn batch_finished(&mut self, duration: Duration) {
        let duration_us = micros(duration);
        self.latest_batch_duration_us = Some(duration_us);
        self.maximum_batch_duration_us = self.maximum_batch_duration_us.max(duration_us);
    }

    pub fn publication_finished(&mut self, duration: Duration, queue_depth: usize) {
        let duration_us = micros(duration);
        self.latest_publication_duration_us = Some(duration_us);
        self.maximum_publication_duration_us =
            self.maximum_publication_duration_us.max(duration_us);
        self.last_publication_us = Some(self.elapsed_us());
        self.publication_count = self.publication_count.saturating_add(1);
        self.queue_high_water = self.queue_high_water.max(queue_depth);
    }

    pub fn poll_lateness(&mut self, lateness: Duration) {
        let lateness_us = micros(lateness);
        self.latest_poll_lateness_us = Some(lateness_us);
        self.maximum_poll_lateness_us = self.maximum_poll_lateness_us.max(lateness_us);
    }

    pub fn poll_event(&mut self, event: PollEvent) {
        match event {
            PollEvent::WaitStarted {
                requested_us,
                scheduled_deadline_us,
                started_us,
            } => {
                self.in_flight_wait = Some(InFlightWait {
                    requested: Duration::from_micros(requested_us),
                    scheduled_deadline_us,
                    started_us,
                });
            }
            PollEvent::WaitReturned {
                returned_us,
                cancelled,
            } => {
                if let Some(wait) = self.in_flight_wait.take() {
                    let elapsed_us = returned_us.saturating_sub(wait.started_us);
                    let overshoot_us = returned_us.saturating_sub(wait.scheduled_deadline_us);
                    self.waits.latest_requested_us = Some(micros(wait.requested));
                    self.waits.latest_scheduled_deadline_us = Some(wait.scheduled_deadline_us);
                    self.waits.latest_started_us = Some(wait.started_us);
                    self.waits.latest_returned_us = Some(returned_us);
                    self.waits.latest_elapsed_us = Some(elapsed_us);
                    self.waits.latest_overshoot_us = Some(overshoot_us);
                    self.waits.latest_cancelled = Some(cancelled);
                    self.waits.maximum_elapsed_us = self.waits.maximum_elapsed_us.max(elapsed_us);
                    self.waits.maximum_overshoot_us =
                        self.waits.maximum_overshoot_us.max(overshoot_us);
                    self.waits.count = self.waits.count.saturating_add(1);
                    if cancelled {
                        self.waits.cancellations = self.waits.cancellations.saturating_add(1);
                    }
                }
            }
            PollEvent::PollAdmitted {
                started_us,
                lateness_us: _,
                post_wait_us: Some(post_wait_us),
            } => {
                self.latest_poll_admitted_us = Some(started_us);
                self.latest_post_wait_us = Some(post_wait_us);
                self.maximum_post_wait_us = self.maximum_post_wait_us.max(post_wait_us);
                self.post_wait_count = self.post_wait_count.saturating_add(1);
            }
            PollEvent::PollAdmitted {
                started_us,
                post_wait_us: None,
                ..
            } => self.latest_poll_admitted_us = Some(started_us),
        }
    }

    pub fn snapshot(&mut self, queue_depth: usize) -> Snapshot {
        self.queue_high_water = self.queue_high_water.max(queue_depth);
        Snapshot {
            elapsed_us: self.elapsed_us(),
            in_flight_operation: self.in_flight.map(|flight| flight.operation),
            in_flight_started_us: self.in_flight.map(|flight| flight.started_us),
            in_flight_wait: self.in_flight_wait,
            waits: self.waits.clone(),
            operations: self.operations.clone(),
            latest_batch_duration_us: self.latest_batch_duration_us,
            maximum_batch_duration_us: self.maximum_batch_duration_us,
            latest_publication_duration_us: self.latest_publication_duration_us,
            maximum_publication_duration_us: self.maximum_publication_duration_us,
            last_publication_us: self.last_publication_us,
            publication_count: self.publication_count,
            queue_depth,
            queue_high_water: self.queue_high_water,
            latest_poll_lateness_us: self.latest_poll_lateness_us,
            maximum_poll_lateness_us: self.maximum_poll_lateness_us,
            latest_post_wait_us: self.latest_post_wait_us,
            maximum_post_wait_us: self.maximum_post_wait_us,
            post_wait_count: self.post_wait_count,
            latest_poll_admitted_us: self.latest_poll_admitted_us,
        }
    }
}

const fn index(operation: Operation) -> usize {
    match operation {
        Operation::Get1 => 0,
        Operation::Get3 => 1,
        Operation::InterruptRead => 2,
        Operation::Get2Analog => 3,
    }
}

fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

#[cfg(test)]
#[path = "reader_telemetry_tests.rs"]
mod tests;
