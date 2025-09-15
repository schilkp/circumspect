#![allow(clippy::too_many_arguments)]

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use synthetto::{ChildOrder, Synthetto};

pub mod dpi;
mod svdpi;

fn ioerr_to_str(e: io::Error) -> String {
    format!("Failed to write to file - {e}")
}

#[derive(Debug, Clone)]
struct TrackSlice {
    name: Option<String>,
    flows: Vec<u64>,
}

impl TrackSlice {
    fn new(name: Option<String>, mut flows: Vec<u64>) -> Self {
        flows.sort_unstable();
        Self { name, flows }
    }
}

impl PartialEq for TrackSlice {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if self.flows.len() != other.flows.len() {
            return false;
        }

        let mut self_flows = self.flows.clone();
        let mut other_flows = other.flows.clone();
        self_flows.sort_unstable();
        other_flows.sort_unstable();
        self_flows == other_flows
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CounterValue {
    Int(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
struct Counter {
    last_value: CounterValue,
}

impl Counter {
    fn new(value: CounterValue) -> Self {
        Self { last_value: value }
    }
}

#[derive(Debug, Default)]
struct Track {
    active_slices: Vec<TrackSlice>,
}

#[derive(Debug)]
struct Context {
    w: BufWriter<File>,
    synthetto: Synthetto,
    timescale: f64,
    time_mult: u32,
    tracks: HashMap<u64, Track>,
    counters: HashMap<u64, Counter>,
    encode_buffer: Vec<u8>,
}

pub enum ReplacementBehaviour {
    NewSlice,
    Replace,
    ReplaceIfDifferent,
}

impl Context {
    pub fn new(path: PathBuf, timescale: f64, time_mult: u32) -> Result<Self, String> {
        let f = BufWriter::new(
            File::create(&path).map_err(|e| format!("Failed to open trace file - {e}"))?,
        );

        Ok(Context {
            w: f,
            synthetto: Synthetto::new(),
            timescale,
            time_mult,
            tracks: HashMap::new(),
            counters: HashMap::new(),
            encode_buffer: Vec::with_capacity(64),
        })
    }

    fn convert_ts(&self, ts: f64) -> u64 {
        let ts_sec = self.timescale * ts;
        let ts_nsec = ts_sec * 1000000000.0;
        let ts_scaled = ts_nsec * (self.time_mult as f64);
        ts_scaled as u64
    }

    fn get_mut_track(&mut self, uuid: u64) -> &mut Track {
        self.tracks.entry(uuid).or_default()
    }

    fn trim_encode_buffer(&mut self) {
        if self.encode_buffer.capacity() > 256 {
            self.encode_buffer = Vec::with_capacity(256);
        }
    }

    pub fn flush(&mut self) -> Result<(), String> {
        self.w
            .flush()
            .map_err(|e| format!("Failed to flush to trace file - {e}"))
    }

    pub fn new_flow(&mut self) -> u64 {
        self.synthetto.new_flow()
    }

    pub fn new_track(
        &mut self,
        name: String,
        parent_uuid: Option<u64>,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
    ) -> Result<u64, String> {
        self.encode_buffer.clear();

        let uuid = self
            .synthetto
            .new_track(
                name,
                parent_uuid,
                description,
                child_ordering,
                sibling_order_rank,
                &mut self.encode_buffer,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();

        Ok(uuid)
    }

    pub fn new_process(
        &mut self,
        pid: i32,
        process_name: Option<String>,
        cmdline: Vec<String>,
        priority: Option<i32>,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
    ) -> Result<u64, String> {
        self.encode_buffer.clear();

        let uuid = self
            .synthetto
            .new_process(
                pid,
                process_name,
                cmdline,
                priority,
                description,
                child_ordering,
                sibling_order_rank,
                &mut self.encode_buffer,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();

        Ok(uuid)
    }

    pub fn new_thread(
        &mut self,
        pid: i32,
        tid: i32,
        thread_name: String,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
    ) -> Result<u64, String> {
        self.encode_buffer.clear();

        let uuid = self
            .synthetto
            .new_thread(
                pid,
                tid,
                thread_name,
                description,
                child_ordering,
                sibling_order_rank,
                &mut self.encode_buffer,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();

        Ok(uuid)
    }

    pub fn new_counter(
        &mut self,
        name: String,
        unit_name: Option<String>,
        is_incremental: bool,
        parent_uuid: Option<u64>,
        description: Option<String>,
        child_ordering: Option<ChildOrder>,
        sibling_order_rank: Option<i32>,
    ) -> Result<u64, String> {
        self.encode_buffer.clear();

        let unit = synthetto::CounterTrackUnit::from_string(unit_name);
        let uuid = self
            .synthetto
            .new_counter(
                name,
                unit,
                is_incremental,
                parent_uuid,
                description,
                child_ordering,
                sibling_order_rank,
                &mut self.encode_buffer,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();

        Ok(uuid)
    }

    pub fn slice_begin_evt(
        &mut self,
        track_uuid: u64,
        ts: f64,
        name: Option<String>,
        flows: Vec<u64>,
        flows_end: Vec<u64>,
        replace_behaviour: ReplacementBehaviour,
        correlation_id: Option<u64>,
    ) -> Result<(), String> {
        let new_slice = TrackSlice::new(name.clone(), flows.clone());

        match replace_behaviour {
            ReplacementBehaviour::Replace => {
                let track = self.get_mut_track(track_uuid);
                if !track.active_slices.is_empty() {
                    self.slice_end_evt(track_uuid, ts, vec![], vec![], true, None)?;
                }
            }
            ReplacementBehaviour::NewSlice => {
                // No replacement - always create new slice.
            }
            ReplacementBehaviour::ReplaceIfDifferent => {
                let track = self.get_mut_track(track_uuid);
                if let Some(current_slice) = track.active_slices.last() {
                    if *current_slice == new_slice {
                        // Same slice, do nothing
                        return Ok(());
                    } else {
                        self.slice_end_evt(track_uuid, ts, vec![], vec![], true, None)?;
                    }
                }
            }
        }

        self.encode_buffer.clear();
        let ts = self.convert_ts(ts);
        synthetto::slice_begin_evt(
            track_uuid,
            ts,
            name,
            flows,
            flows_end,
            correlation_id,
            &mut self.encode_buffer,
        )
        .expect("prost encode should only fail if buffer is too small, but buffer is vec");
        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();

        self.get_mut_track(track_uuid).active_slices.push(new_slice);
        Ok(())
    }

    pub fn slice_end_evt(
        &mut self,
        track_uuid: u64,
        ts: f64,
        flows: Vec<u64>,
        flows_end: Vec<u64>,
        force: bool,
        correlation_id: Option<u64>,
    ) -> Result<(), String> {
        self.encode_buffer.clear();
        let ts = self.convert_ts(ts);

        let track = self.get_mut_track(track_uuid);
        if track.active_slices.is_empty() && !force {
            return Ok(());
        }

        synthetto::slice_end_evt(
            track_uuid,
            ts,
            flows,
            flows_end,
            correlation_id,
            &mut self.encode_buffer,
        )
        .expect("prost encode should only fail if buffer is too small, but buffer is vec");
        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();

        let track = self.get_mut_track(track_uuid);
        track.active_slices.pop();
        Ok(())
    }

    pub fn instant_evt(
        &mut self,
        track_uuid: u64,
        ts: f64,
        name: Option<String>,
        flows: Vec<u64>,
        flows_end: Vec<u64>,
        correlation_id: Option<u64>,
    ) -> Result<(), String> {
        self.encode_buffer.clear();
        let ts = self.convert_ts(ts);
        synthetto::instant_evt(
            track_uuid,
            ts,
            name,
            flows,
            flows_end,
            correlation_id,
            &mut self.encode_buffer,
        )
        .expect("prost encode should only fail if buffer is too small, but buffer is vec");
        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();
        Ok(())
    }

    pub fn counter_evt(
        &mut self,
        track_uuid: u64,
        ts: u64,
        value: CounterValue,
        compress: bool,
    ) -> Result<(), String> {
        if compress
            && let Some(counter) = self.counters.get(&track_uuid)
            && counter.last_value == value
        {
            return Ok(());
        }

        self.encode_buffer.clear();
        match value {
            CounterValue::Int(val) => {
                synthetto::int_counter_evt(track_uuid, ts, val, &mut self.encode_buffer).expect(
                    "prost encode should only fail if buffer is too small, but buffer is vec",
                );
            }
            CounterValue::Float(val) => {
                synthetto::float_counter_evt(track_uuid, ts, val, &mut self.encode_buffer).expect(
                    "prost encode should only fail if buffer is too small, but buffer is vec",
                );
            }
        }
        self.w
            .write_all(&self.encode_buffer)
            .map_err(ioerr_to_str)?;
        self.trim_encode_buffer();

        if compress {
            self.counters.insert(track_uuid, Counter::new(value));
        }

        Ok(())
    }
}
