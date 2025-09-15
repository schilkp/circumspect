use synthetto::ChildOrder;

use crate::{Context, CounterValue, ReplacementBehaviour, svdpi::svBit};
use std::{
    ffi::{CStr, c_char, c_double, c_int, c_uint, c_ulonglong, c_void},
    path::PathBuf,
    ptr::null_mut,
    str::Utf8Error,
    sync::Mutex,
};

// ==== Context Object Management ==============================================

// Type backing  cspect_ctx chandles
type CtxCHandle = Mutex<Context>;

#[unsafe(no_mangle)]
pub extern "C" fn cspect_new(
    trace_path: *const c_char,
    timescale: c_double,
    time_mult: c_uint,
) -> *mut c_void {
    match cspect_new_actual(trace_path, timescale, time_mult) {
        Ok(ctx) => Box::into_raw(ctx) as *mut c_void,
        Err(e) => {
            println!("cspect: {}", e);
            null_mut()
        }
    }
}

fn cspect_new_actual(
    trace_path: *const c_char,
    timescale: c_double,
    time_mult: c_uint,
) -> Result<Box<CtxCHandle>, String> {
    let trace_path = unsafe { recover_cstr(trace_path)? };
    let trace_path = PathBuf::from(trace_path);
    let ctx: Box<CtxCHandle> =
        Box::new(Mutex::new(Context::new(trace_path, timescale, time_mult)?));
    Ok(ctx)
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_finish(cspect_ctx: *mut c_void) -> c_int {
    // Re-introduce chandle objects into the rust memory model.
    if cspect_ctx.is_null() {
        println!("cspect: cspect_ctx is nullptr!");
        return 1;
    }
    let cspect_ctx: Box<Mutex<Context>> = unsafe { Box::from_raw(cspect_ctx as *mut CtxCHandle) };

    // Since this function also deletes the context, we don't have to
    // re-leak the context.
    match cspect_finish_actual(&cspect_ctx) {
        Ok(()) => 0,
        Err(e) => {
            println!("cspect: {}", e);
            1
        }
    }
}

fn cspect_finish_actual(ctx: &Mutex<Context>) -> Result<(), String> {
    let mut ctx = ctx.lock().unwrap();
    ctx.flush()
}

// ==== Object Functions =======================================================

// DPI wrapper function body for functions with return type Result<(), String>
// Note: Only generates function body. Generating the whole function would
// easily be possibly but confuses cbindgen.
macro_rules! object_function_body_err_ret {
    ($func:ident, $ctx:ident $(, $arg:expr)* $(,)?) => {{
        // Re-introduce chandle objects into the rust memory model.
        if $ctx.is_null() {
            println!("cspect: cspect_ctx is nullptr!");
            return 1;
        }
        let cspect_ctx: Box<Mutex<Context>> = unsafe { Box::from_raw($ctx as *mut CtxCHandle) };

        // Lock + call actual function:
        let result = {
          let mut ctx = cspect_ctx.lock().unwrap();
          match $func(&mut ctx $(, $arg)*) {
              Ok(_) => 0,
              Err(e) => {
                  println!("cspect: {}", e);
                  1
              }
          }
        };

        // Re-leak since we don't own the memory.
        let _ = Box::into_raw(cspect_ctx);
        result
    }}
}

// DPI wrapper function body for functions with return type Result<u64, String>
// Note: Only generates function body. Generating the whole function would
// easily be possibly but confuses cbindgen.
macro_rules! object_function_body_uuid_ret {
    ($func:ident, $ctx:ident $(, $arg:expr)* $(,)?) => {{
        // Re-introduce chandle objects into the rust memory model.
        if $ctx.is_null() {
            println!("cspect: cspect_ctx is nullptr!");
            return 1;
        }
        let cspect_ctx: Box<Mutex<Context>> = unsafe { Box::from_raw($ctx as *mut CtxCHandle) };

        // Lock + call actual function:
        let result = {
          let mut ctx = cspect_ctx.lock().unwrap();
          match $func(&mut ctx $(, $arg)*) {
              Ok(v) => v,
              Err(e) => {
                  println!("cspect: {}", e);
                  0
              }
          }
        };

        // Re-leak since we don't own the memory.
        let _ = Box::into_raw(cspect_ctx);
        result
    }}
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_flush(cspect_ctx: *mut c_void) -> c_int {
    object_function_body_err_ret!(cspect_flush_actual, cspect_ctx)
}

fn cspect_flush_actual(ctx: &mut Context) -> Result<(), String> {
    ctx.flush()
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_new_flow(cspect_ctx: *mut c_void) -> c_ulonglong {
    object_function_body_uuid_ret!(cspect_new_flow_actual, cspect_ctx)
}

fn cspect_new_flow_actual(ctx: &mut Context) -> Result<u64, String> {
    Ok(ctx.new_flow())
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_new_track(
    cspect_ctx: *mut c_void,
    name: *const c_char,
    parent_uuid: c_ulonglong,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> c_ulonglong {
    object_function_body_uuid_ret!(
        cspect_new_track_actual,
        cspect_ctx,
        name,
        parent_uuid,
        description,
        child_ordering,
        child_order_rank
    )
}

fn cspect_new_track_actual(
    ctx: &mut Context,
    name: *const c_char,
    parent_uuid: c_ulonglong,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> Result<u64, String> {
    let name = unsafe { recover_cstr(name)?.to_string() };
    let parent_uuid = recover_optional_uuid(parent_uuid);
    let description = unsafe { recover_optional_cstr(description)?.map(String::from) };
    let child_ordering = recover_child_ordering(child_ordering)?;
    let child_order_rank = recover_optional_i32(child_order_rank);
    ctx.new_track(
        name,
        parent_uuid,
        description,
        child_ordering,
        child_order_rank,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_slice_begin(
    cspect_ctx: *mut c_void,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    replacement_behaviour: c_int,
) -> c_int {
    object_function_body_err_ret!(
        cspect_slice_begin_actual,
        cspect_ctx,
        parent_uuid,
        ts,
        name,
        flow1,
        flow2,
        flow3,
        flow_end1,
        flow_end2,
        flow_end3,
        replacement_behaviour,
    )
}

fn cspect_slice_begin_actual(
    ctx: &mut Context,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    replacement_behaviour: c_int,
) -> Result<(), String> {
    let parent_uuid = recover_required_uuid(parent_uuid)?;
    let ts: f64 = ts;
    let name = unsafe { recover_optional_cstr(name)?.map(String::from) };
    let replace_behaviour = recover_replacement_behaviour(replacement_behaviour)?;
    let mut flows = vec![];
    if let Some(x) = recover_optional_uuid(flow1) {
        flows.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow2) {
        flows.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow3) {
        flows.push(x)
    }
    let mut flows_end = vec![];
    if let Some(x) = recover_optional_uuid(flow_end1) {
        flows_end.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow_end2) {
        flows_end.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow_end3) {
        flows_end.push(x)
    }
    ctx.slice_begin_evt(parent_uuid, ts, name, flows, flows_end, replace_behaviour)
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_slice_end(
    cspect_ctx: *mut c_void,
    parent_uuid: c_ulonglong,
    ts: c_double,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    force: svBit,
) -> c_int {
    object_function_body_err_ret!(
        cspect_slice_end_actual,
        cspect_ctx,
        parent_uuid,
        ts,
        flow1,
        flow2,
        flow3,
        flow_end1,
        flow_end2,
        flow_end3,
        force
    )
}

fn cspect_slice_end_actual(
    ctx: &mut Context,
    parent_uuid: c_ulonglong,
    ts: c_double,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    force: svBit,
) -> Result<(), String> {
    let parent_uuid = recover_required_uuid(parent_uuid)?;
    let ts: f64 = ts;
    let mut flows = vec![];
    if let Some(x) = recover_optional_uuid(flow1) {
        flows.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow2) {
        flows.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow3) {
        flows.push(x)
    }
    let mut flows_end = vec![];
    if let Some(x) = recover_optional_uuid(flow_end1) {
        flows_end.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow_end2) {
        flows_end.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow_end3) {
        flows_end.push(x)
    }
    let force = recover_bool(force);
    ctx.slice_end_evt(parent_uuid, ts, flows, flows_end, force)
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_instant_evt(
    cspect_ctx: *mut c_void,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
) -> c_int {
    object_function_body_err_ret!(
        cspect_instant_evt_actual,
        cspect_ctx,
        parent_uuid,
        ts,
        name,
        flow1,
        flow2,
        flow3,
        flow_end1,
        flow_end2,
        flow_end3,
    )
}

fn cspect_instant_evt_actual(
    ctx: &mut Context,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
) -> Result<(), String> {
    let parent_uuid = recover_required_uuid(parent_uuid)?;
    let ts: f64 = ts;
    let name = unsafe { recover_optional_cstr(name)?.map(String::from) };
    let mut flows = vec![];
    if let Some(x) = recover_optional_uuid(flow1) {
        flows.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow2) {
        flows.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow3) {
        flows.push(x)
    }
    let mut flows_end = vec![];
    if let Some(x) = recover_optional_uuid(flow_end1) {
        flows_end.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow_end2) {
        flows_end.push(x)
    }
    if let Some(x) = recover_optional_uuid(flow_end3) {
        flows_end.push(x)
    }
    ctx.instant_evt(parent_uuid, ts, name, flows, flows_end)
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_new_process(
    cspect_ctx: *mut c_void,
    pid: c_int,
    process_name: *const c_char,
    cmdline: *const c_char,
    prio: c_int,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> c_ulonglong {
    object_function_body_uuid_ret!(
        cspect_new_process_actual,
        cspect_ctx,
        pid,
        process_name,
        cmdline,
        prio,
        description,
        child_ordering,
        child_order_rank
    )
}

fn cspect_new_process_actual(
    ctx: &mut Context,
    pid: c_int,
    process_name: *const c_char,
    cmdline: *const c_char,
    priority: c_int,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> Result<u64, String> {
    let process_name = unsafe { recover_optional_cstr(process_name)?.map(String::from) };
    let cmdline = unsafe { recover_optional_cstr(cmdline) }?
        .map(|x| vec![x.to_string()])
        .unwrap_or(vec![]);
    let priority = recover_optional_i32(priority);
    let description = unsafe { recover_optional_cstr(description)?.map(String::from) };
    let child_ordering = recover_child_ordering(child_ordering)?;
    let child_order_rank = recover_optional_i32(child_order_rank);
    ctx.new_process(
        pid,
        process_name,
        cmdline,
        priority,
        description,
        child_ordering,
        child_order_rank,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_new_thread(
    cspect_ctx: *mut c_void,
    pid: c_int,
    tid: c_int,
    thread_name: *const c_char,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> c_ulonglong {
    object_function_body_uuid_ret!(
        cspect_new_thread_actual,
        cspect_ctx,
        pid,
        tid,
        thread_name,
        description,
        child_ordering,
        child_order_rank
    )
}

fn cspect_new_thread_actual(
    ctx: &mut Context,
    pid: c_int,
    tid: c_int,
    thread_name: *const c_char,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> Result<u64, String> {
    let thread_name = unsafe { recover_cstr(thread_name)?.to_string() };
    let description = unsafe { recover_optional_cstr(description)?.map(String::from) };
    let child_ordering = recover_child_ordering(child_ordering)?;
    let child_order_rank = recover_optional_i32(child_order_rank);

    ctx.new_thread(
        pid,
        tid,
        thread_name,
        description,
        child_ordering,
        child_order_rank,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_new_counter(
    cspect_ctx: *mut c_void,
    name: *const c_char,
    unit_name: *const c_char,
    is_incremental: svBit,
    parent_uuid: c_ulonglong,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> c_ulonglong {
    object_function_body_uuid_ret!(
        cspect_new_counter_actual,
        cspect_ctx,
        name,
        unit_name,
        is_incremental,
        parent_uuid,
        description,
        child_ordering,
        child_order_rank
    )
}

fn cspect_new_counter_actual(
    ctx: &mut Context,
    name: *const c_char,
    unit_name: *const c_char,
    is_incremental: svBit,
    parent_uuid: c_ulonglong,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> Result<u64, String> {
    let name = unsafe { recover_cstr(name)?.to_string() };
    let unit_name = unsafe { recover_optional_cstr(unit_name)?.map(String::from) };
    let is_incremental = recover_bool(is_incremental);
    let parent_uuid = recover_optional_uuid(parent_uuid);
    let description = unsafe { recover_optional_cstr(description)?.map(String::from) };
    let child_ordering = recover_child_ordering(child_ordering)?;
    let child_order_rank = recover_optional_i32(child_order_rank);

    ctx.new_counter(
        name,
        unit_name,
        is_incremental,
        parent_uuid,
        description,
        child_ordering,
        child_order_rank,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_int_counter_evt(
    cspect_ctx: *mut c_void,
    track_uuid: c_ulonglong,
    ts: c_double,
    val: c_ulonglong,
    compress: svBit,
) -> c_int {
    object_function_body_err_ret!(
        cspect_int_counter_evt_actual,
        cspect_ctx,
        track_uuid,
        ts,
        val,
        compress
    )
}

fn cspect_int_counter_evt_actual(
    ctx: &mut Context,
    track_uuid: c_ulonglong,
    ts: c_double,
    val: c_ulonglong,
    compress: svBit,
) -> Result<(), String> {
    let track_uuid = recover_required_uuid(track_uuid)?;
    let ts = ctx.convert_ts(ts);
    let val = CounterValue::Int(val as i64);
    let compress = recover_bool(compress);
    ctx.counter_evt(track_uuid, ts, val, compress)
}

#[unsafe(no_mangle)]
pub extern "C" fn cspect_float_counter_evt(
    cspect_ctx: *mut c_void,
    track_uuid: c_ulonglong,
    ts: c_double,
    val: c_double,
    compress: svBit,
) -> c_int {
    object_function_body_err_ret!(
        cspect_float_counter_evt_actual,
        cspect_ctx,
        track_uuid,
        ts,
        val,
        compress
    )
}

fn cspect_float_counter_evt_actual(
    ctx: &mut Context,
    track_uuid: c_ulonglong,
    ts: c_double,
    val: c_double,
    compress: svBit,
) -> Result<(), String> {
    let track_uuid = recover_required_uuid(track_uuid)?;
    let ts = ctx.convert_ts(ts);
    let val = CounterValue::Float(val);
    let compress = recover_bool(compress);
    ctx.counter_evt(track_uuid, ts, val, compress)
}

// ==== Utils ==================================================================

fn utf8err_to_str(e: Utf8Error) -> String {
    format!("Failed to decode UTF8 string - {e}")
}

unsafe fn recover_cstr<'a>(cstr: *const c_char) -> Result<&'a str, String> {
    if cstr.is_null() {
        return Err("trace_path string is nullptr!".to_string());
    }
    unsafe { CStr::from_ptr(cstr).to_str().map_err(utf8err_to_str) }
}

unsafe fn recover_optional_cstr<'a>(cstr: *const c_char) -> Result<Option<&'a str>, String> {
    if cstr.is_null() {
        return Ok(None);
    }
    let str = unsafe { CStr::from_ptr(cstr).to_str().map_err(utf8err_to_str)? };

    if str.is_empty() {
        Ok(None)
    } else {
        Ok(Some(str))
    }
}

fn recover_bool(val: svBit) -> bool {
    val != 0
}

fn recover_required_uuid(val: c_ulonglong) -> Result<u64, String> {
    match recover_optional_uuid(val) {
        Some(val) => Ok(val),
        None => Err(String::from("Required UUID is zero")),
    }
}

fn recover_optional_uuid(val: c_ulonglong) -> Option<u64> {
    if val == 0 { None } else { Some(val) }
}

fn recover_optional_i32(val: c_int) -> Option<i32> {
    if val == 0 { None } else { Some(val) }
}

fn recover_child_ordering(child_order: c_int) -> Result<Option<ChildOrder>, String> {
    match child_order {
        0 => Ok(None),
        1 => Ok(Some(ChildOrder::Lexicographic)),
        2 => Ok(Some(ChildOrder::Chronological)),
        3 => Ok(Some(ChildOrder::Explicit)),
        i => Err(format!("invalid child ordering {i}")),
    }
}

fn recover_replacement_behaviour(
    replacement_behaviour: c_int,
) -> Result<ReplacementBehaviour, String> {
    match replacement_behaviour {
        0 => Ok(ReplacementBehaviour::NewSlice),
        1 => Ok(ReplacementBehaviour::Replace),
        2 => Ok(ReplacementBehaviour::ReplaceIfDifferent),
        i => Err(format!("invalid replacement behaviour {i}")),
    }
}
