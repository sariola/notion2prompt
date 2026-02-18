//! State management algebras for recursive fetching.
//!
//! These traits provide capabilities for managing the state of recursive
//! content traversal, including visit tracking for cycle detection and
//! depth limiting for safety.

use crate::types::NotionId;
use async_trait::async_trait;

use super::error::TrackError;

/// Visit tracking capability for preventing cycles during recursive traversal.
///
/// # Laws
///
/// All implementations must satisfy these laws:
///
/// - **L1 (Idempotency)**: Visiting the same ID twice returns `false` on the second visit.
///   ```text
///   visit(id) == Ok(true)
///   visit(id) == Ok(false)
///   ```
///
/// - **L2 (Is-Visited Consistency)**: `is_visited(id)` reflects the result of `visit(id)`.
///   ```text
///   visit(id) == Ok(first_result)
///   is_visited(id) == first_result
///   ```
///
/// - **L3 (Persistence)**: Once visited, an ID remains visited.
///   ```text
///   visit(id) == Ok(true)
///   // ... any number of other operations ...
///   is_visited(id) == true
///   ```
///
/// This trait is **object-safe** and can be used as `dyn VisitTracker`.
#[async_trait]
pub trait VisitTracker: Send + Sync {
    /// Mark an ID as visited and return whether this was the first visit.
    ///
    /// Returns `Ok(true)` if this is the first visit to this ID,
    /// `Ok(false)` if the ID was already visited, or an error.
    async fn visit(&self, id: &NotionId) -> Result<bool, TrackError>;

    /// Check if an ID has been visited without marking it.
    async fn is_visited(&self, id: &NotionId) -> bool;
}

/// Depth limiting capability for controlling recursion depth.
///
/// # Laws
///
/// All implementations must satisfy these laws:
///
/// - **L1 (Monotonic Decrease)**: Depth only decreases with each level.
///   ```text
///   depth() == d
///   enter() -> d'
///   d' <= d
///   ```
///
/// - **L2 (Can-Enter Consistency)**: `can_enter()` reflects whether depth > 0.
///   ```text
///   can_enter() == (depth() > 0)
///   ```
///
/// - **L3 (Zero Termination)**: When depth reaches 0, `can_enter()` is false.
///   ```text
///   enter() repeated until depth() == 0
///   can_enter() == false
///   ```
///
/// This trait is **object-safe** and can be used as `dyn DepthLimiter`.
#[async_trait]
pub trait DepthLimiter: Send + Sync {
    /// Get the current remaining depth.
    async fn depth(&self) -> u8;

    /// Check if we can enter another level of recursion.
    async fn can_enter(&self) -> bool;

    /// Enter a new level, returning the new depth limit.
    ///
    /// Returns `None` if depth would go negative (i.e., limit exceeded).
    async fn enter(&self) -> Option<u8>;

    /// Exit a level, restoring the previous depth.
    async fn exit(&self);
}

// ==============================================================================
// Extension Trait for VisitTracker
// ==============================================================================

/// Extension trait with typed convenience methods for [`VisitTracker`].
///
/// This trait is **NOT object-safe** due to generic methods but provides
/// a blanket implementation for all `VisitTracker` types.
#[async_trait]
#[allow(dead_code)]
pub trait VisitTrackerExt: VisitTracker {
    /// Visit multiple IDs and return how many were first-time visits.
    async fn visit_many(&self, ids: &[NotionId]) -> Result<usize, TrackError> {
        let mut first_visits = 0;
        for id in ids {
            if self.visit(id).await? {
                first_visits += 1;
            }
        }
        Ok(first_visits)
    }

    /// Visit an ID only if not already visited; returns the visit result.
    async fn visit_if_new(&self, id: &NotionId) -> Result<bool, TrackError> {
        if self.is_visited(id).await {
            Ok(false)
        } else {
            self.visit(id).await
        }
    }
}

