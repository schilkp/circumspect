use synthetto::ChildOrder;

use crate::{svdpi::svBit, Context, CounterValue, ReplacementBehaviour};
use std::{
    ffi::{c_char, c_double, c_int, c_uint, c_ulonglong, c_void, CStr},
    path::PathBuf,
    ptr::null_mut,
    str::Utf8Error,
    sync::Mutex,
};

// ==== UUID Vector Object =====================================================

// Type backing  uuid_arr chandles
type UUIDVecCHandle = Mutex<Vec<u64>>;

#[no_mangle]
pub extern "C" fn cspect_dpi_uuid_vec_new(
    uuid0: c_ulonglong,
    uuid1: c_ulonglong,
    uuid2: c_ulonglong,
    uuid3: c_ulonglong,
) -> *mut c_void {
    let mut vec = Vec::with_capacity(8);
    for uuid in [uuid0, uuid1, uuid2, uuid3] {
        if let Some(uuid) = recover_optional_uuid(uuid) {
            vec.push(uuid);
        }
    }
    let handle = Box::new(Mutex::new(vec));
    Box::into_raw(handle) as *mut c_void
}

#[no_mangle]
pub extern "C" fn cspect_dpi_uuid_vec_append(
    uuid_vec: *mut c_void,
    uuid0: c_ulonglong,
    uuid1: c_ulonglong,
    uuid2: c_ulonglong,
    uuid3: c_ulonglong,
) -> c_int {
    // Re-introduce chandle objects into the rust memory model.
    if uuid_vec.is_null() {
        println!("cspect: uuid_vec is nullptr!");
        return 1;
    }

    let vec: Box<Mutex<Vec<u64>>> = unsafe { Box::from_raw(uuid_vec as *mut UUIDVecCHandle) };

    {
        let mut vec = vec.lock().unwrap();
        for uuid in [uuid0, uuid1, uuid2, uuid3] {
            if let Some(uuid) = recover_optional_uuid(uuid) {
                vec.push(uuid);
            }
        }
        drop(vec); // re-lock
    }

    // Don't keep ownership:
    let _ = Box::into_raw(vec) as *mut c_void;

    0
}

#[no_mangle]
pub extern "C" fn cspect_dpi_uuid_vec_delete(uuid_vec: *mut c_void) -> c_int {
    if uuid_vec.is_null() {
        println!("cspect: uuid_vec is nullptr!");
        return 1;
    }

    // re-introduce into rust memroy model and drop it:
    let vec: Box<Mutex<Vec<u64>>> = unsafe { Box::from_raw(uuid_vec as *mut UUIDVecCHandle) };
    drop(vec);

    0
}

// ==== Context Object Management ==============================================

// Type backing  cspect_ctx chandles
type CtxCHandle = Mutex<Context>;

#[no_mangle]
pub extern "C" fn cspect_dpi_new(
    trace_path: *const c_char,
    timescale: c_double,
    time_mult: c_uint,
) -> *mut c_void {
    match cspect_new(trace_path, timescale, time_mult) {
        Ok(ctx) => Box::into_raw(ctx) as *mut c_void,
        Err(e) => {
            println!("cspect: {}", e);
            null_mut()
        }
    }
}

