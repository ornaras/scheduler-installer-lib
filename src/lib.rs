extern crate rand;

use std::fs::File;
use std::env::temp_dir;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use runas::Command;
use futures_util::StreamExt;
use rand::distr::{Alphanumeric, SampleString};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use tokio::runtime::Runtime;

const ASPNET_URL: &str = "https://download.visualstudio.microsoft.com/download/pr/8cfa7f46-88f2-4521-a2d8-59b827420344/447de18a48115ac0fe6f381f0528e7a5/aspnetcore-runtime-6.0.36-win-x86.exe"; // {5FEC97CA-FD93-392D-BF36-D9C3492A5698}
const HOSTING_BUNDLE: &str = "https://download.visualstudio.microsoft.com/download/pr/9b8253ef-554d-4636-b708-e154c0199ce5/f3673dd1f2dc80e5b0505cbd2d4bd5d2/dotnet-hosting-6.0.36-win.exe"; // {040F8B83-B3BA-303A-A5BC-FE3E7FC0093B}
const ENABLE_IIS: &str = "\"Enable-WindowsOptionalFeature -Online -FeatureName IIS-ASPNET, IIS-ManagementConsole -All\"";
const CREATE_SITE: &str = "\"New-IISSite -Name SkatWorkerAPI -BindingInformation '*:80:' -PhysicalPath '$env:systemdrive\\ScanKassWorker'\"";
const WD86_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/WebDeploy_x86_ru-RU.msi";
const WD64_URL: &str = "https://download.microsoft.com/download/b/d/8/bd882ec4-12e0-481a-9b32-0fae8e3c0b78/webdeploy_amd64_ru-RU.msi";

#[no_mangle]
pub extern "C" fn is_installed() -> bool{
    std::fs::exists("C:\\ScanKassWorker\\SkatWorkerAPI.exe").unwrap()
}

fn exists_app(pattern: &str) -> bool{
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut soft = hklm.open_subkey("Software").unwrap();
    if architecture() == "AMD64" {
        soft = soft.open_subkey("WOW6432Node").unwrap();
    }
    let apps = soft.open_subkey("Microsoft\\Windows\\CurrentVersion\\Uninstall").unwrap();
    for i in apps.enum_keys().map(|x| x.unwrap()) {
        if i.starts_with(pattern) {
            return true;
        }
    }
    false
}

fn architecture() -> String {
    std::env::var("PROCESSOR_ARCHITECTURE").unwrap()
}

#[no_mangle]
pub extern "C" fn install() -> i32 {
    Runtime::new().unwrap().block_on(install_async())
}

async fn install_async() -> i32 {
    if is_installed() { return 1; }

    let wd_url = match architecture().as_str() {
        "AMD64" => WD64_URL,
        "x86" => WD86_URL,
        _ => return 2
    };

    if !exists_app("Microsoft ASP.NET Core 6.0.36 Shared Framework") {
        download_and_execute(ASPNET_URL).await;
    }

    if !exists_app("Microsoft ASP.NET Core 6.0.36 Hosting Bundle") {
        download_and_execute(HOSTING_BUNDLE).await;
    }

    download_and_install(wd_url).await;

    Command::new("powershell").arg("-Command").arg(ENABLE_IIS).status().unwrap();
    Command::new("powershell").arg("-Command").arg(CREATE_SITE).status().unwrap();

    install_skat_worker().await;

    0
}

async fn download(url: &str, extension: &str) -> String {
    let filename = format!("{0}.{1}", Alphanumeric.sample_string(&mut rand::rng(),16), extension);
    let filepath = format!("{0}{1}", temp_dir().display(), filename);
    let mut file = File::create(filepath.clone()).expect("Не удалось создать файл");

    let resp = reqwest::get(url).await.expect("Не удалось отправить HTTP-запрос");
    let mut stream = resp.bytes_stream();

    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        file.write_all(&chunk).unwrap();
    }

    file.flush().unwrap();

    filepath
}

async fn download_and_execute(url: &str) {
    let path = download(url, "exe").await;
    Command::new(&path).arg("/install").arg("/passive").arg("/norestart").status().unwrap();
    std::fs::remove_file(&path).unwrap();
}

async fn download_and_install(url: &str) {
    let path = download(url, "msi").await;
    Command::new("msiexec").arg("/i").arg(path.as_str()).arg("/passive").arg("/norestart")
        .status().unwrap();
    std::fs::remove_file(&path).unwrap();
}

async fn download_and_extract(url: &str) -> String {
    let path = download(url, "zip").await;
    let mut file = File::open(path).unwrap();
    let mut data: Vec<u8> = vec![];
    file.read_to_end(&mut data).unwrap();
    let dir_path = format!("{0}{1}",temp_dir().display(),Alphanumeric.sample_string(&mut rand::rng(),16));
    std::fs::create_dir_all(&dir_path).unwrap();
    zip_extract::extract(Cursor::new(&data), &PathBuf::from(&dir_path), true).unwrap();
    dir_path
}

async fn get_latest_release(owner: &str, repo: &str) -> String {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{0}/{1}/releases",owner,repo);
    let resp = client.get(&url)
        .header("accept", "application/vnd.github+json")
        .header("User-Agent", "curl")
        .send().await.expect("Не удалось отправить запрос");
    let body = resp.text().await.expect("Не удалось получить тело ответа");
    let json_value: serde_json::Value = serde_json::from_str(&body).unwrap();
    let res = format!("{}", json_value[0]["assets"][0]["browser_download_url"]);
    res[1..res.len() - 1].to_string()
}

async fn install_skat_worker(){
    let url = get_latest_release("StarkovVV18", "SkatWorker").await;
    let path = download_and_extract(url.as_str()).await;
    Command::new(format!("{}/SkatWorkerAPI.deploy.cmd", path)).arg("/Y").status().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        install();
        assert_eq!(is_installed(), true);
    }
}
