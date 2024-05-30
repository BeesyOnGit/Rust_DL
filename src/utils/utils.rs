use std::{
    fs::File,
    io::{self, Write},
};

use num_cpus;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

use super::http_utils::PartStruct;

pub fn round_f32(float: f32) -> i32 {
    let char_float = float.to_string();
    let chars_vec: Vec<&str> = char_float.split(".").collect();
    let num_rounded = chars_vec[0].parse::<i32>().unwrap();
    return num_rounded;
}

pub fn get_usable_cpus(divider: f32) -> i32 {
    let cpu_number = num_cpus::get() as f32;
    let usable_cpu = round_f32(cpu_number / divider);
    return usable_cpu;
}

pub fn prompt_user(message: String) -> String {
    // Prompt the user for input
    println!("{}", &message);

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Read line failed.");

    input = String::from(input.trim());

    return input;
}

pub fn check_url(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(_) => {
            return true;
        }
        Err(_) => return false,
    }
}

pub fn extract_file_name(url_str: &str) -> String {
    let url_owned = url_str.to_owned();
    let url_vec: Vec<&str> = url_owned.split("/").collect();
    let mut file_name = url_vec[url_vec.len() - 1].trim().to_owned();

    let url_parsed = Url::parse(url_str).unwrap();
    let scheme = url_parsed.scheme();
    let host = url_parsed.host_str().unwrap();

    if file_name == scheme {
        return "".to_string();
    }
    if file_name == host {
        return "".to_string();
    }
    if !file_name.contains(".") {
        return "".to_string();
    }
    if file_name.contains("=") {
        let tmp_file_name: Vec<&str> = file_name.split("=").collect();
        file_name = tmp_file_name[0].trim().to_owned();
    }

    return file_name.to_string();
}

pub fn save_to_file(file_name: &str, bytes_vec: Vec<PartStruct>) -> bool {
    let mut file = File::create(format!("C:/Users/THINKBOOK/Downloads/{}", file_name)).unwrap();
    for part in bytes_vec {
        file.write_all(&part.content).unwrap();
    }
    return true;
}

pub fn stamp_time() -> u128 {
    // Get the current system time
    let now = SystemTime::now();

    // Get the duration since the Unix epoch (1970-01-01 00:00:00 UTC)
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis(),
        Err(_) => {
            // Handle the case where the current time is before the Unix epoch
            panic!("SystemTime before UNIX EPOCH!");
        }
    }
}

pub fn format_millis(millis: u128) -> String {
    // Calculate hours, minutes, seconds, and remaining milliseconds
    let hours = millis / 3_600_000;
    let minutes = (millis % 3_600_000) / 60_000;
    let seconds = (millis % 60_000) / 1000;
    let remaining_millis = millis % 1000;

    // Format the components into a string
    let mut formatted_time = String::new();
    if hours > 0 {
        formatted_time.push_str(&format!("{:02}H ", hours));
    }
    if minutes > 0 {
        formatted_time.push_str(&format!("{:02}M ", minutes));
    }
    formatted_time.push_str(&format!("{:02}S ", seconds));
    formatted_time.push_str(&format!("{:03}Ms", remaining_millis));

    formatted_time
}
