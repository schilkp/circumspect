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

#[derive(Debug, Default)]
struct Track {
    active_slice_count: u32,
}

#[derive(Debug)]
struct Context {
    w: BufWriter<File>,
    synthetto: Synthetto,
    timescale: f64,
    time_mult: u32,
    tracks: HashMap<u64, Track>,
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
        let mut data = Vec::with_capacity(64);

        let uuid = self
            .synthetto
            .new_track(
                name,
                parent_uuid,
                description,
                child_ordering,
                sibling_order_rank,
                &mut data,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w.write_all(&data).map_err(ioerr_to_str)?;

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
        let mut data = Vec::with_capacity(64);

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
                &mut data,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w.write_all(&data).map_err(ioerr_to_str)?;

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
        let mut data = Vec::with_capacity(64);

        let uuid = self
            .synthetto
            .new_thread(
                pid,
                tid,
                thread_name,
                description,
                child_ordering,
                sibling_order_rank,
                &mut data,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w.write_all(&data).map_err(ioerr_to_str)?;

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
        let mut data = Vec::with_capacity(64);

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
                &mut data,
            )
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w.write_all(&data).map_err(ioerr_to_str)?;

        Ok(uuid)
    }

    pub fn slice_begin_evt(
        &mut self,
        track_uuid: u64,
        ts: f64,
        name: Option<String>,
        flows: Vec<u64>,
        replace_previous_slice: bool,
    ) -> Result<(), String> {
        let mut did_replace_slice = false;
        if replace_previous_slice && self.get_mut_track(track_uuid).active_slice_count != 0 {
            self.slice_end_evt(track_uuid, ts, vec![])?;
            did_replace_slice = true;
        }

        let mut data = Vec::with_capacity(64);
        let ts = self.convert_ts(ts);
        synthetto::slice_begin_evt(track_uuid, ts, name, flows, &mut data)
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");
        self.w.write_all(&data).map_err(ioerr_to_str)?;

        if !did_replace_slice {
            self.get_mut_track(track_uuid).active_slice_count += 1;
        }
        Ok(())
    }

    pub fn slice_end_evt(
        &mut self,
        track_uuid: u64,
        ts: f64,
        flows: Vec<u64>,
    ) -> Result<(), String> {
        let mut data = Vec::with_capacity(64);
        let ts = self.convert_ts(ts);
        synthetto::slice_end_evt(track_uuid, ts, flows, &mut data)
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");
        self.w.write_all(&data).map_err(ioerr_to_str)?;

        let track = self.get_mut_track(track_uuid);
        if track.active_slice_count != 0 {
            track.active_slice_count -= 1;
        }
        Ok(())
    }

    pub fn instant_evt(
        &mut self,
        track_uuid: u64,
        ts: f64,
        name: Option<String>,
        flows: Vec<u64>,
    ) -> Result<(), String> {
        let mut data = Vec::with_capacity(64);
        let ts = self.convert_ts(ts);
        synthetto::instant_evt(track_uuid, ts, name, flows, &mut data)
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");
        self.w.write_all(&data).map_err(ioerr_to_str)
    }

    pub fn int_counter_evt(&mut self, track_uuid: u64, ts: u64, val: i64) -> Result<(), String> {
        let mut data = Vec::with_capacity(64);

        synthetto::int_counter_evt(track_uuid, ts, val, &mut data)
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w.write_all(&data).map_err(ioerr_to_str)
    }

    pub fn float_counter_evt(&mut self, track_uuid: u64, ts: u64, val: f64) -> Result<(), String> {
        let mut data = Vec::with_capacity(64);

        synthetto::float_counter_evt(track_uuid, ts, val, &mut data)
            .expect("prost encode should only fail if buffer is too small, but buffer is vec");

        self.w.write_all(&data).map_err(ioerr_to_str)
    }
}
