use std::io;
use std::io::{Read, Write};

use std;

use tokio_core::io::{Codec, EasyBuf};

use codecs::{ToByte, FromByte};
use error::{Result, KafkaCode};
use utils::PartitionOffset;
use super::{HeaderRequest, HeaderResponse};
use super::{API_KEY_OFFSET, API_VERSION};


#[derive(Debug)]
pub struct OffsetRequest<'a, S: AsRef<str>> {
    pub header: HeaderRequest<S>,
    pub replica: i32,
    pub topic_partitions: Vec<TopicPartitionOffsetRequest<'a>>,
}

#[derive(Default, Debug)]
pub struct TopicPartitionOffsetRequest<'a> {
    pub topic: &'a str,
    pub partitions: Vec<PartitionOffsetRequest>,
}

#[derive(Default, Debug)]
pub struct PartitionOffsetRequest {
    pub partition: i32,
    pub max_offsets: i32,
    pub time: i64,
}

impl<'a, S: AsRef<str>> OffsetRequest<'a, S> {
    pub fn new(correlation_id: i32, client_id: S) -> OffsetRequest<'a, S> {
        OffsetRequest {
            header: HeaderRequest::new(API_KEY_OFFSET, API_VERSION, correlation_id, client_id),
            replica: -1,
            topic_partitions: vec![],
        }
    }

    pub fn add(&mut self, topic: &'a str, partition: i32, time: i64) {
        for tp in &mut self.topic_partitions {
            if tp.topic == topic {
                tp.add(partition, time);
                return;
            }
        }
        let mut tp = TopicPartitionOffsetRequest::new(topic);
        tp.add(partition, time);
        self.topic_partitions.push(tp);
    }
}

// struct OffsetRequestCodec

// impl Codec for OffsetRequestCodec {
//     type In = OffsetRequest<AsRef<str>>;
//     type Out = OffsetRequest;

//     fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
//         msg.encode(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
//     }

//     fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {

        
//         unimplemented!()
//     }
//}

impl<'a> TopicPartitionOffsetRequest<'a> {
    pub fn new(topic: &'a str) -> TopicPartitionOffsetRequest<'a> {
        TopicPartitionOffsetRequest {
            topic: topic,
            partitions: vec![],
        }
    }

    pub fn add(&mut self, partition: i32, time: i64) {
        self.partitions.push(PartitionOffsetRequest::new(partition, time));
    }
}

impl PartitionOffsetRequest {
    pub fn new(partition: i32, time: i64) -> PartitionOffsetRequest {
        PartitionOffsetRequest {
            partition: partition,
            max_offsets: 1,
            time: time,
        }
    }
}

impl<'a, S: AsRef<str>> ToByte for OffsetRequest<'a, S> {
    fn encode<T: Write>(&self, buffer: &mut T) -> Result<()> {
        try_multi!(self.header.encode(buffer),
                   self.replica.encode(buffer),
                   self.topic_partitions.encode(buffer))
    }
}

impl<'a> ToByte for TopicPartitionOffsetRequest<'a> {
    fn encode<T: Write>(&self, buffer: &mut T) -> Result<()> {
        try_multi!(self.topic.encode(buffer), self.partitions.encode(buffer))
    }
}

impl ToByte for PartitionOffsetRequest {
    fn encode<T: Write>(&self, buffer: &mut T) -> Result<()> {
        try_multi!(self.partition.encode(buffer),
                   self.time.encode(buffer),
                   self.max_offsets.encode(buffer))
    }
}

// --------------------------------------------------------------------

#[derive(Default, Debug)]
pub struct OffsetResponse {
    pub header: HeaderResponse,
    pub topic_partitions: Vec<TopicPartitionOffsetResponse>,
}

#[derive(Default, Debug)]
pub struct TopicPartitionOffsetResponse {
    pub topic: String,
    pub partitions: Vec<PartitionOffsetResponse>,
}

#[derive(Default, Debug)]
pub struct PartitionOffsetResponse {
    pub partition: i32,
    pub error: i16,
    pub offset: Vec<i64>,
}

impl PartitionOffsetResponse {
    pub fn into_offset(&self) -> std::result::Result<PartitionOffset, KafkaCode> {
        match KafkaCode::from_protocol(self.error) {
            Some(code) => Err(code),
            None => {
                let offset = match self.offset.first() {
                    Some(offs) => *offs,
                    None => -1,
                };

                Ok(PartitionOffset {
                    partition: self.partition,
                    offset: offset,
                })
            }
        }
    }
}

impl FromByte for OffsetResponse {
    type R = OffsetResponse;

    #[allow(unused_must_use)]
    fn decode<T: Read>(&mut self, buffer: &mut T) -> Result<()> {
        try_multi!(self.header.decode(buffer), self.topic_partitions.decode(buffer))
    }
}

impl FromByte for TopicPartitionOffsetResponse {
    type R = TopicPartitionOffsetResponse;

    #[allow(unused_must_use)]
    fn decode<T: Read>(&mut self, buffer: &mut T) -> Result<()> {
        try_multi!(self.topic.decode(buffer), self.partitions.decode(buffer))
    }
}

impl FromByte for PartitionOffsetResponse {
    type R = PartitionOffsetResponse;

    #[allow(unused_must_use)]
    fn decode<T: Read>(&mut self, buffer: &mut T) -> Result<()> {
        try_multi!(self.partition.decode(buffer),
                   self.error.decode(buffer),
                   self.offset.decode(buffer))
    }
}