// Blanket implementation for all VisitTracker
#[async_trait]
impl<T: VisitTracker + ?Sized> VisitTrackerExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Simple in-memory visit tracker for testing laws.
    struct InMemoryVisitTracker {
        visited: Arc<RwLock<HashSet<String>>>,
    }

    impl InMemoryVisitTracker {
        fn new() -> Self {
            Self {
                visited: Arc::new(RwLock::new(HashSet::new())),
            }
        }
    }

    #[async_trait]
    impl VisitTracker for InMemoryVisitTracker {
        async fn visit(&self, id: &NotionId) -> Result<bool, TrackError> {
            let mut visited = self.visited.write().await;
            Ok(visited.insert(id.as_str().to_string()))
        }

        async fn is_visited(&self, id: &NotionId) -> bool {
            let visited = self.visited.read().await;
            visited.contains(id.as_str())
        }
    }

    /// Simple in-memory depth limiter for testing laws.
    struct InMemoryDepthLimiter {
        depth: Arc<RwLock<u8>>,
    }

    impl InMemoryDepthLimiter {
        fn new(initial: u8) -> Self {
            Self {
                depth: Arc::new(RwLock::new(initial)),
            }
        }
    }

    #[async_trait]
    impl DepthLimiter for InMemoryDepthLimiter {
        async fn depth(&self) -> u8 {
            *self.depth.read().await
        }

        async fn can_enter(&self) -> bool {
            *self.depth.read().await > 0
        }

        async fn enter(&self) -> Option<u8> {
            let mut depth = self.depth.write().await;
            if *depth == 0 {
                None
            } else {
                *depth -= 1;
                Some(*depth)
            }
        }

        async fn exit(&self) {
            let mut depth = self.depth.write().await;
            *depth = depth.saturating_add(1);
        }
    }

    // ========================================================================
    // VisitTracker Law Tests
    // ========================================================================

    #[tokio::test]
    async fn law_l1_idempotency() {
        let tracker = InMemoryVisitTracker::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        // First visit should return true (first time)
        assert!(tracker.visit(&id).await.unwrap());

        // Second visit should return false (already visited)
        assert!(!tracker.visit(&id).await.unwrap());
    }

    #[tokio::test]
    async fn law_l2_is_visited_consistency() {
        let tracker = InMemoryVisitTracker::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        // Before visit
        assert!(!tracker.is_visited(&id).await);

        // Visit returns true (first time)
        let visit_result = tracker.visit(&id).await.unwrap();
        assert!(visit_result);

        // is_visited should now return true
        assert!(tracker.is_visited(&id).await);

        // Second visit returns false
        assert!(!tracker.visit(&id).await.unwrap());

        // is_visited still returns true
        assert!(tracker.is_visited(&id).await);
    }

    #[tokio::test]
    async fn law_l3_persistence() {
        let tracker = InMemoryVisitTracker::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();
        let other = NotionId::parse("550e8400e29b41d4a716446655440001").unwrap();

        // Visit ID
        tracker.visit(&id).await.unwrap();

        // Visit other IDs
        tracker.visit(&other).await.unwrap();
        tracker.visit(&other).await.unwrap();

        // Original ID should still be visited
        assert!(tracker.is_visited(&id).await);
    }

    // ========================================================================
    // DepthLimiter Law Tests
    // ========================================================================

    #[tokio::test]
    async fn law_l1_monotonic_decrease() {
        let limiter = InMemoryDepthLimiter::new(5);

        // Initial depth
        let d0 = limiter.depth().await;
        assert_eq!(d0, 5);

        // Enter - depth decreases
        let d1 = limiter.enter().await.unwrap();
        assert!(d1 <= d0);

        let d2 = limiter.depth().await;
        assert_eq!(d1, d2);
    }

    #[tokio::test]
    async fn law_l2_can_enter_consistency() {
        let limiter = InMemoryDepthLimiter::new(3);

        // Positive depth means we can enter
        assert_eq!(limiter.depth().await, 3);
        assert!(limiter.can_enter().await);

        // At depth 1, we can still enter one more time
        while limiter.depth().await > 1 {
            limiter.enter().await.unwrap();
        }
        assert_eq!(limiter.depth().await, 1);
        assert!(limiter.can_enter().await);

        // After entering, depth is 0
        limiter.enter().await.unwrap();
        assert_eq!(limiter.depth().await, 0);
        assert!(!limiter.can_enter().await);
    }

    #[tokio::test]
    async fn law_l3_zero_termination() {
        let limiter = InMemoryDepthLimiter::new(2);

        assert!(limiter.can_enter().await);

        limiter.enter().await.unwrap();
        assert_eq!(limiter.depth().await, 1);
        assert!(limiter.can_enter().await);

        limiter.enter().await.unwrap();
        assert_eq!(limiter.depth().await, 0);
        assert!(!limiter.can_enter().await);

        // Further attempts return None
        assert_eq!(limiter.enter().await, None);
    }

    // ========================================================================
    // Extension Trait Tests
    // ========================================================================

    #[tokio::test]
    async fn visit_many_counts_first_visits() {
        let tracker = InMemoryVisitTracker::new();
        let id1 = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();
        let id2 = NotionId::parse("550e8400e29b41d4a716446655440001").unwrap();
        let id3 = NotionId::parse("550e8400e29b41d4a716446655440002").unwrap();

        // First time - all new
        let count = tracker
            .visit_many(&[id1.clone(), id2.clone(), id3.clone()])
            .await
            .unwrap();
        assert_eq!(count, 3);

        // Second time - none new
        let count = tracker
            .visit_many(&[id1.clone(), id2.clone(), id3.clone()])
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn visit_if_new_only_visits_unvisited() {
        let tracker = InMemoryVisitTracker::new();
        let id = NotionId::parse("550e8400e29b41d4a716446655440000").unwrap();

        // First time - should visit
        assert!(tracker.visit_if_new(&id).await.unwrap());
        assert!(tracker.is_visited(&id).await);

        // Second time - should not visit (already visited)
        assert!(!tracker.visit_if_new(&id).await.unwrap());
    }
}
