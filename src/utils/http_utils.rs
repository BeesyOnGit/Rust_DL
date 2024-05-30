use std::io::Read;
use std::sync::mpsc::Sender;

use bytes::Bytes;

use reqwest::blocking::Client;
use reqwest::Client as asyncCli;

pub async fn get_file_size(url_str: &str) -> u64 {
    let client = asyncCli::new();
    match client.head(url_str).send().await {
        Ok(mut response) => {
            // Check if the request was successful
            if !response.status().is_success() {
                println!("Request was not successful: {}", response.status());
                return 0;
            }
            // Get the Content-Length header
            let headers = response.headers_mut();

            let content_length = headers["content-length"].clone();
            let content_length_str = String::from(content_length.to_str().unwrap());

            if content_length_str.is_empty() {
                return 0;
            }
            return content_length_str.parse::<u64>().unwrap();
        }
        Err(e) => {
            println!("Error sending request: {}", e);
            return 0;
        }
    }
    // return 388960659;
}

#[derive(Debug)]
pub struct PartStruct {
    pub part_number: i8,
    pub content: Bytes,
    pub empty: bool,
    pub progress: u64,
}

pub fn download_part(
    url: String,
    start: String,
    end: String,
    part_num: i8,
    sender: Sender<PartStruct>,
) -> PartStruct {
    let client = Client::new();

    let mut request = client
        .get(url)
        .header("Range", format!("bytes={}-{}", start, end))
        // .timeout(std::time::Duration::from_secs(36000))
        .send();
    // .send();

    let mut buffer = [0; 4096]; // Define the buffer size
    let mut vec_bod: Vec<u8> = Vec::new();
    while let Ok(read_bytes) = request.as_mut().unwrap().read(&mut buffer) {
        if read_bytes == 0 {
            break;
        }
        vec_bod.extend_from_slice(&buffer[..read_bytes]);

        // downloaded_bytes += read_bytes as u64;
        // println!("thread {} : {} byts", part_num, downloaded_bytes);

        let _ = sender.send(PartStruct {
            part_number: 0,
            content: Bytes::new(),
            empty: true,
            progress: read_bytes.to_owned() as u64,
        });
    }

    match request {
        Ok(res) => {
            if !res.status().is_success() {
                return PartStruct {
                    content: Bytes::new(),
                    part_number: part_num,
                    progress: 0,
                    empty: true,
                };
            }
            match res.bytes() {
                Ok(_) => {
                    return PartStruct {
                        content: Bytes::from(vec_bod),
                        part_number: part_num,
                        progress: 0,
                        empty: false,
                    }
                }
                Err(err) => {
                    println!("{}", err);
                    return PartStruct {
                        content: Bytes::new(),
                        part_number: part_num,
                        progress: 0,
                        empty: true,
                    };
                }
            }
        }

        Err(err) => {
            println!("{:#?} file", err);
            return PartStruct {
                content: Bytes::new(),
                part_number: part_num,
                progress: 0,
                empty: true,
            };
        }
    }
}

// pub async fn _download_file(url: String) -> PartStruct {
//     let client = asyncCli::new();
//     println!("{}", url);
//     let request = client
//         .get(url)
//         .header("Access-Control-Allow-Origin", "*")
//         .timeout(std::time::Duration::from_secs(20000))
//         .send()
//         .await;
//     // .send();

//     println!("start download");

//     match request {
//         Ok(res) => {
//             if !res.status().is_success() {
//                 return PartStruct {
//                     content: Bytes::new(),
//                     part_number: 1,
//                     empty: true,
//                 };
//             }
//             return PartStruct {
//                 content: res.bytes().await.unwrap(),
//                 part_number: 1,
//                 empty: false,
//             };
//         }
//         Err(err) => {
//             println!("{:#?} file : http_utils.rs", err);
//             return PartStruct {
//                 content: Bytes::new(),
//                 part_number: 1,
//                 empty: true,
//             };
//         }
//     }
// }
