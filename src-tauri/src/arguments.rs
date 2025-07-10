use std::env;

use base64::encode;
use mac_address::get_mac_address;
use machine_uid::get as get_machine_uid;
use rand::Rng;
use std::io;

#[derive(Debug)]
struct Argument {
    key: String,
    data: String,
}

pub fn handle_arguments() {
    let is_key = |input: &str| input.chars().take(2).collect::<String>() == "--";

    let input_args_raw: Vec<String> = env::args().collect::<Vec<String>>();
    let mut args: std::iter::Peekable<std::slice::Iter<'_, String>> =
        input_args_raw.iter().peekable();

    while let Some(arg) = args.next() {
        if is_key(arg) {
            let key: String = arg.chars().skip(2).collect::<String>();

            let data: String = match args.peek() {
                Some(next_arg) if !is_key(next_arg) => args.next().unwrap().clone(),
                _ => String::new(),
            };

            let argument_pair: Argument = Argument { key, data };
        }
    }
}

pub fn get_device_id() -> Option<String> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        return Some(
            rand::thread_rng()
                .gen_range(1_000_000_000..2_000_000_000)
                .to_string(),
        );
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let mut id = 0u32;
        if let Ok(Some(ma)) = get_mac_address() {
            for x in &ma.bytes()[2..] {
                id = (id << 8) | (*x as u32);
            }
            id &= 0x1FFFFFFF;
            return Some(id.to_string());
        }
        None
    }
}
