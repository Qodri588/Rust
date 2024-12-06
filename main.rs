use reqwest::Client;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::error::Error;

// Struktur untuk deserialisasi JSON dari API
#[derive(Deserialize)]
struct Folder {
    fld_id: String,
    name: String,
}

#[derive(Deserialize)]
struct File {
    file_code: String,
    title: String,
    download_url: String,
    single_img: String,
    length: i64,
    views: i64,
    uploaded: String,
    fld_id: String,
    name: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    msg: String,
    result: T,
}

#[derive(Deserialize)]
struct FolderResult {
    folders: Vec<Folder>,
}

#[derive(Deserialize)]
struct FileResult {
    files: Vec<File>,
}

const API_KEY: &str = "350871o0uomobcm787efod";
const BASE_URL: &str = "https://doodapi.com/api";

async fn fetch_folders(client: &Client) -> Result<Vec<Folder>, Box<dyn Error>> {
    let url = format!("{}/folder/list?key={}&fld_id=0", BASE_URL, API_KEY);
    let response: ApiResponse<FolderResult> = client.get(&url).send().await?.json().await?;
    if response.msg == "OK" {
        Ok(response.result.folders)
    } else {
        Err("Failed to fetch folders".into())
    }
}

async fn fetch_files(client: &Client, folder_id: &str) -> Result<Vec<File>, Box<dyn Error>> {
    let url = format!("{}/file/list?key={}&fld_id={}", BASE_URL, API_KEY, folder_id);
    let response: ApiResponse<FileResult> = client.get(&url).send().await?.json().await?;
    if response.msg == "OK" {
        Ok(response.result.files)
    } else {
        Err(format!("Failed to fetch files for folder {}", folder_id).into())
    }
}

fn init_database(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            fld_id TEXT PRIMARY KEY,
            name TEXT,
            parent_id TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            file_code TEXT PRIMARY KEY,
            title TEXT,
            download_url TEXT,
            single_img TEXT,
            length INTEGER,
            views INTEGER,
            uploaded TEXT,
            fld_id TEXT,
            name TEXT
        )",
        [],
    )?;
    Ok(())
}

fn save_folder(conn: &Connection, folder: &Folder) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT OR IGNORE INTO folders (fld_id, name, parent_id) VALUES (?1, ?2, ?3)",
        params![folder.fld_id, folder.name, "0"],
    )?;
    Ok(())
}

fn save_file(conn: &Connection, file: &File) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT OR IGNORE INTO files (file_code, title, download_url, single_img, length, views, uploaded, fld_id, name) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            file.file_code, file.title, file.download_url, file.single_img, file.length, file.views, file.uploaded,
            file.fld_id, file.name
        ],
    )?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let conn = Connection::open("dood.db")?;

    init_database(&conn)?;

    let folders = fetch_folders(&client).await?;
    for folder in folders {
        save_folder(&conn, &folder)?;

        let files = fetch_files(&client, &folder.fld_id).await?;
        for file in files {
            save_file(&conn, &file)?;
        }
    }

    println!("Finished processing folders and files.");
    Ok(())
}
