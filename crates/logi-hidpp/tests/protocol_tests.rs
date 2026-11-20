use logi_hidpp::features::{
    ChargingStatus, DeviceType, SmartShiftMode, SmartShiftSettings,
};
use logi_hidpp::{HidppDevice, HidppError, HidppReport, MockTransport};

#[test]
fn test_report_serialization_and_parsing() {
    let mut report = HidppReport::new_long(0x01, 0x05, 0x02, 0x01);
    report.payload[0] = 0xAA;
    report.payload[1] = 0xBB;

    let bytes = report.to_bytes();
    assert_eq!(bytes[0], 0x11); // Long report
    assert_eq!(bytes[1], 0x01); // Slot 1
    assert_eq!(bytes[2], 0x05); // Feature index 0x05
    assert_eq!(bytes[3], 0x21); // Function 2 (0x20) | Software ID 1 (0x01)
    assert_eq!(bytes[4], 0xAA);
    assert_eq!(bytes[5], 0xBB);

    let parsed = HidppReport::parse(&bytes).expect("Should parse valid report");
    assert_eq!(parsed.report_id, 0x11);
    assert_eq!(parsed.device_slot, 0x01);
    assert_eq!(parsed.feature_index, 0x05);
    assert_eq!(parsed.function_id, 0x02);
    assert_eq!(parsed.software_id, 0x01);
    assert_eq!(parsed.payload[0], 0xAA);
}

#[test]
fn test_error_report_parsing() {
    // 0x11, Slot 1, 0x8F (Error), Feature 0x05, Function 0x01, Error 0x02 (Invalid argument)
    let error_bytes = vec![0x11, 0x01, 0x8F, 0x05, 0x10, 0x02, 0x00];
    let err = HidppReport::parse(&error_bytes).unwrap_err();
    match err {
        HidppError::ProtocolError { code, feature_index, .. } => {
            assert_eq!(code, 0x02);
            assert_eq!(feature_index, 0x05);
        }
        _ => panic!("Expected ProtocolError, got {:?}", err),
    }
}

#[test]
fn test_device_offline_error_parsing() {
    // Error 0x09 is RF timeout / device sleep
    let error_bytes = vec![0x11, 0x01, 0x8F, 0x08, 0x10, 0x09, 0x00];
    let err = HidppReport::parse(&error_bytes).unwrap_err();
    assert!(matches!(err, HidppError::DeviceOffline));
}

#[test]
fn test_unified_battery_and_standby_retention() {
    let mut transport = MockTransport::new();
    let mut dev = HidppDevice::new(0x01);

    // 1. Register Root resolution for FEATURE_UNIFIED_BATTERY (0x1004) -> Index 0x08
    let root_req = vec![0x11, 0x01, 0x00, 0x01, 0x10, 0x04];
    let mut root_resp = vec![0u8; 20];
    root_resp[0] = 0x11;
    root_resp[1] = 0x01;
    root_resp[2] = 0x00;
    root_resp[3] = 0x01;
    root_resp[4] = 0x08; // Feature index 0x08
    transport.register_response(&root_req, &root_resp);

    // 2. Register Battery Status (Function 1 of Feature 0x08): 85% charging
    let batt_req = vec![0x11, 0x01, 0x08, 0x11];
    let mut batt_resp = vec![0u8; 20];
    batt_resp[0] = 0x11;
    batt_resp[1] = 0x01;
    batt_resp[2] = 0x08;
    batt_resp[3] = 0x11;
    batt_resp[4] = 85;   // 85%
    batt_resp[5] = 0x01; // Charging
    batt_resp[6] = 0x00; // Not critical
    transport.register_response(&batt_req, &batt_resp);

    // Query active battery
    let batt = dev.query_battery(&mut transport).expect("Should read battery");
    assert_eq!(batt.percentage, 85);
    assert!(batt.charging);
    assert_eq!(batt.status, ChargingStatus::Charging);
    assert!(!batt.standby);

    // 3. Now simulate peripheral entering RF deep sleep (DeviceOffline response)
    let sleep_resp = vec![0x11, 0x01, 0x8F, 0x08, 0x10, 0x09, 0x00];
    transport.register_response(&batt_req, &sleep_resp);

    let standby_batt = dev.query_battery(&mut transport).expect("Should retain last known battery in standby");
    assert_eq!(standby_batt.percentage, 85);
    assert!(!standby_batt.charging);
    assert_eq!(standby_batt.status, ChargingStatus::Standby);
    assert!(standby_batt.standby);
}

