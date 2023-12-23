mod drive;
// mod metal_sensor;
mod tests;
mod view;

fn main() {
    let mut a = drive::ControlManger::new(drive::LaunchMode::Debug);
    a.launch();
    println!("Hello, world!");
}
