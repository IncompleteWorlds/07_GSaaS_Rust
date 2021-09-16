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


use crate::wait_for_task::SharedState;


#[derive(Debug)]
pub struct ProcessMessageFuture {
    shared_state: SharedState,
}
 
 
impl Future for ProcessMessageFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {        
        debug!("Poll ProcessMessageFuture. Completed: {}", self.shared_state.completed);

        if self.shared_state.completed {
            Poll::Ready(())
        } else {
            self.shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}


impl ProcessMessageFuture {
    pub fn new() -> Self {
        let shared_state = SharedState {
            completed: false,
            waker: None,
        };

        ProcessMessageFuture { shared_state }
    }

    pub fn is_completed(&self) -> bool 
    {        
        self.shared_state.completed
    }

    pub fn reset(&mut self)
    {
        self.shared_state.completed = false;
    }

    pub fn set_completed(&mut self) 
    {
        debug!("Asynchronous task unblocked");

        self.shared_state.completed = true;
        if let Some(waker) = self.shared_state.waker.take() {
            waker.wake()
        }
    }

    pub fn do_work(&mut self, in_job: fn()) 
    {
        in_job();

        self.set_completed();
    }
}


