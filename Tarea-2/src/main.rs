mod product;
mod factory;

use product::Product;
use factory::Factory;

fn main() {
    let mut factory = Factory::new();

    let mut p1 = Product::new(1);
    let mut p2 = Product::new(2);
    // registrar arrival relative al inicio de la fábrica
    p1.arrival_time = Some(factory.current_time.elapsed());
    p2.arrival_time = Some(factory.current_time.elapsed());

    factory.cutting_queue.push_back(p1);
    factory.cutting_queue.push_back(p2);

    factory.cutting_stage();
    factory.assembly_stage();
    factory.packaging_stage();
}

    // let handle = thread::spawn(move || {
    //     // thread work
    //     let start = Instant::now();
    //     thread::sleep(Duration::from_secs(3));
    //     println!("Thread finished after {:?}", start.elapsed());
    // });

    // handle.join().unwrap();


    // let mut queue: VecDeque<i32> = VecDeque::new();

    // queue.push_back(1);
    // queue.push_back(2);
    // queue.push_back(3);

    // queue.pop_front();

    // for item in &queue {
    //     println!("{}", item);
    // }