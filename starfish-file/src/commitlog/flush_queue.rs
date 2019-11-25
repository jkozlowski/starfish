// TODO: Add gates
use crate::Shared;
use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use futures::channel::oneshot::Sender;
use futures::future::Shared as SharedFut;
use futures::FutureExt;
use std::collections::BTreeMap;
use std::fmt;
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

impl fmt::Debug for Notifier {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

/// Keeps an ordered queue of pending operations. Allows flushes for various chunks
/// to complete in arbitrary order while making sure that callbacks for mutations
/// at higher position run only after all the lower position mutations are finished.
// #[derive(Default)]
#[derive(Clone, Debug)]
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

    pub async fn run_with_ordered_post_op<R, F, P>(&self, t: T, action: F, post: P) -> R
    where
        F: Future<Output = ()> + 'static,
        P: Future<Output = R> + 'static,
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
            let mut iter = map.range_mut((Bound::Unbounded, Bound::Excluded(t)));
            if let Some(prev) = iter.next_back() {
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
        let ret = post.await;

        let mut map = self.map.borrow_mut();
        let mut iter = map.range_mut(t..);
        let me = iter.next().unwrap();
        me.1.count -= 1;

        if me.1.count == 0 {
            let notifier_again = map.remove(&t).unwrap();
            notifier_again.sender.send(()).unwrap()
        }

        return ret;
    }

    // Waits for all operations currently active to finish
    pub async fn wait_for_all(&self) {
        if !self.map.is_empty() {
            let last_key = *self.map.range(..).next_back().unwrap().0;
            return self.wait_for_pending(last_key).await;
        }
    }

    // Waits for all operations whose key is less than or equal to "rp"
    // to complete
    pub async fn wait_for_pending(&self, t: T) {
        if let Some(e) = self
            .map
            .range((Bound::Unbounded, Bound::Included(t)))
            .next_back()
        {
            e.1.receiver.clone().await.unwrap();
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
    use futures::FutureExt;
    use hamcrest2::assert_that;
    use hamcrest2::prelude::*;
    use rand::prelude::*;
    use rand::thread_rng;
    use std::mem;
    use std::time::Duration;
    use tokio::timer::delay_for;

    #[tokio::test]
    pub async fn test_run_with_ordered_post_op() {
        let num_ops = 5;
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
            let ret = queue
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
                        i
                    },
                )
                .await;
            assert_that!(ret, is(eq(i)));
        }

        let queue: FlushQueue<usize> = FlushQueue::new();
        let env = Shared::new(Env::create(num_ops));

        let mut ops = vec![];
        for i in &expected_result {
            // Tests overlaping borrows
            delay_for(Duration::from_nanos(1)).await;
            let (f, handle) = run_single_op(*i, queue.clone(), env.clone()).remote_handle();
            spawn(f);
            ops.push(handle);
        }

        fn shuffled(expected_result: &[usize]) -> Vec<usize> {
            let mut vec: Vec<usize> = expected_result.to_vec();
            vec.shuffle(&mut thread_rng());
            vec
        }

        for i in shuffled(&expected_result) {
            let sender = {
                let p = &mut env.borrow_mut().promises[i];
                mem::replace(&mut p.sender, None).unwrap()
            };
            delay_for(Duration::from_nanos(1)).await;
            sender.send(()).unwrap();
        }

        queue.wait_for_all().await;

        assert_that!(
            &env.borrow().result[..],
            contains(expected_result).exactly()
        );
    }
}
