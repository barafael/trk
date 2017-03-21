mod timesheet;

fn main() {
    let ts = timesheet::Session::new();
    println!("{:?}", ts);
}
