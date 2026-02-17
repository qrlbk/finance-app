//! Встроенная модель: скачивание GGUF и регистрация в Ollama. Пользователь один раз ставит Ollama,
//! в приложении включает «Встроенная модель» — приложение скачивает модель и создаёт её в Ollama.

use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

pub const EMBEDDED_MODEL_NAME: &str = "finance-embedded";
const GGUF_FILENAME: &str = "smollm2-360m-instruct-q8_0.gguf";
const HF_URL: &str = "https://huggingface.co/HuggingFaceTB/SmolLM2-360M-Instruct-GGUF/resolve/main/smollm2-360m-instruct-q8_0.gguf";

/// 0–100 or 255 if not downloading
static DOWNLOAD_PROGRESS: AtomicU8 = AtomicU8::new(255);

/// Минимальный размер файла модели в байтах (чтобы не считать «скачанным» пустой/битый файл)
const MIN_GGUF_SIZE: u64 = 1_000_000;

const OLLAMA_URL: &str = "http://127.0.0.1:11434";

/// Единая директория данных приложения (как в db/crypto)
fn app_data_dir() -> Result<PathBuf, String> {
    let dirs = directories::ProjectDirs::from("com", "kuralbekadilet475", "finance-app")
        .ok_or_else(|| "Could not determine data directory".to_string())?;
    Ok(dirs.data_dir().to_path_buf())
}

fn get_embedded_dir() -> Result<PathBuf, String> {
    let dir = app_data_dir()?.join("embedded_llm");
    fs::create_dir_all(&dir).map_err(|e| format!("Create dir: {}", e))?;
    Ok(dir)
}

fn model_path() -> Result<PathBuf, String> {
    Ok(get_embedded_dir()?.join(GGUF_FILENAME))
}

/// Статус встроенной модели для UI
#[derive(Serialize)]
pub struct EmbeddedLlmStatus {
    /// Файл модели скачан и размер нормальный
    pub downloaded: bool,
    /// 0–100 во время скачивания, иначе null
    pub download_progress: Option<u8>,
    /// Модель зарегистрирована в Ollama (ollama list показывает finance-embedded)
    pub registered_in_ollama: bool,
    /// Ollama API доступен по HTTP
    pub ollama_reachable: bool,
    /// Сообщение об ошибке (например, Ollama не установлен)
    pub error: Option<String>,
}

/// Запустить сервер Ollama в фоне (ollama serve), если он ещё не отвечает.
/// Ждёт 5 секунд после запуска, чтобы сервер успел подняться.
#[cfg(target_os = "macos")]
pub fn try_start_ollama_server() {
    if check_ollama_reachable() {
        return;
    }
    for bin in &["ollama", "/opt/homebrew/bin/ollama", "/usr/local/bin/ollama"] {
        if std::process::Command::new(bin)
            .arg("serve")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
        {
            std::thread::sleep(Duration::from_secs(5));
            return;
        }
    }
    std::thread::sleep(Duration::from_secs(2));
}

#[cfg(not(target_os = "macos"))]
pub fn try_start_ollama_server() {
    if check_ollama_reachable() {
        return;
    }
    let _ = std::process::Command::new("ollama")
        .arg("serve")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    std::thread::sleep(Duration::from_secs(5));
}

/// Проверка доступности Ollama по HTTP (GET /api/tags)
fn check_ollama_reachable() -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let url = format!("{}/api/tags", OLLAMA_URL.trim_end_matches('/'));
    client.get(&url).send().map_or(false, |r| r.status().is_success())
}

/// Текущий статус встроенной модели
pub fn get_embedded_llm_status() -> EmbeddedLlmStatus {
    let download_progress = match DOWNLOAD_PROGRESS.load(Ordering::Relaxed) {
        255 => None,
        p => Some(p),
    };
    let path = model_path();
    let downloaded = path.as_ref().map_or(false, |p| {
        p.exists()
            && fs::metadata(p)
                .map(|m| m.len() >= MIN_GGUF_SIZE)
                .unwrap_or(false)
    });
    let (registered_in_ollama, error) = if downloaded {
        check_ollama_has_model()
    } else {
        (false, None)
    };
    let ollama_reachable = downloaded && registered_in_ollama && !error.is_some() && check_ollama_reachable();
    let error = if ollama_reachable {
        None
    } else if let Some(e) = error {
        Some(e)
    } else if downloaded && registered_in_ollama && !check_ollama_reachable() {
        Some("Ollama не запущен или недоступен. Запустите Ollama и обновите страницу.".to_string())
    } else {
        None
    };
    EmbeddedLlmStatus {
        downloaded,
        download_progress,
        registered_in_ollama,
        ollama_reachable,
        error,
    }
}

