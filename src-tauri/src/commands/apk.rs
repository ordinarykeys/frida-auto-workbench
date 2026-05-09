use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoFile {
    pub name: String,
    pub arch: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApkAnalysisResult {
    pub package_name: String,
    pub version_name: String,
    pub version_code: String,
    pub min_sdk: String,
    pub target_sdk: String,
    pub hardening: String,
    pub architectures: Vec<String>,
    pub preferred_arch: String,
    pub so_files: Vec<SoFile>,
    pub so_count: usize,
    pub apk_size: u64,
}

fn detect_hardening(so_names: &[String], asset_names: &[String]) -> String {
    let all: Vec<&String> = so_names.iter().chain(asset_names.iter()).collect();
    let any = |needle: &str| all.iter().any(|s| s.to_lowercase().contains(needle));

    if any("libjiagu") || any("libjiagu_64") || any("libjiagu_a64") {
        return "360 加固".into();
    }
    if any("libshellx") || any("libshella") || any("libtup") || any("libtosprotection") {
        return "腾讯乐固/御安全".into();
    }
    if any("libsecmain") || any("libsecshell") || any("libfakedex") {
        return "梆梆安全".into();
    }
    if any("libdexhelper") || any("libdexhelper-x86") {
        return "梆梆/爱加密".into();
    }
    if any("libnqshield") {
        return "网秦/通付盾".into();
    }
    if any("libchaosvmp") || any("libddog") || any("libfdog") {
        return "娜迦加固".into();
    }
    if any("libapssecsdk") || any("libapssec") {
        return "阿里聚安全".into();
    }
    if any("libBugly") {
        return "Bugly (含混淆)".into();
    }
    if any("libmobisec") {
        return "几维安全".into();
    }
    if any("libreajni") || any("libexec.so") {
        return "爱加密".into();
    }
    "未加固".into()
}

fn parse_package_from_manifest(data: &[u8]) -> (String, String, String, String, String) {
    let mut package = String::from("(无法解析)");
    let mut version_name = String::new();
    let mut version_code = String::new();
    let mut min_sdk = String::new();
    let mut target_sdk = String::new();

    if data.len() < 8 {
        return (package, version_name, version_code, min_sdk, target_sdk);
    }

    // Parse Android binary XML string pool to find the package name heuristically.
    // Format: header(8) magic 0x00080003, then string pool chunk with offsets.
    let mut strings: Vec<String> = Vec::new();
    let mut i = 0;
    while i + 4 < data.len() {
        if data[i] == 0x01 && data[i + 1] == 0x00 && data[i + 2] == 0x1C && data[i + 3] == 0x00 {
            // String pool chunk header found
            if i + 28 > data.len() { break; }
            let string_count = u32::from_le_bytes([data[i+8], data[i+9], data[i+10], data[i+11]]) as usize;
            let strings_start = u32::from_le_bytes([data[i+20], data[i+21], data[i+22], data[i+23]]) as usize;
            let pool_base = i + strings_start;
            let offset_table = i + 28;

            for s in 0..string_count.min(4096) {
                let off_pos = offset_table + s * 4;
                if off_pos + 4 > data.len() { break; }
                let off = u32::from_le_bytes([data[off_pos], data[off_pos+1], data[off_pos+2], data[off_pos+3]]) as usize;
                let str_pos = pool_base + off;
                if str_pos + 2 > data.len() { break; }

                // UTF-16 length (modified): high bit set => 4 bytes length
                let mut len_pos = str_pos;
                let len = {
                    let l0 = u16::from_le_bytes([data[len_pos], data[len_pos+1]]) as usize;
                    if l0 & 0x8000 != 0 {
                        len_pos += 2;
                        if len_pos + 2 > data.len() { break; }
                        ((l0 & 0x7FFF) << 16) | u16::from_le_bytes([data[len_pos], data[len_pos+1]]) as usize
                    } else { l0 }
                };
                len_pos += 2;
                let byte_len = len * 2;
                if len_pos + byte_len > data.len() { break; }
                let bytes = &data[len_pos..len_pos + byte_len];
                let u16s: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                let s = String::from_utf16_lossy(&u16s);
                strings.push(s);
            }
            break;
        }
        i += 1;
    }

    // Heuristic: package name is typically a string that has at least 2 dots and lowercase letters, not starting with android
    for s in &strings {
        if s.matches('.').count() >= 2
            && s.len() < 80
            && s.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
            && !s.starts_with("android.")
            && !s.starts_with("androidx.")
            && !s.contains('/')
            && !s.contains(':')
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
        {
            package = s.clone();
            break;
        }
    }

    // Find versionName, versionCode strings (best-effort)
    if let Some(idx) = strings.iter().position(|s| s == "versionName") {
        if let Some(next) = strings.get(idx + 1) { version_name = next.clone(); }
    }
    if let Some(idx) = strings.iter().position(|s| s == "versionCode") {
        if let Some(next) = strings.get(idx + 1) { version_code = next.clone(); }
    }
    if let Some(idx) = strings.iter().position(|s| s == "minSdkVersion") {
        if let Some(next) = strings.get(idx + 1) { min_sdk = next.clone(); }
    }
    if let Some(idx) = strings.iter().position(|s| s == "targetSdkVersion") {
        if let Some(next) = strings.get(idx + 1) { target_sdk = next.clone(); }
    }

    (package, version_name, version_code, min_sdk, target_sdk)
}

#[tauri::command]
pub async fn analyze_apk(path: String) -> Result<ApkAnalysisResult, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("文件不存在: {}", path));
    }
    let apk_size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);

    let file = File::open(p).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut so_files: Vec<SoFile> = Vec::new();
    let mut arch_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut so_names: Vec<String> = Vec::new();
    let mut asset_names: Vec<String> = Vec::new();
    let mut manifest_data: Vec<u8> = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();

        if name == "AndroidManifest.xml" {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).ok();
            manifest_data = buf;
            continue;
        }

        if name.starts_with("lib/") && name.ends_with(".so") {
            let parts: Vec<&str> = name.split('/').collect();
            if parts.len() == 3 {
                let arch = parts[1].to_string();
                let so_name = parts[2].to_string();
                arch_set.insert(arch.clone());
                so_names.push(so_name.clone());
                so_files.push(SoFile { name: so_name, arch });
            }
        }

        if name.starts_with("assets/") {
            asset_names.push(name);
        }
    }

    let (package_name, version_name, version_code, min_sdk, target_sdk) =
        parse_package_from_manifest(&manifest_data);
    let hardening = detect_hardening(&so_names, &asset_names);

    so_files.sort_by(|a, b| {
        let arch_priority = |s: &str| match s {
            "arm64-v8a" => 0,
            "armeabi-v7a" => 1,
            "x86_64" => 2,
            "x86" => 3,
            _ => 4,
        };
        arch_priority(&a.arch).cmp(&arch_priority(&b.arch))
            .then(a.name.cmp(&b.name))
    });
    so_files.dedup_by(|a, b| a.name == b.name && a.arch == b.arch);

    let architectures: Vec<String> = arch_set.into_iter().collect();
    let preferred_arch = if architectures.iter().any(|a| a == "arm64-v8a") {
        "arm64-v8a".to_string()
    } else if architectures.iter().any(|a| a == "armeabi-v7a") {
        "armeabi-v7a".to_string()
    } else {
        architectures.first().cloned().unwrap_or_default()
    };

    let so_count = so_files.len();

    Ok(ApkAnalysisResult {
        package_name,
        version_name,
        version_code,
        min_sdk,
        target_sdk,
        hardening,
        architectures,
        preferred_arch,
        so_files,
        so_count,
        apk_size,
    })
}
