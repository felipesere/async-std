use crate::sync::{channel, Receiver, Sender};

use std::pin::Pin;

use pin_project_lite::pin_project;

use crate::stream::Stream;
use crate::task::{Context, Poll};


pin_project! {
    ///
    /// A strem that has been partitioned and only receives a subset of the original stream
    ///
    /// The `receiver` is where a side of the partition will poll for new items.
    /// The `poller` is where a side of the partition will notify the main stream that it was polled.
    ///
    pub struct PartitionStream<T> {
        #[pin]
        receiver: Receiver<T>,
        poller: Sender<()>,
    }
}


fn partition<S, F>(source: S, is_left: F) -> (PartitionStream<S::Item>, PartitionStream<S::Item>)
    where
        S: Stream,
        F: Fn(&S::Item) -> bool,
{
    let (poller, _foo) = channel(1);

    let (_a, left) = channel(1);
    let (_b, right) = channel(1);

    (
        PartitionStream {poller: poller.clone(), receiver: left },
        PartitionStream {poller: poller.clone(), receiver: right },
    )
}

impl<T> Stream for PartitionStream<T> {
   type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut send_fut = Box::pin(self.poller.send(()));

        let sent = match send_fut.poll(cx) {
            Poll::Ready(val) => val,
            _ => return Poll::Pending, // wat? Not really sure what to do here...
        };

        let next = match this.receiver.poll_next(cx) {
            Poll::Ready(val) => val,
            _ => return Poll::Pending, // what should be do here?
        };

        match next {
            ready @ Some(_) => Poll::Ready(ready),
            _ => {
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use crate::task;
    use crate::stream::stream::StreamExt;

    #[test]
    fn it_receives_things_on_the_left() {
        task::block_on(async {
            let s: VecDeque<usize> = vec![1, 2, 3, 4].into_iter().collect();

            let (mut a, b) = partition(s, |&val| val % 2 == 0);

            assert_eq!(a.next().await, Some(1));
        })
    }
}
