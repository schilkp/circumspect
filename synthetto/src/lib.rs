#![allow(clippy::too_many_arguments)]

pub use prost::{EncodeError, Message, bytes::BufMut, decode_length_delimiter};

pub mod protos {
    #![allow(clippy::large_enum_variant)]
    #![allow(clippy::enum_variant_names)]
    include!(concat!(env!("OUT_DIR"), "/perfetto.protos.rs"));
}

const TRUSTED_PACKET_SEQUENCE_ID: Option<protos::trace_packet::OptionalTrustedPacketSequenceId> =
    Some(
        protos::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(0xDEADBEEF),
    );

pub use protos::*;

pub enum ChildOrder {
    Lexicographic,
    Chronological,
    Explicit,
}

impl ChildOrder {
    fn to_proto_enum(&self) -> i32 {
        (match self {
            ChildOrder::Lexicographic => {
                protos::track_descriptor::ChildTracksOrdering::Lexicographic
            }
            ChildOrder::Chronological => {
                protos::track_descriptor::ChildTracksOrdering::Chronological
            }
            ChildOrder::Explicit => protos::track_descriptor::ChildTracksOrdering::Explicit,
        }) as i32
    }
}

#[derive(Debug)]
pub struct Synthetto {
    uuid_cnt: u64,
}

impl Default for Synthetto {
    fn default() -> Self {
        Self::new()
    }
}

impl Synthetto {
    pub fn new() -> Self {
        Synthetto { uuid_cnt: 1 }
    }

    fn next_uuid(&mut self) -> u64 {
        let uuid = self.uuid_cnt;
        self.uuid_cnt += 1;
        uuid
    }

    pub fn new_flow(&mut self) -> u64 {
        self.next_uuid()
    }

    pub fn new_process<B: BufMut>(
        &mut self,
        pid: i32,
        process_name: Option<String>,
        cmdline: Vec<String>,
        priority: Option<i32>,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
        buf: &mut B,
    ) -> Result<u64, EncodeError> {
        let uuid = self.next_uuid();

        let evt = TracePacket {
            timestamp: None,
            data: Some(protos::trace_packet::Data::TrackDescriptor(
                protos::TrackDescriptor {
                    uuid: Some(uuid),
                    parent_uuid: None,
                    process: Some(protos::ProcessDescriptor {
                        pid: Some(pid),
                        cmdline,
                        process_name,
                        process_priority: priority,
                        start_timestamp_ns: None,
                    }),
                    thread: None,
                    counter: None,
                    static_or_dynamic_name: None,
                    description,
                    child_ordering: child_ordering.map(|x| x.to_proto_enum()),
                    sibling_order_rank,
                    sibling_merge_behavior: None,
                    sibling_merge_key: None,
                },
            )),
            optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
            ..protos::TracePacket::default()
        };

        buf.put_u8(0x0A);
        evt.encode_length_delimited(buf)?;
        Ok(uuid)
    }

    pub fn new_thread<B: BufMut>(
        &mut self,
        pid: i32,
        tid: i32,
        thread_name: String,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
        buf: &mut B,
    ) -> Result<u64, EncodeError> {
        let uuid = self.next_uuid();

        let evt = TracePacket {
            timestamp: None,
            data: Some(protos::trace_packet::Data::TrackDescriptor(
                protos::TrackDescriptor {
                    uuid: Some(uuid),
                    parent_uuid: None,
                    process: None,
                    thread: Some(protos::ThreadDescriptor {
                        pid: Some(pid),
                        tid: Some(tid),
                        thread_name: Some(thread_name),
                    }),
                    counter: None,
                    static_or_dynamic_name: None,
                    description,
                    child_ordering: child_ordering.map(|x| x.to_proto_enum()),
                    sibling_order_rank,
                    sibling_merge_behavior: None,
                    sibling_merge_key: None,
                },
            )),
            optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
            ..protos::TracePacket::default()
        };

        buf.put_u8(0x0A);
        evt.encode_length_delimited(buf)?;
        Ok(uuid)
    }