/// Проверяем, есть ли модель finance-embedded в Ollama
fn check_ollama_has_model() -> (bool, Option<String>) {
    let out = std::process::Command::new("ollama")
        .args(["list"])
        .output();
    match out {
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return (false, Some("Ollama не установлен. Установите с https://ollama.com".to_string()));
            }
            return (false, Some(format!("Ollama: {}", e)));
        }
        Ok(o) if !o.status.success() => {
            return (false, Some("Ollama не запущен или ошибка".to_string()));
        }
        Ok(o) => {
            let list = String::from_utf8_lossy(&o.stdout);
            let has = list.lines().any(|l| l.contains(EMBEDDED_MODEL_NAME));
            return (has, None);
        }
    }
}

fn is_model_file_valid(path: &PathBuf) -> bool {
    path.exists()
        && fs::metadata(path)
            .map(|m| m.len() >= MIN_GGUF_SIZE)
            .unwrap_or(false)
}

/// Скачать модель с Hugging Face и зарегистрировать в Ollama. Запускается в фоне, возвращает сразу.
pub fn download_and_register_embedded_model() -> Result<(), String> {
    let dir = get_embedded_dir()?;
    let path = dir.join(GGUF_FILENAME);
    if is_model_file_valid(&path) {
        return ensure_registered_in_ollama(&dir);
    }
    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    std::thread::spawn(|| {
        if let Err(e) = do_download_and_register() {
            tracing::warn!("embedded llm download failed: {}", e);
        }
    });
    Ok(())
}

fn do_download_and_register() -> Result<(), String> {
    let dir = get_embedded_dir()?;
    let path = dir.join(GGUF_FILENAME);
    if is_model_file_valid(&path) {
        return ensure_registered_in_ollama(&dir);
    }
    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    DOWNLOAD_PROGRESS.store(0, Ordering::Relaxed);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(HF_URL)
        .send()
        .map_err(|e| format!("Запрос: {}", e))?;
    if !resp.status().is_success() {
        DOWNLOAD_PROGRESS.store(255, Ordering::Relaxed);
        return Err(format!("Сервер вернул {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);
    let mut reader = resp;
    let mut file = fs::File::create(&path).map_err(|e| format!("Создать файл: {}", e))?;
    let mut buf = [0u8; 256 * 1024];
    let mut written: u64 = 0;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        written += n as u64;
        if total > 0 {
            let p = ((written as f64 / total as f64) * 100.0).min(99.0) as u8;
            DOWNLOAD_PROGRESS.store(p, Ordering::Relaxed);
        }
    }
    DOWNLOAD_PROGRESS.store(100, Ordering::Relaxed);
    if !check_ollama_reachable() {
        try_start_ollama_server();
    }
    if let Err(e) = ensure_registered_in_ollama(&dir) {
        if !check_ollama_reachable() {
            std::thread::sleep(Duration::from_secs(3));
            ensure_registered_in_ollama(&dir)?;
        } else {
            return Err(e);
        }
    }
    DOWNLOAD_PROGRESS.store(255, Ordering::Relaxed);
    Ok(())
}

/// Создать Modelfile и выполнить ollama create
fn ensure_registered_in_ollama(dir: &PathBuf) -> Result<(), String> {
    if check_ollama_has_model().0 {
        return Ok(());
    }
    let modelfile = dir.join("Modelfile");
    let from_line = format!("FROM ./{}", GGUF_FILENAME);
    fs::write(&modelfile, from_line).map_err(|e| format!("Modelfile: {}", e))?;
    let status = std::process::Command::new("ollama")
        .args(["create", EMBEDDED_MODEL_NAME, "-f", modelfile.to_str().unwrap_or("Modelfile")])
        .current_dir(dir)
        .status()
        .map_err(|e| format!("Ollama create: {}", e))?;
    if !status.success() {
        return Err("Не удалось зарегистрировать модель в Ollama. Убедитесь, что Ollama запущен.".to_string());
    }
    Ok(())
}
