use logi_daemon::gesture_engine::{GestureDirection, GestureTracker};
use logi_daemon::ipc::JsonRpcRequest;
use logi_daemon::virtual_input::VirtualInputDevice;

#[test]
fn test_gesture_engine_vectors() {
    let mut tracker = GestureTracker::new(40);
    tracker.on_button_down();

    // Small motion should not trigger
    assert_eq!(tracker.on_motion(10, 10), None);

    // Large horizontal swipe left: 10 + (-55) = -45 (exceeds threshold 40)
    assert_eq!(tracker.on_motion(-55, 0), Some(GestureDirection::Left));

    // Release after swipe
    assert_eq!(tracker.on_button_up(), GestureDirection::None);
}

#[test]
fn test_gesture_engine_tap_click() {
    let mut tracker = GestureTracker::new(40);
    tracker.on_button_down();
    // No significant motion
    assert_eq!(tracker.on_motion(2, 1), None);
    // Button release should report Click
    assert_eq!(tracker.on_button_up(), GestureDirection::Click);
}

#[test]
fn test_json_rpc_parsing() {
    let req_str = r#"{"jsonrpc": "2.0", "id": 42, "method": "ping"}"#;
    let req: JsonRpcRequest = serde_json::from_str(req_str).expect("Valid JSON-RPC");
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "ping");
    assert_eq!(req.id, Some(serde_json::json!(42)));
}

#[test]
fn test_virtual_input_creation_and_events() {
    if let Ok(mut uinput) = VirtualInputDevice::new("Test Virtual Device") {
        // Emit a key click (e.g. KEY_LEFTMETA = 125)
        let res = uinput.emit_click(125);
        assert!(res.is_ok(), "Should emit click without error");

        // Emit relative scroll wheel
        let rel_res = uinput.emit_rel(0x08, 1);
        assert!(rel_res.is_ok(), "Should emit relative scroll");
    }
}
