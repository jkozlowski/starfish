use std::future::Future;
use tokio_executor::current_thread::CurrentThread;

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{

}