use std::thread;

#[derive(Debug)]
struct Data {
    value: u32,
}

fn main() {
    let mut data = Data { value: 0 };
    let data = &mut data;

    let work_0 = || (0..50000).for_each(|_| data.value += 1);
    let work_1 = || (0..50000).for_each(|_| data.value += 1);

    let thread_0 = thread::spawn(work_0);
    let thread_1 = thread::spawn(work_1);

    thread_0.join().unwrap();
    thread_1.join().unwrap();

    println!("value: {}", data.value);
}
