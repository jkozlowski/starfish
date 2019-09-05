// TODO: Add gates
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
#[derive(Default)]
pub struct FlushQueue<T: Ord + Copy + Debug> {
    map: BTreeMap<T, Notifier>,
}

impl<T: Ord + Copy + Debug> FlushQueue<T> {
    pub fn new() -> Self {
        FlushQueue {
            map: BTreeMap::new(),
        }
    }

    pub async fn run_with_ordered_post_op<F, P>(&mut self, t: T, action: F, post: P)
    where
        F: Future<Output = ()> + 'static,
        P: Future<Output = ()> + 'static,
    {
        // Check that all elements are lower than what we're inserting or it contains the key already
        if self
            .map
            .range((Bound::Excluded(t), Bound::Unbounded))
            .next()
            .is_some()
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
            prev.1.receiver.clone().await;
        }
        // Now is the right time to run post
        post.await;

        me.1.count -= 1;

        if me.1.count == 0 {
            let notifier_again = self.map.remove(&t).unwrap();
            notifier_again.sender.send(()).unwrap()
        }
    }

    // Waits for all operations currently active to finish
    pub async fn wait_for_all(&self) {
        if !self.map.is_empty() {
            return self
                .wait_for_pending(self.map.keys().next().unwrap().clone())
                .await;
        }
    }

    // Waits for all operations whose key is less than or equal to "rp"
    // to complete
    pub async fn wait_for_pending(&self, t: T) {
        if let Some(notifier) = self
            .map
            .range((Bound::Excluded(t), Bound::Unbounded))
            .rev()
            .next()
        {
            notifier.1.receiver.clone().await;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::spawn;
    use crate::Shared;
    use futures::channel::oneshot::Receiver;
    use futures::channel::oneshot::Sender;
    use hamcrest2::assert_that;
    use hamcrest2::prelude::*;
    use rand::prelude::*;
    use rand::thread_rng;
    use rand::Rng;
    use std::mem;

    #[tokio::test]
    pub async fn test_run_with_ordered_post_op() {
        let num_ops = 1000;

        struct Env {
            queue: FlushQueue<usize>,
            promises: Vec<(Option<Sender<()>>, Receiver<()>)>,
            result: Vec<usize>,
        }

        impl Env {
            fn create(num: usize) -> Env {
                let vec: Vec<usize> = (0..num).collect();
                Env {
                    queue: FlushQueue::new(),
                    promises: vec
                        .iter()
                        .map(|_| {
                            let (sender, receiver) = oneshot::channel();
                            (Some(sender), receiver)
                        })
                        .collect(),
                    result: vec![],
                }
            }
        }

        async fn run_single_op(i: usize, env: Shared<Env>) {}

        let mut env = Shared::new(Env::create(num_ops));

        for i in 0..num_ops {
            spawn(run_single_op(i, env.clone()))
        }

        fn shuffled(num_ops: usize) -> Vec<usize> {
            let mut vec: Vec<usize> = (0..num_ops).collect();
            vec.shuffle(&mut thread_rng());
            vec
        }

        for i in shuffled(num_ops) {
            let p = &mut env.borrow_mut().promises[i];
            let sender = mem::replace(&mut p.0, None);
            sender.unwrap().send(()).unwrap();
        }

        // Wait for all to finish
        let env_borrow = env.borrow();
        env_borrow.queue.wait_for_all().await;

        assert_that!(env_borrow.result.is_empty(), is(false));
    }
}
