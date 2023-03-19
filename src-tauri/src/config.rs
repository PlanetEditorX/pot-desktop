use crate::shortcut::register_shortcut;
use crate::APP;
use std::sync::Mutex;
use std::{fs, fs::OpenOptions, io::Read, io::Write, path::PathBuf};
use tauri::{api::path::config_dir, Manager};
use toml::{Table, Value};

pub static APPID: &str = "cn.pylogmon.pot";

fn get_app_config_dir() -> PathBuf {
    let mut app_config_dir_path = config_dir().expect("Get Config Dir Failed");
    app_config_dir_path.push(APPID);
    app_config_dir_path
}

fn get_app_config_file() -> PathBuf {
    let mut app_config_file_path = get_app_config_dir();
    app_config_file_path.push("config.toml");
    app_config_file_path
}

fn check_config() -> bool {
    // 配置目录路径
    let app_config_dir_path = get_app_config_dir();
    // 配置文件路径
    let app_config_file_path = get_app_config_file();

    if !app_config_file_path.exists() {
        if !app_config_dir_path.exists() {
            // 创建目录
            fs::create_dir_all(app_config_dir_path).expect("Create Config Dir Failed");
        }
        // 创建文件
        fs::File::create(app_config_file_path).expect("Create Config File Failed");
        return false;
    }
    return true;
}

pub struct ConfigWrapper(pub Mutex<Config>);
// 配置文件结构体
pub struct Config {
    pub config_toml: Table,
}

impl Config {
    pub fn init_config() -> bool {
        // 配置文件路径
        let app_config_file_path = get_app_config_file();
        // 检查配置文件
        let flag = check_config();
        // 读取配置文件
        let mut config_file =
            fs::File::open(&app_config_file_path).expect("Open Config File Failed");
        let mut contents = String::new();
        config_file
            .read_to_string(&mut contents)
            .expect("Read Config File Failed");
        // 构造配置结构体
        let config = ConfigWrapper(Mutex::new(Config {
            config_toml: contents.parse::<Table>().expect("Parse Config File Failed"),
        }));
        // 写入状态
        APP.get().unwrap().manage(config);
        return flag;
    }
    pub fn get(&self, key: &str, default: Value) -> Value {
        match self.config_toml.get(key) {
            Some(v) => v.to_owned(),
            None => default,
        }
    }
    pub fn set(&mut self, key: &str, value: Value) {
        self.config_toml.insert(key.to_string(), value);
    }
    pub fn write(&self) -> Result<(), String> {
        let app_config_file_path = get_app_config_file();
        let mut config_file = OpenOptions::new()
            .write(true)
            .open(&app_config_file_path)
            .expect("Open Config File Failed");
        let contents = self.config_toml.to_string();
        match config_file.write_all(contents.as_bytes()) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        }
    }
}

pub fn get_config(key: &str, default: Value, state: tauri::State<ConfigWrapper>) -> Value {
    state.0.lock().unwrap().get(key, default)
}

#[tauri::command]
pub fn set_config(key: &str, value: Value, state: tauri::State<ConfigWrapper>) {
    state.0.lock().unwrap().set(key, value);
}

#[tauri::command]
pub fn write_config(state: tauri::State<ConfigWrapper>) -> Result<(), String> {
    register_shortcut().unwrap();
    state.0.lock().unwrap().write()
}

#[tauri::command]
pub fn get_config_str(state: tauri::State<ConfigWrapper>) -> Table {
    println!("{:?}", state.0.lock().unwrap().config_toml);
    return state.0.lock().unwrap().config_toml.clone();
}
