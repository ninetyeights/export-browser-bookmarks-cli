use crate::{Browser, BrowserInfo};

extern crate dirs;

#[cfg(target_os = "macos")]
pub fn get_path(browser: &Browser) -> BrowserInfo {
    let mut data_dir = dirs::home_dir().unwrap();
    match browser {
        Browser::Chrome => {
            data_dir.push("Library/Application Support/Google/Chrome/");
            BrowserInfo {
                app_path: vec!["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"],
                app_name: "Google Chrome",
                data_dir
            }
        },
        Browser::Edge => {
            data_dir.push("Library/Application Support/Microsoft Edge");
            BrowserInfo {
                app_path: vec!["/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"],
                app_name: "Microsoft Edge",
                data_dir
            }
        },
        Browser::Brave => {
            data_dir.push("Library/Application Support/BraveSoftware/Brave-Browser");
            BrowserInfo {
                app_path: vec!["/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"],
                app_name: "Brave Browser",
                data_dir
            }
        },
    }
}