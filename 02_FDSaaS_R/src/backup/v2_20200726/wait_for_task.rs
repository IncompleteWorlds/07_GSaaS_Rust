/**
 * (c) Incomplete Worlds 2020
 * Alberto Fernandez (ajfg)
 *
 * FDS as a Service
 * WaitForAnswerFuture
 * It impliments a future that is only wake up when another task is completed
 * and the 'completed' flag is changed
 * 
 */
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::pin::Pin;

use log::{debug, error, info, trace, warn};


// Shared state between the future and the waiting thread
#[derive(Debug)]
pub struct SharedState {
    // Whether or not the sleep time has elapsed
    pub completed: bool,

    // The waker for the task that `TimerFuture` is running on.
    // The thread can use this after setting `completed = true` to tell
    // `TimerFuture`'s task to wake up, see that `completed = true`, and
    // move forward.
    pub waker: Option<Waker>, 
}
 

#[derive(Debug)]
pub struct WaitForAnswerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}
 
 
impl Future for WaitForAnswerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();
        
        debug!("Poll WaitForAnswerFuture. Completed: {}", shared_state.completed);
        if shared_state.completed == true {
            Poll::Ready(())
        } else {
            // Set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed = true`.
            //
            // It's tempting to do this once rather than repeatedly cloning
            // the waker each time. However, the `WaitForAnswerFuture` can move between
            // tasks on the executor, which could cause a stale waker pointing
            // to the wrong task, preventing `WaitForAnswerFuture` from waking up
            // correctly.
            //
            // N.B. it's possible to check for this using the `Waker::will_wake`
            // function, but we omit that here to keep things simple.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl WaitForAnswerFuture {
    // Create a new `WaitForAnswerFuture` which will complete when signaled by an 
    // external task
    pub fn new() -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        WaitForAnswerFuture { shared_state }
    }

    pub fn is_completed(&self) -> bool 
    {
        let shared_state = self.shared_state.lock().unwrap();

        shared_state.completed
    }

    pub fn reset(&self)
    {
        let mut shared_state = self.shared_state.lock().unwrap();
        
        shared_state.completed = false;
    }

    pub fn set_completed(&self) 
    {
        let mut shared_state = self.shared_state.lock().unwrap();

        debug!("Asynchronous task unblocked");

        // Signal that the timer has completed and wake up the last
        // task on which the future was polled, if one exists.
        shared_state.completed = true;
        if let Some(waker) = shared_state.waker.take() {
            waker.wake()
        }
    }
}

