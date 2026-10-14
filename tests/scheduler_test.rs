use hivehq::error::HiveError;
use hivehq::runtime::scheduler::Scheduler;
use hivehq::runtime::manager::Manager;
use hivehq::runtime::load_exec::ExecPageManager;
use crossbeam::queue::ArrayQueue;
use parking_lot::RwLock;
use std::sync::Arc;

/// Helper: construct a minimal scheduler for testing.
/// This still requires a loaded file for the process queue.
fn make_test_scheduler() -> Scheduler<std::fs::File> {
    // TODO: Build a manager with a minimal loaded file.
    unimplemented!("requires a valid .hive file fixture")
}

#[test]
#[ignore = "requires test fixture"]
fn scheduler_new_allocates_stack() {
    let sched = make_test_scheduler();
    assert!(sched.stack.size > 0);
}

#[test]
#[ignore = "requires test fixture"]
fn scheduler_run_processes_queue() {
    // Push a process into the queue, run the scheduler, and verify
    // the process executes and terminates.
}

#[test]
#[ignore = "requires test fixture"]
fn scheduler_waiting_fd_does_not_panic() {
    // This test would catch the todo!() panic in the WaitingFD branch.
    // Set a process state to WaitingFD, then run scheduler_run.
    // It should either park the process or return an error, not panic.
}

#[test]
#[ignore = "requires test fixture"]
fn scheduler_mail_send_recv() {
    // Spawn two processes, have one send a message to the other,
    // and verify the receiver gets it.
}

#[test]
#[ignore = "requires test fixture"]
fn scheduler_mail_peek_does_not_panic() {
    // This test would catch the todo!() panic in MailPeek.
}

#[test]
#[ignore = "requires test fixture"]
fn scheduler_process_exit_cleans_up() {
    // Verify that after a process exits, its stack frame is freed
    // and its PID is released.
}