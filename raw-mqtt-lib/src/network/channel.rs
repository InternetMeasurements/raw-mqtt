use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use bytes::BytesMut;

use crate::network::channel::ChannelResult::{Added, Replaced};

pub enum ChannelResult {
    Added,
    Replaced
}

#[derive(Debug, Clone)]
pub(crate) enum ChannelSender {
    Bounded(BoundedSender),
    Unbounded(UnboundedSender),
    Lifo(SingleLifoQueue)
}

pub(crate) enum ChannelReceiver {
    Bounded(BoundedReceiver),
    Unbounded(UnboundedReceiver),
    Lifo(SingleLifoQueue)
}

impl ChannelSender {

    pub(crate) async fn send(&mut self, buffer: BytesMut) -> Result<ChannelResult, ()> {
        match self {
            ChannelSender::Bounded(sender) => {
                sender.send(buffer).await
            },
            ChannelSender::Unbounded(sender) => {
                sender.send(buffer).await
            },
            ChannelSender::Lifo(sender) => {
                sender.send(buffer).await
            }
        }
    }

}

impl ChannelReceiver {
    pub(crate) async fn receive(&mut self) -> Result<BytesMut, ()> {
        match self {
            ChannelReceiver::Bounded(receiver) => {
                receiver.receive().await
            },
            ChannelReceiver::Unbounded(receiver) => {
                receiver.receive().await
            },
            ChannelReceiver::Lifo(receiver) => {
                receiver.receive().await
            }
        }
    }
}


#[derive(Debug, Clone)]
pub(crate) struct SingleLifoQueue {
    queue: Arc<Mutex<BytesMut>>,
    permit: Arc<Semaphore>
}
impl SingleLifoQueue {
    pub(crate) fn new() -> SingleLifoQueue {
        let queue = Mutex::new(BytesMut::new());
        SingleLifoQueue {
            queue: Arc::new(queue),
            permit: Arc::new(Semaphore::new(0))
        }
    }

    async fn send(&mut self, buffer: BytesMut) -> Result<ChannelResult, ()> {
        let mut guard = self.queue.lock().unwrap();
        guard.clear();
        guard.extend_from_slice(&buffer);
        if self.permit.available_permits() == 0 {
            self.permit.add_permits(1);
            Ok(Added)
        }
        else {
            Ok(Replaced)
        }
    }

    async fn receive(&mut self) -> Result<BytesMut, ()> {
        self.permit.acquire().await.unwrap().forget();
        Ok(self.queue.lock().unwrap().clone())
    }
}


#[derive(Debug, Clone)]
pub(crate) struct BoundedSender {
    sender: tokio::sync::mpsc::Sender<BytesMut>,
}
impl BoundedSender {

    pub(crate) fn new(sender: tokio::sync::mpsc::Sender<BytesMut>) -> BoundedSender {
        BoundedSender {
            sender
        }
    }
    async fn send(&mut self, buffer: BytesMut) -> Result<ChannelResult, ()> {
        self.sender.send(buffer).await.unwrap();
        Ok(Added)
    }
}


#[derive(Debug, Clone)]
pub(crate) struct UnboundedSender {
    sender: tokio::sync::mpsc::UnboundedSender<BytesMut>,
}
impl UnboundedSender {

    pub(crate)fn new(sender: tokio::sync::mpsc::UnboundedSender<BytesMut>) -> UnboundedSender {
        UnboundedSender {
            sender
        }
    }
    async fn send(&mut self, buffer: BytesMut) -> Result<ChannelResult, ()> {
        self.sender.send(buffer).unwrap();
        Ok(Added)
    }
}

#[derive(Debug)]
pub(crate) struct BoundedReceiver {
    receiver: tokio::sync::mpsc::Receiver<BytesMut>,
}
impl BoundedReceiver {

    pub(crate)fn new(receiver: tokio::sync::mpsc::Receiver<BytesMut>) -> BoundedReceiver {
        BoundedReceiver {
            receiver
        }
    }
    async fn receive(&mut self) -> Result<BytesMut, ()> {
        match self.receiver.recv().await {
            Some(buffer) => {
                Ok(buffer)
            },
            None => {
                Err(())
            }
        }
    }
}


#[derive(Debug)]
pub(crate) struct UnboundedReceiver {
    receiver: tokio::sync::mpsc::UnboundedReceiver<BytesMut>,
}
impl UnboundedReceiver {

    pub(crate)fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<BytesMut>) -> UnboundedReceiver {
        UnboundedReceiver {
            receiver
        }
    }
    async fn receive(&mut self) -> Result<BytesMut, ()> {
        match self.receiver.recv().await {
            Some(buffer) => {
                Ok(buffer)
            },
            None => {
                Err(())
            }
        }
    }
}
