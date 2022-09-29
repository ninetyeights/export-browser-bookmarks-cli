use console::Term;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
#[macro_use]
extern crate lazy_static;

use csv::Writer;
use serde_json::Value;
extern crate dirs;
mod browser_path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Browser {
    Chrome,
    Edge,
    Brave,
}

#[derive(Debug)]
pub struct BrowserInfo {
    app_path: Vec<&'static str>,
    app_name: &'static str,
    data_dir: PathBuf,
}

#[derive(Debug)]
struct BookmarkList {
    data: Vec<[String; 3]>,
}

impl BookmarkList {
    fn new() -> BookmarkList {
        BookmarkList { data: Vec::new() }
    }

    fn push(&mut self, value: [String; 3]) {
        self.data.push(value);
    }
}

lazy_static! {
    #[derive(Debug)]
    static ref BOOKMARK_LIST: Mutex<BookmarkList> = Mutex::new(BookmarkList::new());
    #[derive(Debug)]
    static ref PARENT_PATH: Mutex<String> = Mutex::new(String::new());
    static ref CURRENT_USER: Mutex<String> = Mutex::new(String::new());
}

// 判断是否安装浏览器，是的话返回浏览器路径信息，否则不添加此浏览器
fn get_browser_info(browsers: &[Browser; 3]) -> HashMap<&Browser, BrowserInfo> {
    let mut items: HashMap<&Browser, BrowserInfo> = HashMap::new();
    for browser in browsers.iter() {
        let info = browser_path::get_path(browser);
        let mut has_app = false;
        for path in info.app_path.iter() {
            if Path::new(path).exists() {
                has_app = true;
            }
        }
        if has_app {
            items.insert(browser, info);
        }
    }
    items
}

fn main() {
    // 名字对应枚举
    let mut names = HashMap::new();
    names.insert("Google Chrome", Browser::Chrome);
    names.insert("Microsoft Edge", Browser::Edge);
    names.insert("Brave Browser", Browser::Brave);
    let browsers: [Browser; 3] = [Browser::Chrome, Browser::Edge, Browser::Brave];
    let items = get_browser_info(&browsers);
    let mut options: Vec<&str> = Vec::new();

    for (_, value) in items.iter() {
        options.push(value.app_name);
        // options.push(match key {
        //     Browser::Chrome => "Google Chrome",
        //     Browser::Edge => "Microsoft Edge",
        //     Browser::Brave => "Brave Browser",
        // });
    }
    options.sort();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择浏览器")
        .items(&options)
        .default(1)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    let selection = selection.unwrap();

    // 根据选择的浏览器获取 Local State 用户信息
    let item = items.get(names.get(options[selection]).unwrap()).unwrap();
    let local_state = Path::new(&item.data_dir).join("Local State");
    let value: Value = read_file(&local_state);
    let info: &Value = &value["profile"]["info_cache"];

    // 通过用户目录获取书签文件路径
    let mut profile_options = Vec::new();
    let mut bookmarks = Vec::new();
    for (key, value) in info.as_object().unwrap().iter() {
        profile_options.push(value.get("name").unwrap());
        bookmarks.push(Path::new(&item.data_dir).join(key).join("Bookmarks"));
    }

    let chosen = MultiSelect::new()
        .with_prompt("选择用户")
        .items(&profile_options)
        .interact()
        .unwrap();

    if chosen.is_empty() {
        println!("未选择用户，程序退出!");
        std::process::exit(1);
    }

    // 循环选择的用户列表
    for index in chosen.iter() {
        let path = &bookmarks[*index];
        if !path.exists() {
            println!("跳过用户: {} 没有书签文件", profile_options[*index]);
            continue;
        }
        // 读取书签内容
        let value: Value = read_file(path);
        let value = value.get("roots").unwrap();

        CURRENT_USER.lock().unwrap().clear();
        CURRENT_USER
            .lock()
            .unwrap()
            .push_str(profile_options[*index].as_str().unwrap());

        // 处理书签内容
        handle_bookmarks(value);
    }

    // println!("{:#?}", BOOKMARK_LIST.lock().unwrap());
    // println!("Finished");
    let save_path = Path::new("./export.csv");
    let mut writer = Writer::from_path(save_path).unwrap();
    for item in BOOKMARK_LIST.lock().unwrap().data.iter() {
        writer.write_record(item).unwrap();
        writer.flush().unwrap();
    }
}

fn handle_bookmarks(data: &Value) {
    for (_, value) in data.as_object().unwrap().iter() {
        handle_children(value);
    }
}

// 一层一层往下循环获取书签内容并添加到全局变量中
fn handle_children(value: &Value) {
    let data = value.get("children").unwrap();

    let id = value.get("id").unwrap().as_str().unwrap();
    // 判断第一层级书签并添加用户前缀
    if id == "1" || id == "2" || id == "3" {
        PARENT_PATH.lock().unwrap().clear();
        PARENT_PATH
            .lock()
            .unwrap()
            .push_str(CURRENT_USER.lock().unwrap().as_str());
        PARENT_PATH.lock().unwrap().push('/');
    }

    // 添加文件夹层级
    PARENT_PATH
        .lock()
        .unwrap()
        .push_str(value.get("name").unwrap().as_str().unwrap());
    PARENT_PATH.lock().unwrap().push('/');

    // 循环子级书签
    for item in data.as_array().unwrap().iter() {
        // 判断书签类型为链接添加数据
        let _type = item.get("type").unwrap().as_str().unwrap();
        if _type == String::from("url") {
            let mut full_path = String::new();
            full_path.push_str(PARENT_PATH.lock().unwrap().as_str());
            full_path.push_str(item.get("name").unwrap().as_str().unwrap());
            let url = item.get("url").unwrap().as_str().unwrap();
            BOOKMARK_LIST.lock().unwrap().push([
                full_path.to_string(),
                url.to_string(),
                String::from("testing"),
            ])
        }

        // 如果还有子级书签循环调用当前函数处理
        match item.get("children") {
            #[allow(unused_variables)]
            Some(value) => handle_children(item),
            None => (),
        }
    }
}

// 读取文件内容转换为JSON
fn read_file(path: &PathBuf) -> Value {
    let file = File::open(path).unwrap();
    serde_json::from_reader(file).unwrap()
}
