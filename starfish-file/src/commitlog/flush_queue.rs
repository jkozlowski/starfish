// TODO: Add gates
use crate::spawn;
use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use futures::channel::oneshot::Sender;
use futures::future::Shared;
use futures::FutureExt;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::future::Future;
use std::iter::DoubleEndedIterator;
use std::ops::Bound;

struct Notifier {
    sender: Sender<()>,
    receiver: Shared<Receiver<()>>,
    count: usize,
}

impl Notifier {
    pub fn new() -> Self {
        let (sender, receiver) = oneshot::channel();
        Notifier {
            sender,
            receiver: receiver.shared(),
            count: 0,
        }
    }
}

/// Keeps an ordered queue of pending operations. Allows flushes for various chunks
/// to complete in arbitrary order while making sure that callbacks for mutations
/// at higher position run only after all the lower position mutations are finished.
pub struct FlushQueue<T> {
    map: BTreeMap<T, Notifier>,
}

impl<T: Ord + Copy + Debug> FlushQueue<T> {
    pub fn new() -> Self {
        FlushQueue {
            map: BTreeMap::new(),
        }
    }

    pub async fn run_with_ordered_post_op<F>(&mut self, t: T, action: F, post: F)
    where
        F: Future<Output = ()> + 'static,
    {
        // Check that all elements are lower than what we're inserting or it contains the key already
        if !self.map.is_empty()
            && self
                .map
                .range((Bound::Excluded(t), Bound::Unbounded))
                .next()
                .is_none()
        {
            panic!("Attempting to insert key out of order: {:?}", t);
        }

        {
            let entry = self.map.entry(t).or_insert_with(Notifier::new);
            entry.count += 1;
        }

        // Run the action
        action.await;

        let mut iter = self.map.range_mut(t..);
        let me = iter.next().unwrap();

        if let Some(prev) = iter.next_back().filter(|prev_value| *prev_value.0 < t) {
            // If there is a key before us, wait until that is finished before running our post
            prev.1.receiver.clone().await.map_err(|e| {}).unwrap();
        }
        // Now is the right time to run post
        post.await;

        me.1.count -= 1;

        if me.1.count == 0 {
            self.map.remove(&t);
            // if (f.failed() && _chain_exceptions) {
            //     return handle_failed_future(std::move(f), pr);
            // } else {
            //     pr.set_value();
            // }
        }
    }
}