#[test]
fn test_device_name_and_type() {
    let mut transport = MockTransport::new();
    let mut dev = HidppDevice::new(0x01);

    // 1. Root query for FEATURE_DEVICE_NAME_TYPE (0x0005) -> Index 0x04
    let root_req = vec![0x11, 0x01, 0x00, 0x01, 0x00, 0x05];
    let mut root_resp = vec![0u8; 20];
    root_resp[0] = 0x11;
    root_resp[1] = 0x01;
    root_resp[2] = 0x00;
    root_resp[3] = 0x01;
    root_resp[4] = 0x04; // Index 0x04
    transport.register_response(&root_req, &root_resp);

    // 2. Name length (Function 0): 12 bytes
    let count_req = vec![0x11, 0x01, 0x04, 0x01];
    let mut count_resp = vec![0u8; 20];
    count_resp[0] = 0x11;
    count_resp[1] = 0x01;
    count_resp[2] = 0x04;
    count_resp[3] = 0x01;
    count_resp[4] = 12; // Length 12
    transport.register_response(&count_req, &count_resp);

    // 3. Name chunk (Function 1): "MX Master 3S"
    let name_req = vec![0x11, 0x01, 0x04, 0x11];
    let mut name_resp = vec![0u8; 20];
    name_resp[0] = 0x11;
    name_resp[1] = 0x01;
    name_resp[2] = 0x04;
    name_resp[3] = 0x11;
    name_resp[4..16].copy_from_slice(b"MX Master 3S");
    transport.register_response(&name_req, &name_resp);

    let name = dev.query_name(&mut transport).expect("Should get name");
    assert_eq!(name, "MX Master 3S");

    // 4. Device Type (Function 2): 3 (Mouse)
    let type_req = vec![0x11, 0x01, 0x04, 0x21];
    let mut type_resp = vec![0u8; 20];
    type_resp[0] = 0x11;
    type_resp[1] = 0x01;
    type_resp[2] = 0x04;
    type_resp[3] = 0x21;
    type_resp[4] = 3; // 3 = Mouse
    transport.register_response(&type_req, &type_resp);

    let dev_type = dev.query_device_type(&mut transport).expect("Should get type");
    assert_eq!(dev_type, DeviceType::Mouse);
}

#[test]
fn test_dpi_query_and_set() {
    let mut transport = MockTransport::new();
    let mut dev = HidppDevice::new(0x01);

    // Root query for ADJUSTABLE_DPI (0x2201) -> Index 0x09
    let root_req = vec![0x11, 0x01, 0x00, 0x01, 0x22, 0x01];
    let mut root_resp = vec![0u8; 20];
    root_resp[0] = 0x11;
    root_resp[1] = 0x01;
    root_resp[2] = 0x00;
    root_resp[3] = 0x01;
    root_resp[4] = 0x09;
    transport.register_response(&root_req, &root_resp);

    // Query current DPI (Function 2): 1600 (0x0640)
    let dpi_req = vec![0x11, 0x01, 0x09, 0x21];
    let mut dpi_resp = vec![0u8; 20];
    dpi_resp[0] = 0x11;
    dpi_resp[1] = 0x01;
    dpi_resp[2] = 0x09;
    dpi_resp[3] = 0x21;
    dpi_resp[4] = 0x00; // Sensor index 0
    dpi_resp[5] = 0x06; // 1600 high
    dpi_resp[6] = 0x40; // 1600 low
    transport.register_response(&dpi_req, &dpi_resp);

    let dpi_settings = dev.query_dpi(&mut transport).expect("Should query DPI");
    assert_eq!(dpi_settings.current, 1600);

    // Set DPI to 2400 (0x0960) (Function 3)
    let set_dpi_req = vec![0x11, 0x01, 0x09, 0x31, 0x00, 0x09, 0x60];
    let mut set_dpi_resp = vec![0u8; 20];
    set_dpi_resp[0] = 0x11;
    set_dpi_resp[1] = 0x01;
    set_dpi_resp[2] = 0x09;
    set_dpi_resp[3] = 0x31;
    transport.register_response(&set_dpi_req, &set_dpi_resp);

    dev.set_dpi(&mut transport, 2400).expect("Should set DPI");
}

#[test]
fn test_smartshift_query_and_set() {
    let mut transport = MockTransport::new();
    let mut dev = HidppDevice::new(0x01);

    // Root query for SMART_SHIFT (0x2110) -> Index 0x0A
    let root_req = vec![0x11, 0x01, 0x00, 0x01, 0x21, 0x10];
    let mut root_resp = vec![0u8; 20];
    root_resp[0] = 0x11;
    root_resp[1] = 0x01;
    root_resp[2] = 0x00;
    root_resp[3] = 0x01;
    root_resp[4] = 0x0A;
    transport.register_response(&root_req, &root_resp);

    // Query SmartShift (Function 0): AutoDisengage (0), threshold 32
    let ss_req = vec![0x11, 0x01, 0x0A, 0x01];
    let mut ss_resp = vec![0u8; 20];
    ss_resp[0] = 0x11;
    ss_resp[1] = 0x01;
    ss_resp[2] = 0x0A;
    ss_resp[3] = 0x01;
    ss_resp[4] = 0x00; // AutoDisengage
    ss_resp[5] = 32;   // Threshold
    transport.register_response(&ss_req, &ss_resp);

    let ss = dev.query_smartshift(&mut transport).expect("Should query SmartShift");
    assert_eq!(ss.mode, SmartShiftMode::AutoDisengage);
    assert_eq!(ss.threshold, 32);
    assert!(ss.enabled);

    // Set SmartShift (Function 1)
    let set_ss_req = vec![0x11, 0x01, 0x0A, 0x11, 0x01, 48];
    let mut set_ss_resp = vec![0u8; 20];
    set_ss_resp[0] = 0x11;
    set_ss_resp[1] = 0x01;
    set_ss_resp[2] = 0x0A;
    set_ss_resp[3] = 0x11;
    transport.register_response(&set_ss_req, &set_ss_resp);

    dev.set_smartshift(&mut transport, &SmartShiftSettings {
        enabled: true,
        threshold: 48,
        mode: SmartShiftMode::AlwaysRatchet,
    }).expect("Should set SmartShift");
}
