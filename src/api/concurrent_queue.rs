// src/api/concurrent_queue.rs
//! Concurrent work-stealing queue implementation for parallel fetching.

use super::fetch_queue::{ExplorationStep, PrioritizedWorkItem, StepOutcome};
use crossbeam::deque::{Injector, Stealer, Worker};
use parking_lot::Mutex;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Tracks work queue completion state
#[derive(Debug)]
struct WorkTracker {
    /// Number of work items queued
    pending_work: AtomicUsize,
    /// Number of work items completed
    completed_work: AtomicUsize,
}

impl WorkTracker {
    fn new() -> Self {
        Self {
            pending_work: AtomicUsize::new(0),
            completed_work: AtomicUsize::new(0),
        }
    }

    fn add_pending(&self, count: usize) {
        self.pending_work.fetch_add(count, Ordering::SeqCst);
    }

    fn mark_completed(&self) {
        self.completed_work.fetch_add(1, Ordering::SeqCst);
    }

    fn has_pending_work(&self) -> bool {
        let pending = self.pending_work.load(Ordering::SeqCst);
        let completed = self.completed_work.load(Ordering::SeqCst);
        pending > completed
    }
}

/// Thread-safe work queue with work-stealing support.
pub struct ConcurrentWorkQueue {
    /// Global injector for new work items
    injector: Arc<Injector<PrioritizedWorkItem>>,
    /// Stealers for work-stealing between workers
    stealers: Vec<Stealer<PrioritizedWorkItem>>,
    /// Results collector
    results: Arc<Mutex<Vec<StepOutcome>>>,
    /// Work completion tracker
    work_tracker: Arc<WorkTracker>,
    /// Sequence counter for maintaining FIFO within priorities
    sequence_counter: Arc<AtomicUsize>,
}

impl ConcurrentWorkQueue {
    /// Creates a new concurrent work queue with the specified number of workers.
    pub fn new(num_workers: usize) -> (Self, Vec<WorkerQueue>) {
        let injector = Arc::new(Injector::new());
        let mut workers = Vec::with_capacity(num_workers);
        let mut stealers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(WorkerQueue {
                worker,
                injector: Arc::clone(&injector),
                work_tracker: None,     // Will be set later
                sequence_counter: None, // Will be set later
                priority_buffer: Arc::new(Mutex::new(BinaryHeap::new())),
            });
        }

        let work_tracker = Arc::new(WorkTracker::new());
        let sequence_counter = Arc::new(AtomicUsize::new(0));

        let queue = Self {
            injector,
            stealers,
            results: Arc::new(Mutex::new(Vec::new())),
            work_tracker: work_tracker.clone(),
            sequence_counter: sequence_counter.clone(),
        };

        // Update workers to include work tracker and sequence counter
        let workers_with_tracker = workers
            .into_iter()
            .map(|mut w| {
                w.work_tracker = Some(work_tracker.clone());
                w.sequence_counter = Some(sequence_counter.clone());
                w
            })
            .collect();

        (queue, workers_with_tracker)
    }

    /// Enqueues a work item to the global queue.
    pub fn enqueue(&self, item: ExplorationStep) {
        self.work_tracker.add_pending(1);
        let sequence = self.sequence_counter.fetch_add(1, Ordering::SeqCst);
        let prioritized = PrioritizedWorkItem {
            priority: item.priority(),
            sequence,
            item,
        };
        self.injector.push(prioritized);
    }

    /// Enqueues multiple work items.
    pub fn enqueue_multiple(&self, items: Vec<ExplorationStep>) {
        let count = items.len();
        self.work_tracker.add_pending(count);

        // Sort by priority first
        let mut prioritized_items: Vec<_> = items
            .into_iter()
            .map(|item| {
                let sequence = self.sequence_counter.fetch_add(1, Ordering::SeqCst);
                PrioritizedWorkItem {
                    priority: item.priority(),
                    sequence,
                    item,
                }
            })
            .collect();

        // Sort by priority (highest first) to ensure critical items are processed first
        prioritized_items.sort_by(|a, b| b.cmp(a));

        for item in prioritized_items {
            self.injector.push(item);
        }
    }

    /// Checks if there is any pending work.
    pub fn has_pending_work(&self) -> bool {
        self.work_tracker.has_pending_work()
    }

    /// Marks a work item as completed.
    pub fn mark_completed(&self) {
        self.work_tracker.mark_completed();
    }

    /// Stores a result in the thread-safe results collector.
    pub fn store_result(&self, result: StepOutcome) {
        self.results.lock().push(result);
    }

    /// Collects all results.
    pub fn collect_results(self) -> Vec<StepOutcome> {
        Arc::try_unwrap(self.results)
            .map(|mutex| mutex.into_inner())
            .unwrap_or_else(|arc| arc.lock().clone())
    }

    /// Creates a stealer for work-stealing.
    pub fn stealers(&self) -> &[Stealer<PrioritizedWorkItem>] {
        &self.stealers
    }
}

/// Per-worker queue with work-stealing capabilities.
pub struct WorkerQueue {
    /// Local work queue
    worker: Worker<PrioritizedWorkItem>,
    /// Reference to global injector
    injector: Arc<Injector<PrioritizedWorkItem>>,
    /// Work tracker for completion detection
    work_tracker: Option<Arc<WorkTracker>>,
    /// Sequence counter for priority ordering
    sequence_counter: Option<Arc<AtomicUsize>>,
    /// Local priority heap for sorting stolen work
    priority_buffer: Arc<Mutex<BinaryHeap<PrioritizedWorkItem>>>,
}

impl WorkerQueue {
    /// Dequeues a work item, trying local queue first, then stealing.
    pub fn dequeue(&self, stealers: &[Stealer<PrioritizedWorkItem>]) -> Option<ExplorationStep> {
        // First, check if we have items in the priority buffer
        if let Some(mut buffer) = self.priority_buffer.try_lock() {
            if let Some(prioritized) = buffer.pop() {
                return Some(prioritized.item);
            }
        }

        // Try local queue first
        if let Some(prioritized) = self.worker.pop() {
            return Some(prioritized.item);
        }

        // Try stealing from global injector - steal batch for efficiency
        let mut stolen_items = Vec::new();
        if let crossbeam::deque::Steal::Success(item) =
            self.injector.steal_batch_and_pop(&self.worker)
        {
            stolen_items.push(item);
            // Continue stealing while we can
            while let crossbeam::deque::Steal::Success(item) = self.injector.steal() {
                stolen_items.push(item);
                if stolen_items.len() >= 8 {
                    break; // Don't steal too many at once
                }
            }
        }

        // Also try stealing from other workers
        for stealer in stealers {
            if let crossbeam::deque::Steal::Success(item) = stealer.steal() {
                stolen_items.push(item);
            }
        }

        if stolen_items.is_empty() {
            return None;
        }

        // Sort stolen items by priority and put all but the highest priority in the buffer
        stolen_items.sort_by(|a, b| b.cmp(a)); // Highest priority first

        // Split off the first item
        if let Some(first) = stolen_items.pop() {
            // Put the rest in the buffer
            if !stolen_items.is_empty() {
                let mut buffer = self.priority_buffer.lock();
                for item in stolen_items {
                    buffer.push(item);
                }
            }
            Some(first.item)
        } else {
            None
        }
    }
}
