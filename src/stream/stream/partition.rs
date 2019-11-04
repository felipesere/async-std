use crate::sync::{channel, Receiver, Sender};

use std::pin::Pin;

use pin_project_lite::pin_project;

use crate::stream::Stream;
use crate::task::{Context, Poll};

///
/// A strem that has been partitioned and only receives a subset of the original stream
///
/// The `receiver` is where a side of the partition will poll for new items.
/// The `poller` is where a side of the partition will notify the main stream that it was polled.
///
pub struct PartitionStream<T> {
    receiver: Receiver<T>,
    poller: Sender<()>,
}


fn partition<S, F>(source: S, is_left: F) -> (PartitionStream<S::Item>, PartitionStream<S::Item>)
    where
        S: Stream,
        F: Fn(&S::Item) -> bool,
{
    let (poller, foo) = channel(1);

    let (a, left) = channel(1);
    let (b, right) = channel(1);

    (
        PartitionStream {poller: poller.clone(), receiver: left },
        PartitionStream {poller: poller.clone(), receiver: right },
    )
}


impl<T> Stream for PartitionStream<T> {
   type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}
