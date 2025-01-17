use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Debug)]
struct Data {
    value: u32,
}

fn main() -> thread::Result<()> {
    let data = Data { value: 0 };
    let arc_mutex = Arc::new(Mutex::new(data));
    let arc_mutex_0 = arc_mutex.clone();
    let arc_mutex_1 = arc_mutex.clone();

    let work_0 = move || {
        (0..50000).for_each(|_| {
            if let Ok(mut data) = arc_mutex_0.lock() {
                data.value += 1;
            }
        });
    };
    let work_1 = move || {
        (0..50000).for_each(|_| {
            if let Ok(mut data) = arc_mutex_1.lock() {
                data.value += 1;
            }
        })
    };

    let thread_0 = thread::spawn(work_0);
    let thread_1 = thread::spawn(work_1);

    thread_0.join()?;
    thread_1.join()?;

    if let Ok(Ok(data)) = Arc::try_unwrap(arc_mutex).map(Mutex::into_inner) {
        println!("value: {}", data.value);
    }

    Ok(())
}
