#[test]
fn test_drive() {
    let mut manger = crate::drive::ControlManger::new(crate::drive::LaunchMode::Debug);
    manger.launch();
}