    pub fn new_track<B: BufMut>(
        &mut self,
        name: String,
        parent_uuid: Option<u64>,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
        buf: &mut B,
    ) -> Result<u64, EncodeError> {
        let uuid = self.next_uuid();

        let evt = TracePacket {
            timestamp: None,
            data: Some(protos::trace_packet::Data::TrackDescriptor(
                protos::TrackDescriptor {
                    uuid: Some(uuid),
                    parent_uuid,
                    process: None,
                    thread: None,
                    counter: None,
                    static_or_dynamic_name: Some(
                        protos::track_descriptor::StaticOrDynamicName::Name(name),
                    ),
                    description,
                    child_ordering: child_ordering.map(|x| x.to_proto_enum()),
                    sibling_order_rank,
                    sibling_merge_behavior: None,
                    sibling_merge_key: None,
                },
            )),
            optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
            ..protos::TracePacket::default()
        };

        buf.put_u8(0x0A);
        evt.encode_length_delimited(buf)?;
        Ok(uuid)
    }

    pub fn new_counter<B: BufMut>(
        &mut self,
        name: String,
        unit: CounterTrackUnit,
        is_incremental: bool,
        parent_uuid: Option<u64>,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
        buf: &mut B,
    ) -> Result<u64, EncodeError> {
        let uuid = self.next_uuid();

        let evt = TracePacket {
            timestamp: None,
            data: Some(protos::trace_packet::Data::TrackDescriptor(
                protos::TrackDescriptor {
                    uuid: Some(uuid),
                    parent_uuid,
                    process: None,
                    thread: None,
                    counter: Some(protos::CounterDescriptor {
                        categories: vec![],
                        unit: unit.to_proto_unit(),
                        unit_name: unit.to_proto_unit_name(),
                        unit_multiplier: None,
                        is_incremental: Some(is_incremental),
                        y_axis_share_key: None,
                    }),
                    static_or_dynamic_name: Some(
                        protos::track_descriptor::StaticOrDynamicName::Name(name),
                    ),
                    description,
                    child_ordering: child_ordering.map(|x| x.to_proto_enum()),
                    sibling_order_rank,
                    sibling_merge_behavior: None,
                    sibling_merge_key: None,
                },
            )),
            optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
            ..protos::TracePacket::default()
        };

        buf.put_u8(0x0A);
        evt.encode_length_delimited(buf)?;
        Ok(uuid)
    }
}

pub fn slice_begin_evt<B: BufMut>(
    track_uuid: u64,
    ts: u64,
    name: Option<String>,
    flows: Vec<u64>,
    flows_end: Vec<u64>,
    correlation_id: Option<u64>,
    buf: &mut B,
) -> Result<(), EncodeError> {
    let correlation_id_field =
        correlation_id.map(protos::track_event::CorrelationIdField::CorrelationId);

    let evt = TracePacket {
        timestamp: Some(ts),
        data: Some(protos::trace_packet::Data::TrackEvent(protos::TrackEvent {
            name_field: name.map(protos::track_event::NameField::Name),
            track_uuid: Some(track_uuid),
            r#type: Some(protos::track_event::Type::SliceBegin as i32),
            flow_ids: flows,
            terminating_flow_ids: flows_end,
            correlation_id_field,
            ..protos::TrackEvent::default()
        })),
        optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
        ..protos::TracePacket::default()
    };

    buf.put_u8(0x0A);
    evt.encode_length_delimited(buf)?;
    Ok(())
}

pub fn slice_end_evt<B: BufMut>(
    track_uuid: u64,
    ts: u64,
    flows: Vec<u64>,
    flows_end: Vec<u64>,
    correlation_id: Option<u64>,
    buf: &mut B,
) -> Result<(), EncodeError> {
    let correlation_id_field =
        correlation_id.map(protos::track_event::CorrelationIdField::CorrelationId);

    let evt = protos::TracePacket {
        timestamp: Some(ts),
        data: Some(protos::trace_packet::Data::TrackEvent(protos::TrackEvent {
            name_field: None,
            track_uuid: Some(track_uuid),
            r#type: Some(protos::track_event::Type::SliceEnd as i32),
            flow_ids: flows,
            terminating_flow_ids: flows_end,
            correlation_id_field,
            ..protos::TrackEvent::default()
        })),
        optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
        ..protos::TracePacket::default()
    };

    buf.put_u8(0x0A);
    evt.encode_length_delimited(buf)?;
    Ok(())
}

