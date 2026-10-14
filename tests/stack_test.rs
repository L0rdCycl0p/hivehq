use hivehq::error::HiveError;
use hivehq::runtime::scheduler::stack::{FrameState, Stack};

#[test]
fn stack_new_rejects_zero_size() {
    let result = Stack::new(0);
    assert!(matches!(result, Err(HiveError::CanNotAllocateStack)));
}

#[test]
fn stack_new_allocates_valid_region() {
    let stack = Stack::new(4096).expect("stack should allocate");
    assert!(!stack.start.is_null());
    assert!(!stack.end.is_null());
    assert_eq!(stack.size, 4096);
    assert_eq!(stack.frames.len(), 1);
    assert!(matches!(stack.frames[0].state, FrameState::Free));
}

#[test]
fn stack_allocate_returns_correct_region() {
    let mut stack = Stack::new(4096).expect("stack should allocate");
    let region = stack.allocate(1, 1024).expect("allocate should succeed");
    assert_eq!(region.state, FrameState::Used(1));
    assert_eq!(stack.frames.len(), 2); // original split
    assert!(matches!(stack.frames[0].state, FrameState::Used(1)));
    assert!(matches!(stack.frames[1].state, FrameState::Free));
}

#[test]
fn stack_allocate_exhaustion() {
    let mut stack = Stack::new(1024).expect("stack should allocate");
    let _ = stack.allocate(1, 1024).expect("first allocate");
    let result = stack.allocate(2, 1024);
    assert!(matches!(result, Err(HiveError::StackExhausted)));
}

#[test]
fn stack_free_marks_frame_as_free() {
    let mut stack = Stack::new(4096).expect("stack should allocate");
    let region = stack.allocate(1, 1024).expect("allocate");
    stack.free(region.start).expect("free should succeed");
    assert!(matches!(stack.frames[0].state, FrameState::Free));
}

#[test]
fn stack_free_unknown_frame_errors() {
    let mut stack = Stack::new(4096).expect("stack should allocate");
    let result = stack.free(std::ptr::null_mut());
    assert!(matches!(
        result,
        Err(HiveError::AttemptedToFreeUnknownStackFrame)
    ));
}

#[test]
fn stack_compact_merges_adjacent_free_frames() {
    let mut stack = Stack::new(4096).expect("stack should allocate");
    let r1 = stack.allocate(1, 1024).expect("alloc 1");
    let r2 = stack.allocate(2, 1024).expect("alloc 2");
    let _r3 = stack.allocate(3, 1024).expect("alloc 3");
    stack.free(r1.start).expect("free 1");
    stack.free(r2.start).expect("free 2");
    stack.compact();
    // After compaction, the two adjacent free frames should merge.
    let free_count = stack
        .frames
        .iter()
        .filter(|f| matches!(f.state, FrameState::Free))
        .count();
    assert!(free_count <= 2, "adjacent free frames should be merged");
}

#[test]
fn stack_fragmentation_resolved_by_compact() {
    let mut stack = Stack::new(4096).expect("stack should allocate");
    let r1 = stack.allocate(1, 1024).expect("alloc 1");
    let r2 = stack.allocate(2, 1024).expect("alloc 2");
    let _r3 = stack.allocate(3, 1024).expect("alloc 3");
    stack.free(r1.start).expect("free 1");
    stack.free(r2.start).expect("free 2");

    stack.compact();

    let _r4 = stack.allocate(4, 2048).expect("alloc 4 after compact");
}

#[test]
fn stack_allocate_larger_than_remaining() {
    let mut stack = Stack::new(1024).expect("stack should allocate");
    let result = stack.allocate(1, 2048);
    assert!(matches!(result, Err(HiveError::StackExhausted)));
}