fn cspect_new(
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

#[no_mangle]
pub extern "C" fn cspect_dpi_finish(cspect_ctx: *mut c_void) -> c_int {
    // Re-introduce chandle objects into the rust memory model.
    if cspect_ctx.is_null() {
        println!("cspect: cspect_ctx is nullptr!");
        return 1;
    }
    let cspect_ctx: Box<Mutex<Context>> = unsafe { Box::from_raw(cspect_ctx as *mut CtxCHandle) };

    // Since this function also deletes the context, we don't have to
    // re-leak the context.
    match cspect_finish(&cspect_ctx) {
        Ok(()) => 0,
        Err(e) => {
            println!("cspect: {}", e);
            1
        }
    }
}

fn cspect_finish(ctx: &Mutex<Context>) -> Result<(), String> {
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

#[no_mangle]
pub extern "C" fn cspect_dpi_flush(cspect_ctx: *mut c_void) -> c_int {
    object_function_body_err_ret!(cspect_flush, cspect_ctx)
}

fn cspect_flush(ctx: &mut Context) -> Result<(), String> {
    ctx.flush()
}

#[no_mangle]
pub extern "C" fn cspect_dpi_new_uuid(cspect_ctx: *mut c_void) -> c_ulonglong {
    object_function_body_uuid_ret!(cspect_new_uuid, cspect_ctx)
}

fn cspect_new_uuid(ctx: &mut Context) -> Result<u64, String> {
    Ok(ctx.new_uuid())
}

#[no_mangle]
pub extern "C" fn cspect_dpi_new_track(
    cspect_ctx: *mut c_void,
    name: *const c_char,
    parent_uuid: c_ulonglong,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> c_ulonglong {
    object_function_body_uuid_ret!(
        cspect_new_track,
        cspect_ctx,
        name,
        parent_uuid,
        description,
        child_ordering,
        child_order_rank
    )
}

fn cspect_new_track(
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

#[no_mangle]
pub extern "C" fn cspect_dpi_slice_begin(
    cspect_ctx: *mut c_void,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow0: c_ulonglong,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_others: *mut c_void,
    flow_end0: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    flow_end_others: *mut c_void,
    replacement_behaviour: c_int,
    correlation_id: c_ulonglong,
) -> c_int {
    object_function_body_err_ret!(
        cspect_slice_begin,
        cspect_ctx,
        parent_uuid,
        ts,
        name,
        flow0,
        flow1,
        flow2,
        flow3,
        flow_others,
        flow_end0,
        flow_end1,
        flow_end2,
        flow_end3,
        flow_end_others,
        replacement_behaviour,
        correlation_id,
    )
}

fn cspect_slice_begin(
    ctx: &mut Context,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow0: c_ulonglong,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_others: *mut c_void,
    flow_end0: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    flow_end_others: *mut c_void,
    replacement_behaviour: c_int,
    correlation_id: c_ulonglong,
) -> Result<(), String> {
    let parent_uuid = recover_required_uuid(parent_uuid)?;
    let ts: f64 = ts;
    let name = unsafe { recover_optional_cstr(name)?.map(String::from) };
    let replace_behaviour = recover_replacement_behaviour(replacement_behaviour)?;
    let flows = recover_uuid_vec(flow0, flow1, flow2, flow3, flow_others);
    let flows_end = recover_uuid_vec(flow_end0, flow_end1, flow_end2, flow_end3, flow_end_others);
    let correlation_id = recover_optional_uuid(correlation_id);
    ctx.slice_begin_evt(
        parent_uuid,
        ts,
        name,
        flows,
        flows_end,
        replace_behaviour,
        correlation_id,
    )
}

#[no_mangle]
pub extern "C" fn cspect_dpi_slice_end(
    cspect_ctx: *mut c_void,
    parent_uuid: c_ulonglong,
    ts: c_double,
    flow0: c_ulonglong,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_others: *mut c_void,
    flow_end0: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    flow_end_others: *mut c_void,
    force: svBit,
    correlation_id: c_ulonglong,
) -> c_int {
    object_function_body_err_ret!(
        cspect_slice_end,
        cspect_ctx,
        parent_uuid,
        ts,
        flow0,
        flow1,
        flow2,
        flow3,
        flow_others,
        flow_end0,
        flow_end1,
        flow_end2,
        flow_end3,
        flow_end_others,
        force,
        correlation_id,
    )
}

fn cspect_slice_end(
    ctx: &mut Context,
    parent_uuid: c_ulonglong,
    ts: c_double,
    flow0: c_ulonglong,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_others: *mut c_void,
    flow_end0: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    flow_end_others: *mut c_void,
    force: svBit,
    correlation_id: c_ulonglong,
) -> Result<(), String> {
    let parent_uuid = recover_required_uuid(parent_uuid)?;
    let ts: f64 = ts;
    let flows = recover_uuid_vec(flow0, flow1, flow2, flow3, flow_others);
    let flows_end = recover_uuid_vec(flow_end0, flow_end1, flow_end2, flow_end3, flow_end_others);
    let force = recover_bool(force);
    let correlation_id = recover_optional_uuid(correlation_id);
    ctx.slice_end_evt(parent_uuid, ts, flows, flows_end, force, correlation_id)
}

#[no_mangle]
pub extern "C" fn cspect_dpi_instant_evt(
    cspect_ctx: *mut c_void,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow0: c_ulonglong,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_others: *mut c_void,
    flow_end0: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    flow_end_others: *mut c_void,
    correlation_id: c_ulonglong,
) -> c_int {
    object_function_body_err_ret!(
        cspect_instant_evt,
        cspect_ctx,
        parent_uuid,
        ts,
        name,
        flow0,
        flow1,
        flow2,
        flow3,
        flow_others,
        flow_end0,
        flow_end1,
        flow_end2,
        flow_end3,
        flow_end_others,
        correlation_id,
    )
}

fn cspect_instant_evt(
    ctx: &mut Context,
    parent_uuid: c_ulonglong,
    ts: c_double,
    name: *const c_char,
    flow0: c_ulonglong,
    flow1: c_ulonglong,
    flow2: c_ulonglong,
    flow3: c_ulonglong,
    flow_others: *mut c_void,
    flow_end0: c_ulonglong,
    flow_end1: c_ulonglong,
    flow_end2: c_ulonglong,
    flow_end3: c_ulonglong,
    flow_end_others: *mut c_void,
    correlation_id: c_ulonglong,
) -> Result<(), String> {
    let parent_uuid = recover_required_uuid(parent_uuid)?;
    let ts: f64 = ts;
    let name = unsafe { recover_optional_cstr(name)?.map(String::from) };
    let flows = recover_uuid_vec(flow0, flow1, flow2, flow3, flow_others);
    let flows_end = recover_uuid_vec(flow_end0, flow_end1, flow_end2, flow_end3, flow_end_others);
    let correlation_id = recover_optional_uuid(correlation_id);
    ctx.instant_evt(parent_uuid, ts, name, flows, flows_end, correlation_id)
}

#[no_mangle]
pub extern "C" fn cspect_dpi_new_process(
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
        cspect_new_process,
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

fn cspect_new_process(
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

#[no_mangle]
pub extern "C" fn cspect_dpi_new_thread(
    cspect_ctx: *mut c_void,
    pid: c_int,
    tid: c_int,
    thread_name: *const c_char,
    description: *const c_char,
    child_ordering: c_int,
    child_order_rank: c_int,
) -> c_ulonglong {
    object_function_body_uuid_ret!(
        cspect_new_thread,
        cspect_ctx,
        pid,
        tid,
        thread_name,
        description,
        child_ordering,
        child_order_rank
    )
}

fn cspect_new_thread(
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

#[no_mangle]
pub extern "C" fn cspect_dpi_new_counter(
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
        cspect_new_counter,
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

fn cspect_new_counter(
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

#[no_mangle]
pub extern "C" fn cspect_dpi_int_counter_evt(
    cspect_ctx: *mut c_void,
    track_uuid: c_ulonglong,
    ts: c_double,
    val: c_ulonglong,
    compress: svBit,
) -> c_int {
    object_function_body_err_ret!(
        cspect_int_counter_evt,
        cspect_ctx,
        track_uuid,
        ts,
        val,
        compress
    )
}

fn cspect_int_counter_evt(
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

#[no_mangle]
pub extern "C" fn cspect_dpi_float_counter_evt(
    cspect_ctx: *mut c_void,
    track_uuid: c_ulonglong,
    ts: c_double,
    val: c_double,
    compress: svBit,
) -> c_int {
    object_function_body_err_ret!(
        cspect_float_counter_evt,
        cspect_ctx,
        track_uuid,
        ts,
        val,
        compress
    )
}

fn cspect_float_counter_evt(
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

fn recover_optional_i32(val: c_int) -> Option<i32> {
    if val == 0 {
        None
    } else {
        Some(val)
    }
}

fn recover_required_uuid(val: c_ulonglong) -> Result<u64, String> {
    match recover_optional_uuid(val) {
        Some(val) => Ok(val),
        None => Err(String::from("Required UUID is zero")),
    }
}

fn recover_optional_uuid(val: c_ulonglong) -> Option<u64> {
    if val == 0 {
        None
    } else {
        Some(val)
    }
}

fn recover_uuid_vec(
    uuid0: c_ulonglong,
    uuid1: c_ulonglong,
    uuid2: c_ulonglong,
    uuid3: c_ulonglong,
    vec_handle: *mut c_void,
) -> Vec<u64> {
    let mut v = vec![];

    for uuid in [uuid0, uuid1, uuid2, uuid3] {
        if let Some(uuid) = recover_optional_uuid(uuid) {
            v.push(uuid);
        }
    }

    if !vec_handle.is_null() {
        let vec: Box<Mutex<Vec<u64>>> = unsafe { Box::from_raw(vec_handle as *mut UUIDVecCHandle) };
        {
            let vec = vec.lock().unwrap();
            for val in vec.iter() {
                v.push(*val);
            }
            drop(vec); // re-lock
        }
        // Don't keep ownership:
        let _ = Box::into_raw(vec) as *mut c_void;
    }

    v
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