pub fn instant_evt<B: BufMut>(
    track_uuid: u64,
    ts: u64,
    name: Option<String>,
    flows: Vec<u64>,
    flows_end: Vec<u64>,
    correlation_id: Option<u64>,
    buf: &mut B,
) -> Result<(), EncodeError> {
    let name_field = name.map(protos::track_event::NameField::Name);
    let correlation_id_field =
        correlation_id.map(protos::track_event::CorrelationIdField::CorrelationId);

    let evt = protos::TracePacket {
        timestamp: Some(ts),
        data: Some(protos::trace_packet::Data::TrackEvent(protos::TrackEvent {
            name_field,
            track_uuid: Some(track_uuid),
            r#type: Some(protos::track_event::Type::Instant as i32),
            flow_ids: flows,
            terminating_flow_ids: flows_end,
            correlation_id_field,
            ..protos::TrackEvent::default()
        })),
        optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
        ..protos::TracePacket::default()
    };

    buf.put_u8(0x0A);
    evt.encode_length_delimited(buf)?;
    Ok(())
}

pub fn int_counter_evt<V, B: BufMut>(
    track_uuid: u64,
    ts: u64,
    val: V,
    buf: &mut B,
) -> Result<(), EncodeError>
where
    V: Into<i64>,
{
    let evt = protos::TracePacket {
        timestamp: Some(ts),
        data: Some(protos::trace_packet::Data::TrackEvent(protos::TrackEvent {
            track_uuid: Some(track_uuid),
            r#type: Some(protos::track_event::Type::Counter as i32),
            counter_value_field: Some(protos::track_event::CounterValueField::CounterValue(
                val.into(),
            )),
            ..protos::TrackEvent::default()
        })),
        optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
        ..protos::TracePacket::default()
    };
    buf.put_u8(0x0A);
    evt.encode_length_delimited(buf)?;
    Ok(())
}

pub fn float_counter_evt<V, B: BufMut>(
    track_uuid: u64,
    ts: u64,
    val: V,
    buf: &mut B,
) -> Result<(), EncodeError>
where
    V: Into<f64>,
{
    let evt = protos::TracePacket {
        timestamp: Some(ts),
        data: Some(protos::trace_packet::Data::TrackEvent(protos::TrackEvent {
            track_uuid: Some(track_uuid),
            r#type: Some(protos::track_event::Type::Counter as i32),
            counter_value_field: Some(protos::track_event::CounterValueField::DoubleCounterValue(
                val.into(),
            )),
            ..protos::TrackEvent::default()
        })),
        optional_trusted_packet_sequence_id: TRUSTED_PACKET_SEQUENCE_ID,
        ..protos::TracePacket::default()
    };

    buf.put_u8(0x0A);
    evt.encode_length_delimited(buf)?;
    Ok(())
}

pub enum CounterTrackUnit {
    Unspecified,
    TimeNs,
    Count,
    SizeBytes,
    Custom(String),
}

impl CounterTrackUnit {
    pub fn from_string(unit_name: Option<String>) -> Self {
        match unit_name.as_deref() {
            None => CounterTrackUnit::Unspecified,
            Some("") => CounterTrackUnit::Unspecified,
            Some("TimeNs") => CounterTrackUnit::TimeNs,
            Some("Count") => CounterTrackUnit::Count,
            Some("SizeBytes") => CounterTrackUnit::SizeBytes,
            Some(custom) => CounterTrackUnit::Custom(custom.to_string()),
        }
    }

    fn to_proto_unit(&self) -> Option<i32> {
        Some(match self {
            CounterTrackUnit::Unspecified => protos::counter_descriptor::Unit::Unspecified,
            CounterTrackUnit::TimeNs => protos::counter_descriptor::Unit::TimeNs,
            CounterTrackUnit::Count => protos::counter_descriptor::Unit::Count,
            CounterTrackUnit::SizeBytes => protos::counter_descriptor::Unit::SizeBytes,
            CounterTrackUnit::Custom(_) => protos::counter_descriptor::Unit::Unspecified,
        } as i32)
    }

    fn to_proto_unit_name(&self) -> Option<String> {
        if let Self::Custom(name) = self {
            Some(name.clone())
        } else {
            None
        }
    }
}
