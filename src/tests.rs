#[test]
fn test_drive() {
    let manger = crate::drive::ControlManger::new(crate::drive::LaunchMode::Debug);
    manger.launch();
}
