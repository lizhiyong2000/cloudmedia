//! Connection Request Handler
//!
//! This module contains the logic for sending all outgoing messagse through the connection sink.

use futures::stream::Fuse;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{Future, Sink, Stream, ready};

use crate::protocol::rtsp::header::map::HeaderMapExtension;
use crate::protocol::rtsp::header::types::Date;
use crate::protocol::rtsp::protocol::codec::{Message, ProtocolError};
use futures::task::{Poll, Context};
use std::pin::Pin;

/// The type responsible for sending all outgoing messages through the connection sink.
///
/// To ensure that all messages are sent through the sink, the sender instance should not be dropped
/// until [`Sender::poll`] has returned `Ok(Ready(()))`.
#[must_use = "futures do nothing unless polled"]
pub struct Sender<TSink>
where
    TSink: Sink<Message, Error = ProtocolError> + Send + 'static,
{
    /// The current message that we are trying to send but cannot yet because the sink is not ready.
    buffered_message: Option<Message>,

    /// The outgoing stream of messages that are to be sent through the sink.
    rx_outgoing_message: Fuse<UnboundedReceiver<Message>>,

    /// The sink representing a reliable transport mechanism (e.g. TCP).
    sink: TSink,
}

impl<TSink> Sender<TSink>
where
    TSink: Sink<Message, Error = ProtocolError> + Send + 'static,
{
    /// Constructs a new sender as a wrapper around the given sink.
    ///
    /// The sink must represent a reliable transport mechanism (e.g. TCP).
    ///
    /// Along with the sender itself, a handle to the sender is also returned. This handle is
    /// used to send messages through the sender and is cloneable.
    pub fn new(sink: TSink) -> (Self, SenderHandle) {
        let (tx_outgoing_message, rx_outgoing_message) = unbounded();
        let sender = Sender {
            buffered_message: None,
            rx_outgoing_message: rx_outgoing_message.fuse(),
            sink,
        };

        (sender, SenderHandle(tx_outgoing_message))
    }

    /// Reads outgoing messages to be sent outwards and submits them to the internal sink.
    ///
    /// All outgoing messages automatically have a `"Date"` header appended with the current time.
    ///
    /// If `Poll::Ready(())` is returned, then the outgoing message stream has ended, so there
    /// is no longer any new messages to be sent. There may still be messages that have yet to have
    /// been flushed though.
    ///
    /// If `Poll::Pending` is returned, then the sink is unable to accept the message at this
    /// time, probably because it is full. The message will be buffered temporarily until we can try
    /// to send it through the sink again.
    ///
    /// If `Err(`[`ProtocolError`]`)` is returned, there was either an error trying to send a
    /// message through the sink or there was an error trying to flush the sink.
    fn poll_write(&mut self) -> Poll<()> {
        loop {
            match self
                .rx_outgoing_message
                .poll()
                .expect("`Sender.rx_outgoing_message` should not error")
            {
                Poll::Ready(Some(mut message)) => {
                    match message {
                        Message::Request(ref mut request) => {
                            request.headers_mut().typed_insert(Date::new());
                        }
                        Message::Response(ref mut response) => {
                            response.headers_mut().typed_insert(Date::new());
                        }
                    }

                    ready!(self.try_send_message(message));
                }
                Poll::Pending => {
                    ready!(self.sink.poll_complete());
                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready(()),
            }
        }
    }

    /// Tries to send the given message through the internal sink.
    ///
    /// If `Poll::Ready(())` is returned, then the message was successfully sent through the
    /// sink. It may not have been flushed yet though, this will happen at a later point.
    ///
    /// If `Poll::Pending` is returned, then the sink is unable to accept the message at this
    /// time, probably because it is full. The message will be buffered temporarily until we can try
    /// to send it through the sink again.
    ///
    /// If `Err(`[`ProtocolError`]`)` is returned, there was an error trying to send the message
    /// through the sink.
    fn try_send_message(&mut self, message: Message) -> Poll<()> {
        debug_assert!(self.buffered_message.is_none());

        if let Ok(()) = self.sink.start_send(message)? {
            self.buffered_message = Some(message);
            return Poll::Pending;
        }

        Poll::Ready(())
    }
}

impl<TSink> Future for Sender<TSink>
where
    TSink: Sink<Message, Error = ProtocolError> + Send + 'static,
{
    type Output = ();


    /// Reads outgoing messages to be sent outwards and submits them to the internal sink.
    ///
    /// If `Poll::Ready(())` is returned, then the outgoing message stream has ended, so there
    /// are no longer any new messages to be sent and all existing messages have been flushed.
    ///
    /// If `Poll::Pending` is returned, then the sink is unable to accept anymore messages at
    /// the current time.
    ///
    /// If `Err(`[`ProtocolError`]`)` is returned, there was either an error trying to send a
    /// message through the sink or there was an error trying to flush the sink.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(buffered_message) = self.buffered_message.take() {
            ready!(self.try_send_message(buffered_message));
        }

        ready!(self.poll_write());

        debug_assert!(self.buffered_message.is_none());
        self.sink.close()
    }
}

#[derive(Clone, Debug)]
pub struct SenderHandle(pub(crate) UnboundedSender<Message>);

impl SenderHandle {
    pub fn try_send_message(&self, message: Message) -> Result<(), ()> {
        self.0.unbounded_send(message).map_err(|_| ())
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use futures::channel::mpsc;
    // use futures::{Sink, Stream};
    use std::mem;
    use tokio::runtime::current_thread;

    use crate::protocol::rtsp::header::name::HeaderName;
    use crate::protocol::rtsp::method::Method;
    use crate::protocol::rtsp::protocol::codec::Message;
    use crate::protocol::rtsp::protocol::connection::sender::Sender;
    use crate::protocol::rtsp::request::Request;
    use crate::protocol::rtsp::uri::request::URI;

    #[test]
    fn test_sender_send_message() {
        let (tx_sink, rx_sink) = mpsc::unbounded();
        let (sender, handle) = Sender::new(Box::new(tx_sink.sink_map_err(|_| panic!())));

        let message = Message::Request(
            Request::<()>::builder()
                .with_method(Method::Setup)
                .with_uri(URI::asterisk())
                .with_body(BytesMut::new())
                .build()
                .unwrap(),
        );
        handle.try_send_message(message.clone()).unwrap();

        // Need to drop the handle, otherwise the sender will never finish.
        mem::drop(handle);

        assert!(current_thread::block_on_all(sender).is_ok());

        let mut messages = current_thread::block_on_all(rx_sink.collect()).unwrap();
        assert_eq!(messages.len(), 1);

        let request = match messages.remove(0) {
            Message::Request(mut request) => {
                request.headers_mut().remove(&HeaderName::Date).unwrap();
                request
            }
            _ => panic!(),
        };
        assert_eq!(message, Message::Request(request));
    }
}