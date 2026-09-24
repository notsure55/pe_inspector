fn main() {
    let health = 100;

    loop {
        println!("my value:{:X}", health);
        println!("my address :{:#X?}", (&health) as *const _);

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
