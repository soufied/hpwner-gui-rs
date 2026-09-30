use std::io::Write;
use std::process::{Command, Stdio};
use std::time::SystemTime;

pub enum CommandRequest {
    Decrypt {
        link: String,
    },
    EncryptHapp {
        url: String,
        format: String,
    },
    EncryptV2 {
        url: String,
        format: String,
        key: Option<String>,
    },
    Fetch {
        url: String,
        hwid: Option<String>,
        ua: Option<String>,
    },
    ConvertUrl {
        url: String,
        hwid: Option<String>,
        ua: Option<String>,
        modes: Vec<String>,
    },
    ConvertFile {
        path: String,
        modes: Vec<String>,
    },
    ConvertStdin {
        content: String,
        modes: Vec<String>,
    },
}

fn extract_host(url: &str) -> Option<String> {
    let without_scheme = url.split("://").nth(1).unwrap_or(url);
    let without_userinfo = without_scheme.rsplit('@').next().unwrap_or(without_scheme);
    let authority = without_userinfo
        .split('/')
        .next()
        .unwrap_or(without_userinfo)
        .split('?')
        .next()
        .unwrap_or(without_userinfo)
        .split('#')
        .next()
        .unwrap_or(without_userinfo);
    let host = authority.split(':').next().unwrap_or(authority);
    let trimmed = host.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_lowercase())
    }
}

fn spilled_fetch_response_path(url: &str) -> Option<std::path::PathBuf> {
    let host = extract_host(url)?;
    Some(std::path::PathBuf::from(format!("hpwnresp_{}.txt", host)))
}

fn take_spilled_fetch_response(path: &std::path::Path, not_before: SystemTime) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let fresh_enough = modified >= not_before
        || not_before
            .duration_since(modified)
            .map(|age| age.as_secs() < 5)
            .unwrap_or(true);
    if !fresh_enough {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let content = String::from_utf8_lossy(&bytes).to_string();
    let _ = std::fs::remove_file(path);
    Some(content)
}

pub fn execute(exe_path: &str, req: CommandRequest) -> Result<String, String> {
    let mut fetch_url: Option<String> = None;
    let (args, stdin_data) = match req {
        CommandRequest::Decrypt { link } => (vec![link], None),
        CommandRequest::EncryptHapp { url, format } => (vec![url, format], None),
        CommandRequest::EncryptV2 { url, format, key } => {
            let mut a = vec![url, format];
            if let Some(k) = key {
                if !k.is_empty() {
                    a.push(k);
                }
            }
            (a, None)
        }
        CommandRequest::Fetch { url, hwid, ua } => {
            fetch_url = Some(url.clone());
            let mut a = vec![url];
            if let Some(h) = hwid {
                let trimmed = h.trim();
                if !trimmed.is_empty() {
                    a.push("hwid".to_string());
                    a.push(trimmed.to_string());
                }
            }
            if let Some(u) = ua {
                let trimmed = u.trim();
                if !trimmed.is_empty() {
                    a.push("ua".to_string());
                    a.push(trimmed.to_string());
                }
            }
            if a.len() == 1 {
                a.push("fetch".to_string());
            }
            (a, None)
        }
        CommandRequest::ConvertUrl {
            url,
            hwid,
            ua,
            modes,
        } => {
            let mut a = vec![url];
            if let Some(h) = hwid {
                let trimmed = h.trim();
                if !trimmed.is_empty() {
                    a.push("hwid".to_string());
                    a.push(trimmed.to_string());
                }
            }
            if let Some(u) = ua {
                let trimmed = u.trim();
                if !trimmed.is_empty() {
                    a.push("ua".to_string());
                    a.push(trimmed.to_string());
                }
            }
            a.extend(modes);
            (a, None)
        }
        CommandRequest::ConvertFile { path, modes } => {
            let mut a = vec![path];
            a.extend(modes);
            (a, None)
        }
        CommandRequest::ConvertStdin { content, modes } => (modes, Some(content)),
    };

    let spill_path = fetch_url.as_deref().and_then(spilled_fetch_response_path);
    let spawn_time = SystemTime::now();

    let mut cmd = Command::new(exe_path);
    cmd.args(&args);

    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Could not start '{}': {}", exe_path, e))?;

    if let Some(data) = stdin_data {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(data.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Execution error: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        if let Some(path) = spill_path {
            if let Some(spilled) = take_spilled_fetch_response(&path, spawn_time) {
                return Ok(spilled);
            }
        }
        if stdout.is_empty() && !stderr.is_empty() {
            Ok(stderr)
        } else {
            Ok(stdout)
        }
    } else {
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".to_string());
        if !stderr.trim().is_empty() {
            Err(format!("Process error (exit code {}):\n{}", code, stderr.trim()))
        } else if !stdout.trim().is_empty() {
            Err(format!("Process error (exit code {}):\n{}", code, stdout.trim()))
        } else {
            Err(format!("Process exited with failure code {}", code))
        }
    }
}
