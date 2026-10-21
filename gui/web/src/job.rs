//! The one deew process of the server, its replay buffer and the fan-out of
//! its output to all connected sockets.

use axum::body::Bytes;
use deew_gui_core::cli;
use deew_gui_core::pty_session::{self, PtySession, SpawnConfig};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc;

pub const REPLAY_CAPACITY: usize = 1024 * 1024;
/// Messages a socket may lag behind before it is disconnected.
pub const SUBSCRIBER_QUEUE: usize = 256;

/// Keeps the last `cap` bytes that were pushed.
pub struct RingBuffer {
    cap: usize,
    data: VecDeque<u8>,
}

impl RingBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            data: VecDeque::new(),
        }
    }

    pub fn push(&mut self, bytes: &[u8]) {
        if bytes.len() >= self.cap {
            self.data.clear();
            self.data.extend(&bytes[bytes.len() - self.cap..]);
            return;
        }
        let overflow = (self.data.len() + bytes.len()).saturating_sub(self.cap);
        self.data.drain(..overflow);
        self.data.extend(bytes);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn snapshot(&self) -> Vec<u8> {
        let (a, b) = self.data.as_slices();
        [a, b].concat()
    }
}

/// What subscribers receive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Output(Bytes),
    Started(Vec<String>),
    Exit(Option<i32>),
}

struct Subscriber {
    id: u64,
    tx: mpsc::Sender<Event>,
}

struct Inner {
    /// Id of the current (or last) job; callbacks of older jobs are ignored.
    job_id: u64,
    session: Option<PtySession>,
    running: bool,
    args: Option<Vec<String>>,
    buffer: RingBuffer,
    subscribers: Vec<Subscriber>,
}

impl Inner {
    /// Never blocks: a subscriber whose queue is full (or closed) is dropped.
    fn broadcast(&mut self, event: Event) {
        self.subscribers
            .retain(|s| s.tx.try_send(event.clone()).is_ok());
    }
}

fn lock(inner: &Mutex<Inner>) -> MutexGuard<'_, Inner> {
    inner.lock().unwrap_or_else(|e| e.into_inner())
}

/// State to send to a newly attached socket, plus its event stream.
pub struct Subscription {
    pub id: u64,
    pub running: bool,
    pub args: Option<Vec<String>>,
    /// Buffered output so far.
    pub replay: Vec<u8>,
    pub events: mpsc::Receiver<Event>,
}

pub struct Job {
    inner: Arc<Mutex<Inner>>,
    /// Program to run; `None` means the deew CLI next to this executable.
    program: Option<PathBuf>,
    next_id: AtomicU64,
}

impl Job {
    pub fn new(program: Option<PathBuf>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                job_id: 0,
                session: None,
                running: false,
                args: None,
                buffer: RingBuffer::new(REPLAY_CAPACITY),
                subscribers: Vec::new(),
            })),
            program,
            next_id: AtomicU64::new(1),
        }
    }

    pub fn is_running(&self) -> bool {
        lock(&self.inner).running
    }

    /// Takes the buffer snapshot and subscribes under one lock, so no byte is
    /// lost or duplicated between the two.
    pub fn subscribe(&self) -> Subscription {
        let (tx, events) = mpsc::channel(SUBSCRIBER_QUEUE);
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut inner = lock(&self.inner);
        inner.subscribers.push(Subscriber { id, tx });
        Subscription {
            id,
            running: inner.running,
            args: inner.args.clone(),
            // Only a running job is replayed; a finished one is just history.
            replay: if inner.running {
                inner.buffer.snapshot()
            } else {
                Vec::new()
            },
            events,
        }
    }

    pub fn unsubscribe(&self, id: u64) {
        lock(&self.inner).subscribers.retain(|s| s.id != id);
    }

    pub fn start(&self, args: Vec<String>, cols: u16, rows: u16) -> Result<(), String> {
        let program = match &self.program {
            Some(program) => program.clone(),
            None => cli::sidecar_path()?,
        };
        let mut inner = lock(&self.inner);
        if inner.running {
            return Err("deew is already running".into());
        }
        let job_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let config = SpawnConfig {
            program,
            args: args.clone(),
            cols,
            rows,
            cwd: std::env::home_dir(),
            env: vec![("COLORTERM".into(), "truecolor".into())],
        };
        let out_inner = Arc::clone(&self.inner);
        let exit_inner = Arc::clone(&self.inner);
        // The callbacks run on other threads and wait for this lock, so they
        // cannot observe the state before it is complete.
        let session = pty_session::spawn(
            config,
            move |bytes| {
                let mut inner = lock(&out_inner);
                if inner.job_id == job_id {
                    inner.buffer.push(bytes);
                    inner.broadcast(Event::Output(Bytes::copy_from_slice(bytes)));
                }
            },
            move |code| {
                let mut inner = lock(&exit_inner);
                if inner.job_id == job_id {
                    inner.session = None;
                    inner.running = false;
                    inner.broadcast(Event::Exit(Some(code)));
                }
            },
        )?;
        inner.job_id = job_id;
        inner.session = Some(session);
        inner.running = true;
        inner.args = Some(args.clone());
        inner.buffer.clear();
        inner.broadcast(Event::Started(args));
        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        match lock(&self.inner).session.as_mut() {
            Some(session) => session.write(data),
            None => Ok(()),
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        match lock(&self.inner).session.as_ref() {
            Some(session) => session.resize(cols, rows),
            None => Ok(()),
        }
    }

    /// Ctrl+C, then kill after a grace period (see `PtySession::stop`).
    pub fn stop(&self) {
        if let Some(session) = lock(&self.inner).session.as_mut() {
            session.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_keeps_the_tail() {
        let mut b = RingBuffer::new(5);
        assert!(b.is_empty());
        b.push(b"ab");
        b.push(b"cd");
        assert_eq!(b.snapshot(), b"abcd");
        b.push(b"efg");
        assert_eq!(b.snapshot(), b"cdefg");
        assert_eq!(b.len(), 5);
        b.push(b"0123456789");
        assert_eq!(b.snapshot(), b"56789");
        b.push(b"");
        assert_eq!(b.snapshot(), b"56789");
        b.clear();
        assert!(b.is_empty());
        b.push(b"x");
        assert_eq!(b.snapshot(), b"x");
    }

    #[test]
    fn ring_buffer_wraps_many_times() {
        let mut b = RingBuffer::new(7);
        let mut all: Vec<u8> = Vec::new();
        for i in 0..100u8 {
            let chunk = vec![i; (i as usize % 4) + 1];
            all.extend(&chunk);
            b.push(&chunk);
        }
        assert_eq!(b.snapshot(), all[all.len() - 7..]);
    }

    #[test]
    fn slow_subscriber_is_dropped_without_blocking() {
        let job = Job::new(None);
        let mut slow = job.subscribe();
        let mut fast = job.subscribe();
        let fast_id = fast.id;
        {
            let mut inner = lock(&job.inner);
            for i in 0..(SUBSCRIBER_QUEUE + 10) {
                inner.broadcast(Event::Output(Bytes::from(vec![i as u8])));
                // keep `fast` drained
                while fast.events.try_recv().is_ok() {}
            }
            assert_eq!(inner.subscribers.len(), 1);
            assert_eq!(inner.subscribers[0].id, fast_id);
        }
        let mut n = 0;
        while slow.events.try_recv().is_ok() {
            n += 1;
        }
        assert_eq!(n, SUBSCRIBER_QUEUE);
    }
}
