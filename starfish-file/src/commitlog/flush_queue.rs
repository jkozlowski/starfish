// TODO: Add gates
use crate::Shared;
use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use futures::channel::oneshot::Sender;
use futures::future::Shared as SharedFut;
use futures::FutureExt;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::future::Future;
use std::iter::DoubleEndedIterator;
use std::ops::Bound;

struct Notifier {
    sender: Sender<()>,
    receiver: SharedFut<Receiver<()>>,
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
// #[derive(Default)]
#[derive(Clone)]
pub struct FlushQueue<T: Ord + Copy + Debug> {
    map: Shared<BTreeMap<T, Notifier>>,
}

impl<T: Ord + Copy + Debug> FlushQueue<T> {
    pub fn new() -> Self {
        FlushQueue {
            map: Shared::new(BTreeMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub async fn run_with_ordered_post_op<F, P>(self, t: T, action: F, post: P)
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
            let mut map = self.map.borrow_mut();
            let entry = map.entry(t).or_insert_with(Notifier::new);
            entry.count += 1;
        }

        // Run the action
        action.await;

        let receiver = {
            let mut map = self.map.borrow_mut();
            let mut iter = map.range_mut(t..);
            let _ = iter.next().unwrap();

            if let Some(prev) = iter.next_back().filter(|prev_value| *prev_value.0 < t) {
                // If there is a key before us, wait until that is finished before running our post
                Some(prev.1.receiver.clone())
            } else {
                None
            }
        };

        // Wait for previous actions to finish
        if let Some(receiver) = receiver {
            receiver.await.unwrap();
        }

        // Now is the right time to run post
        post.await;

        let mut map = self.map.borrow_mut();
        let mut iter = map.range_mut(t..);
        let me = iter.next().unwrap();
        me.1.count -= 1;

        if me.1.count == 0 {
            let notifier_again = map.remove(&t).unwrap();
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
            notifier.1.receiver.clone().await.unwrap();
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
    use futures::future::RemoteHandle;
    use futures::FutureExt;
    use hamcrest2::assert_that;
    use hamcrest2::prelude::*;
    use is_sorted::IsSorted;
    use rand::prelude::*;
    use rand::thread_rng;
    use rand::Rng;
    use std::mem;
    use std::time::Duration;
    use tokio_timer::sleep;

    #[tokio::test]
    pub async fn test_run_with_ordered_post_op() {
        let num_ops = 1000;
        let expected_result: Vec<usize> = (0..num_ops).collect();

        struct Pipe {
            sender: Option<Sender<()>>,
            receiver: Option<Receiver<()>>,
        }

        struct Env {
            promises: Vec<Pipe>,
            result: Vec<usize>,
        }

        impl Env {
            fn create(num: usize) -> Env {
                let vec: Vec<usize> = (0..num).collect();
                Env {
                    promises: vec
                        .iter()
                        .map(|_| {
                            let (sender, receiver) = oneshot::channel();
                            Pipe {
                                sender: Some(sender),
                                receiver: Some(receiver),
                            }
                        })
                        .collect(),
                    result: vec![],
                }
            }
        }

        async fn run_single_op(i: usize, queue: FlushQueue<usize>, env: Shared<Env>) {
            let env_cpy = env.clone();
            let env_cpy1 = env.clone();
            queue
                .run_with_ordered_post_op(
                    i,
                    async move {
                        let receiver = {
                            let env = env_cpy;
                            let p = &mut env.borrow_mut().promises[i];
                            mem::replace(&mut p.receiver, None)
                        };
                        receiver.unwrap().await.unwrap();
                    },
                    async move {
                        let env = env_cpy1;
                        let result = &mut env.borrow_mut().result;
                        result.push(i);
                    },
                )
                .await
        }

        let queue: FlushQueue<usize> = FlushQueue::new();
        let env = Shared::new(Env::create(num_ops));

        let mut ops = vec![];
        for i in &expected_result {
            sleep(Duration::from_nanos(100)).await;
            let (f, handle) = run_single_op(*i, queue.clone(), env.clone()).remote_handle();
            spawn(f);
            ops.push(handle);
        }

        fn shuffled(expected_result: &[usize]) -> Vec<usize> {
            let mut vec: Vec<usize> = expected_result.to_vec();
            vec.shuffle(&mut thread_rng());
            vec
        }

        // Let's sleep for a bit

        for i in shuffled(&expected_result) {
            let sender = {
                let p = &mut env.borrow_mut().promises[i];
                mem::replace(&mut p.sender, None).unwrap()
            };
            sender.send(()).unwrap();
        }

        // Wait for all to finish
        for op in ops {
            op.await
        }

        queue.wait_for_all().await;

        assert_that!(
            &env.borrow().result[0..],
            contains(expected_result).exactly()
        );
    }
}
