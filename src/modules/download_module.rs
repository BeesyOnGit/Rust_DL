use std::{collections::HashMap, sync::mpsc::channel, thread};

use console::Term;

use crate::utils::{
    http_utils::{download_part, get_file_size, PartStruct},
    utils::{
        check_url, extract_file_name, format_millis, get_usable_cpus, prompt_user, save_to_file,
        stamp_time,
    },
};

#[derive(Debug)]
struct ProgressStruct {
    pub part: i8,
    pub total_size: u128,
    pub current_progess: u128,
    pub thread_start_ts: u128,
    pub last_contact: u128,
}

pub async fn download() {
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
        download_url = prompt_user(String::from("Please Provide The Url To Download From"));

        // check validity
        is_valid_url = check_url(&download_url);

        // if not valid
        if !is_valid_url {
            println!("url {} is not valid", download_url);
            println!("\n \n \n")
        }
    }

    let mut save_path = prompt_user(String::from(
        "Do you want to provide a location to download to (if not leave empty and press enter)",
    ));

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

    // create a hashMap to store the progress for every thread
    let mut progress_hash_map: HashMap<i8, ProgressStruct> = HashMap::new();

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

        // calculate total bytes size to be downloaded by the thread
        let part_size = end.parse::<u128>().unwrap() - start.parse::<u128>().unwrap();

        // init th threads progress
        progress_hash_map.insert(
            i as i8,
            ProgressStruct {
                part: i as i8,
                total_size: part_size,
                current_progess: 0,
                thread_start_ts: stamp_time(),
                last_contact: stamp_time() + 1000,
            },
        );

        // spawn a thread for every usable cpu (thread)
        thread::spawn(move || {
            // call the function that will download the part
            let result = download_part(
                download_url_clone,
                start,
                end,
                i as i8,
                sender_clone.clone(),
            );

            // send the result to the channel receiver once received
            sender_clone.send(result).unwrap()
        });
    }

    // infor user that download started
    println!("Download Started");

    // declare and init a new vector that will later receive the result of each part
    let mut result_vec: Vec<PartStruct> = Vec::new();

    // download general advancement
    let mut downloaded: u128 = 0;

    // initializing the terminal
    let stdout = Term::stdout();

    // store download progress status
    // let finish_download = false;

    loop {
        // Receive a result from a thread
        let result = receiver.recv().unwrap();

        // if received value is not the downloaded part
        if result.empty {
            // add to the progression
            downloaded += result.progress as u128;

            // edit the progress state of the sender thread in the hashMap
            if let Some(progress) = progress_hash_map.get_mut(&result.part_number) {
                progress.current_progess += result.progress as u128;
                progress.last_contact = stamp_time();
            }

            // clear previous console logs
            stdout
                .clear_last_lines(progress_hash_map.len() + 1)
                .unwrap();

            // calculate current progress
            let percent = downloaded as f32 * 100.00 / file_size as f32;

            // calculate elapsed time from start
            let elapsed_time = (stamp_time() - begin_ts) / 1000;

            // calculate average download speed
            let download_speed = (downloaded / 1000) / elapsed_time;

            // print the calculations to user
            for (_, struc) in progress_hash_map.iter() {
                let ProgressStruct {
                    total_size,
                    part,
                    current_progess,
                    last_contact,
                    thread_start_ts,
                } = struc;
                println!(
                    "part {} progress : {:3.02} % | speed : {} Ko/s",
                    part + 1,
                    current_progess * 100 / total_size,
                    (current_progess * 1000) / (last_contact - thread_start_ts) / 1000
                )
            }
            println!(
                "Total Downloaded {:3.02} % | Average speed : {} Ko/s ",
                &percent, &download_speed
            );
            continue;
        }

        // if the received value is the downloaded part push it into a new vec
        result_vec.push(result);

        // if the vec have parts equivalent to the used cpus stop the loop
        if result_vec.len() as i32 == usable_cpus {
            // finish_download = true;
            break;
        }
    }

    // sort the result vector
    result_vec.sort_by(|a, b| a.part_number.cmp(&b.part_number));

    // check for empty parts
    for part in result_vec.iter() {
        if part.empty {
            println!("a part of the file failed to download, please try again");
            return;
        }
    }

    if save_path.is_empty() {
        save_path = String::from("./download")
    }

    // save the result bytes to a file
    let is_file_saved = save_to_file(
        &save_path,
        &format!("{}/{}", &save_path, &file_name),
        result_vec,
    );
    if !is_file_saved {
        println!("the file was downloaded but failed to be saved, please try again");
        return;
    }

    // define operation end time
    let end_ts = stamp_time();

    // define download duration
    let duration = format_millis(end_ts - begin_ts);

    println!("File downloaded and saved");
    println!("\n");
    println!(
        "Downloaded ~ {:4.02} Mo in {}",
        file_size / 1_000_000,
        duration
    );
    return;
}
