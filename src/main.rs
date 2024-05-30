mod utils;

use console::Term;
use std::{sync::mpsc::channel, thread};
use utils::{
    http_utils::{download_part, get_file_size, PartStruct},
    utils::{
        check_url, extract_file_name, format_millis, get_usable_cpus, prompt_user, save_to_file,
        stamp_time,
    },
};

#[tokio::main]
async fn main() {
    // defin start time
    let begin_ts = stamp_time();

    // get the total usable cores
    let usable_cpus = get_usable_cpus(1.1);

    // variable that holds the url validity
    let mut is_valid_url = false;

    // varibale that holds the url
    let mut download_url = String::new();

    // looping until the user provides a valid url
    while !is_valid_url {
        // get url from user
        download_url = prompt_user(String::from("please provide the url"));

        // check validity
        is_valid_url = check_url(&download_url);

        // if not valid
        if !is_valid_url {
            println!("url {} is not valid", download_url);
            println!("\n \n \n")
        }
    }

    // extracting the file name from url
    let file_name = extract_file_name(&download_url);

    // checking if there is a file to download
    if file_name.is_empty() {
        println!("the url you provided seems to not provide a file");
        return;
    }

    // getting file size from server
    let file_size = get_file_size(&download_url).await;

    // defining every part size based on the available cpus
    let chunks_size: u128 = file_size as u128 / usable_cpus as u128;

    // creating a channel to comunicate with the to-be spawned threads
    let (sender, receiver) = channel::<PartStruct>();

    for i in 0..=&usable_cpus - 1 {
        // clone the channel sender in order for each thread to have one
        let sender_clone = sender.clone();

        // define the range start on the current thread
        let start = String::from(format!("{}", i as u128 * chunks_size));

        // define the range end of the current thread
        let mut end = String::from(format!("{}", (i as u128 + 1) * chunks_size - 1));

        // if the current iteration in the last set end to the total file size
        if i == usable_cpus - 1 {
            end = file_size.to_string()
        }

        // cloning the link inside the loop in order of it to be moved to the spawned thread
        let download_url_clone = download_url.clone();

        // spawn a thread for every usable cpu
        thread::spawn(move || {
            // call the function that will download the part
            let result = download_part(
                download_url_clone,
                start,
                end,
                i as i8,
                sender_clone.clone(),
            );

            // send the result to the cannel receiver once received
            sender_clone.send(result).unwrap()
        });
    }

    println!("Download Started");

    // declare and init a new vector that will later receive the result of each part
    let mut result_vec: Vec<PartStruct> = Vec::new();

    // advancement
    let mut downloaded: u128 = 0;
    let stdout = Term::stdout();
    let mut finish_download = false;

    while !finish_download {
        // Receive a result from a thread
        let result = receiver.recv().unwrap();

        if result.empty {
            downloaded += result.progress as u128;

            stdout.clear_last_lines(1).unwrap();

            let percent = downloaded as f32 * 100.00 / file_size as f32;

            let elapsed_time = (stamp_time() - begin_ts) / 1000;

            let download_speed = (downloaded / 1000) / elapsed_time;
            println!(
                "Downloaded {:3.02} % | Average speed : {} Ko/s ",
                &percent, &download_speed
            );
            continue;
        }
        // push the result parts into a new vec
        result_vec.push(result);

        if result_vec.len() as i32 == usable_cpus {
            finish_download = true
        }
    }

    // for _ in 0..=usable_cpus - 1.clone() {
    //     // Receive a result from a thread
    //     let result = receiver.recv().unwrap();

    //     if result.empty {
    //         downloaded += result.progress as u128;
    //         stdout.clear_last_lines(1).unwrap();
    //         println!("Downloaded {} % \n", &downloaded);
    //     }
    //     // push the result parts into a new vec
    //     result_vec.push(result);
    // }

    // sort the result vector
    result_vec.sort_by(|a, b| a.part_number.cmp(&b.part_number));

    for part in result_vec.iter() {
        if part.empty {
            println!("a part of the file failed to download, please try again");
            return;
        }
    }

    // save the result bytes to a file
    let is_file_saved = save_to_file(&file_name, result_vec);
    if !is_file_saved {
        println!("the file was downloaded but failed to be saved, please try again");
        return;
    }

    // define opearation end time
    let end_ts = stamp_time();

    // define duration
    let duration = format_millis(end_ts - begin_ts);

    println!("File downloaded and saved");
    println!("\n");
    println!("Download Finished in {}", duration);
    return;
}
