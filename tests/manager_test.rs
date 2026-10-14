use crossbeam::queue::{ArrayQueue, SegQueue};
use hivehq::{
    error::HiveError,
    parser::LoadedFile,
    runtime::{
        manager::{Manager, PidSlot},
        scheduler::{
            Process, ProcessRegistryEntry, ProcessState,
            api::{ProcessRequest, ProcessRequestParam, ProcessRequestTag},
            context::HiveContext,
        },
    },
};
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
fn make_manager() -> Arc<Manager<std::fs::File>> {
    // Minimal manager: no exec page manager, no loaded file.
    Arc::new(Manager {
        exec_page_manager: RwLock::new(hivehq::runtime::load_exec::ExecPageManager::new(1)),
        pids: RwLock::new(Vec::new()),
        loaded_file: RwLock::new(LoadedFile::default()),
        process_queue: ArrayQueue::new(1024),
    })
}
fn make_process() -> Arc<Mutex<ProcessRegistryEntry>> {
    Arc::new(Mutex::new(ProcessRegistryEntry {
        mailbox: SegQueue::new(),
        pid: 0,
        process: Box::new(Process {
            context: HiveContext::default(),
            request: ProcessRequest {
                params: ProcessRequestParam { proc_exit: 0 },
                request: ProcessRequestTag::ProcExit,
            },
        }),
        stack_frames: Vec::new(),
        state: ProcessState::Running,
    }))
}

#[test]
fn allocate_pid_returns_first_free() {
    let manager = make_manager();
    let process = make_process();
    let pid0 = manager.allocate_pid(process.clone()).expect("pid 0");
    let pid1 = manager.allocate_pid(process).expect("pid 1");
    assert_eq!(pid0, 0);
    assert_eq!(pid1, 1);
}

#[test]
fn free_pid_marks_slot_unused() {
    let manager = make_manager();
    let process = make_process();
    let pid = manager.allocate_pid(process).expect("allocate");
    manager.free_pid(pid).expect("free");
    let pids = manager.pids.read();
    assert!(matches!(pids[pid as usize], PidSlot::Unused));
}

#[test]
fn free_pid_twice_errors() {
    let manager = make_manager();
    let process = make_process();
    let pid = manager.allocate_pid(process).expect("allocate");
    manager.free_pid(pid).expect("first free");
    let result = manager.free_pid(pid);
    assert!(matches!(result, Err(HiveError::PidAlreadyFree)));
}

#[test]
fn free_pid_unknown_errors() {
    let manager = make_manager();
    let result = manager.free_pid(999);
    assert!(matches!(result, Err(HiveError::PidNotFound)));
}

#[test]
fn pid_reuse_after_free() {
    let manager = make_manager();
    let process = make_process();
    let pid0 = manager.allocate_pid(process.clone()).expect("pid 0");
    manager.free_pid(pid0).expect("free pid 0");
    let pid1 = manager.allocate_pid(process).expect("pid 1");
    assert_eq!(pid0, pid1, "freed pid should be reused");
}

/// This test is designed to catch the TOCTOU race described in the
/// previous review. Run with `cargo test -- --test-threads=1` to
/// serialize, or use a barrier to force interleaving.
#[test]
fn concurrent_free_pid_no_double_free() {
    use std::sync::Barrier;
    use std::thread;

    let manager = make_manager();
    let pid = manager.allocate_pid(make_process()).expect("allocate");
    let barrier = Arc::new(Barrier::new(2));
    let m1 = Arc::clone(&manager);
    let m2 = Arc::clone(&manager);
    let b1 = Arc::clone(&barrier);
    let b2 = Arc::clone(&barrier);

    let t1 = thread::spawn(move || {
        b1.wait();
        m1.free_pid(pid)
    });
    let t2 = thread::spawn(move || {
        b2.wait();
        m2.free_pid(pid)
    });

    let r1 = t1.join().unwrap();
    let r2 = t2.join().unwrap();
    // Exactly one should succeed, the other should fail with
    // PidAlreadyFree. If both succeed, the TOCTOU race is present.
    let successes = [r1.is_ok(), r2.is_ok()].iter().filter(|&&x| x).count();
    assert_eq!(successes, 1, "exactly one free_pid should succeed");
}
