use std::future::Future;
use tokio_executor::current_thread::spawn as spawn_local;

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_local(future)
}