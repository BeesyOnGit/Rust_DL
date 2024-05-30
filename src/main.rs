mod modules;
mod utils;
use modules::download_module::download;
use utils::utils::prompt_user;

#[tokio::main]
async fn main() {
    loop {
        // invoke download function
        download().await;

        // print new line
        print!("\n \n");

        // prompt user for another download
        let keep_downloading =
            prompt_user("Do you want to download anything else ? tap [Yes] or [No]".to_string())
                .to_lowercase();

        // break loop if user dont want to continue
        if keep_downloading.is_empty() || keep_downloading == "no" {
            break;
        }
    }
    println!("Thanks for using the downloader 🦀")
}
