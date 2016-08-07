//! A crate private module to expose `KafkaClient` internals for use
//! within this crate but not outside of it.

use client;
use error::Result;

pub trait KafkaClientInternals {
    fn internal_produce_messages<'a, 'b, I, J>(
        &mut self, ack: client::RequiredAcks, ack_timeout: i32, messages: I)
        -> Result<Vec<client::TopicPartitionOffset>>
        where J: AsRef<client::ProduceMessage<'a, 'b>>, I: IntoIterator<Item=J>;
}
