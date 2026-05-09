use anyhow::{Context, Result};
use regex::Regex;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

use crate::models::{ApkAnalysis, SoFileEntry};

pub fn analyze_apk(file_path: &str) -> Result<ApkAnalysis> {
    let apk_path = PathBuf::from(file_path);
    let file = File::open(&apk_path).with_context(|| format!("failed to open apk: {file_path}"))?;
    let mut archive = ZipArchive::new(file).context("failed to read apk zip archive")?;

    let mut available_abis: Vec<String> = Vec::new();
    let mut so_files: Vec<SoFileEntry> = Vec::new();
    let mut manifest_bytes = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_string();

        if name == "AndroidManifest.xml" {
            entry.read_to_end(&mut manifest_bytes)?;
        }

        if let Some(stripped) = name.strip_prefix("lib/") {
            let segments: Vec<&str> = stripped.split('/').collect();
            if segments.len() == 2 && segments[1].ends_with(".so") {
                let abi = segments[0].to_string();
                if !available_abis.contains(&abi) {
                    available_abis.push(abi.clone());
                }
                so_files.push(SoFileEntry {
                    abi,
                    path: name.clone(),
                    name: segments[1].to_string(),
                    size: entry.size(),
                });
            }
        }
    }

    let preferred_abi = if available_abis.iter().any(|abi| abi == "arm64-v8a") {
        "arm64-v8a".to_string()
    } else {
        available_abis
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    };

    so_files.sort_by(|left, right| {
        let left_rank = (left.abi != "arm64-v8a") as u8;
        let right_rank = (right.abi != "arm64-v8a") as u8;
        left_rank
            .cmp(&right_rank)
            .then_with(|| left.abi.cmp(&right.abi))
            .then_with(|| left.name.cmp(&right.name))
    });

    let manifest_text = decode_binaryish_manifest(&manifest_bytes);
    let package_name = extract_package_name(&manifest_text).unwrap_or_else(|| "unknown.package".to_string());
    let version_name = extract_attribute(&manifest_text, "versionName");
    let app_name = extract_attribute(&manifest_text, "label");
    let hardening_info = detect_hardening(&so_files, &manifest_text);
    let manifest_summary = build_manifest_summary(&manifest_text, &so_files, &available_abis);

    Ok(ApkAnalysis {
        file_path: file_path.to_string(),
        file_name: apk_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown.apk")
            .to_string(),
        package_name,
        app_name,
        package_version: version_name,
        hardening_info,
        preferred_abi,
        available_abis,
        so_files,
        manifest_summary,
    })
}

fn decode_binaryish_manifest(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .map(|ch| if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' { ' ' } else { ch })
        .collect::<String>()
}

fn extract_package_name(manifest: &str) -> Option<String> {
    let regex = Regex::new(r#"package[\s=":>]+([A-Za-z0-9_\.]+)"#).ok()?;
    regex
        .captures(manifest)
        .and_then(|capture| capture.get(1).map(|value| value.as_str().to_string()))
}

fn extract_attribute(manifest: &str, key: &str) -> Option<String> {
    let pattern = format!(r#"{key}[\s=":>]+([A-Za-z0-9_\.\-]+)"#);
    let regex = Regex::new(&pattern).ok()?;
    regex
        .captures(manifest)
        .and_then(|capture| capture.get(1).map(|value| value.as_str().to_string()))
}

fn detect_hardening(so_files: &[SoFileEntry], manifest: &str) -> String {
    let lower_manifest = manifest.to_lowercase();
    let indicators = [
        ("ijiami", "360 Jiagu / Ijiami"),
        ("libjiagu", "360 Jiagu"),
        ("libchaosvmp", "VMP / native protection"),
        ("libshella", "Tencent Legu / shell"),
        ("libexecmain", "Bangcle"),
        ("secneo", "SecNeo"),
        ("libnqshield", "NetQin"),
    ];

    for entry in so_files {
        let lower_name = entry.name.to_lowercase();
        for (needle, label) in indicators {
            if lower_name.contains(needle) {
                return label.to_string();
            }
        }
    }

    if lower_manifest.contains("stubapp") {
        return "StubApp shell / likely packed".to_string();
    }

    "No obvious hardening signature".to_string()
}

fn build_manifest_summary(manifest: &str, so_files: &[SoFileEntry], abis: &[String]) -> Vec<String> {
    let mut notes = Vec::new();
    let lower_manifest = manifest.to_lowercase();

    notes.push(format!("Available ABIs: {}", if abis.is_empty() { "none".to_string() } else { abis.join(", ") }));
    notes.push(format!("Shared libraries discovered: {}", so_files.len()));

    for permission in [
        "android.permission.INTERNET",
        "android.permission.ACCESS_FINE_LOCATION",
        "android.permission.READ_PHONE_STATE",
        "android.permission.QUERY_ALL_PACKAGES",
    ] {
        if manifest.contains(permission) {
            notes.push(format!("Permission requested: {permission}"));
        }
    }

    if lower_manifest.contains("accessibilityservice") {
        notes.push("Accessibility service strings found in manifest payload.".to_string());
    }
    if lower_manifest.contains("vpnservice") {
        notes.push("VPN service strings found in manifest payload.".to_string());
    }
    if lower_manifest.contains("xposed") || lower_manifest.contains("frida") {
        notes.push("Anti-hooking or hook-related markers detected in manifest strings.".to_string());
    }

    notes
}

#[allow(dead_code)]
fn _path_exists(path: &Path) -> bool {
    path.exists()
}
