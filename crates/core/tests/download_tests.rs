//! Unit and integration tests for model manifest, lookup hierarchy, and downloader safety.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use rustmatte_core::download::{
    delete_partial_download, download_model, download_model_with_options, extract_source_host,
    partial_download_info, DownloadOptions,
};
use rustmatte_core::error::CoreError;
use rustmatte_core::manifest::{manifest_entries, validate_manifest};
use rustmatte_core::model::{
    check_model_status_with_custom, delete_model_in_dir, spec_for_tier, ModelKind, ModelSpec,
    ModelStatus, ModelTier, SPEC_FAST, SPEC_QUALITY,
};
use sha2::{Digest, Sha256};

#[test]
fn test_manifest_specifications() {
    for tier in [ModelTier::Quality, ModelTier::Fast] {
        let spec = spec_for_tier(tier);
        assert!(!spec.default_filename.is_empty());
        assert_eq!(spec.expected_sha256.len(), 64);
        assert!(spec.expected_sha256.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(spec.expected_size > 0);
        assert!(!spec.license.is_empty());
        assert!(!spec.source_urls.is_empty());

        // First URL is the project GitHub Release asset; second is the verified HuggingFace fallback mirror
        assert!(spec.source_urls[0]
            .starts_with("https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/"));
        assert!(spec.source_urls[1].starts_with("https://huggingface.co/"));
    }

    assert_eq!(SPEC_QUALITY.expected_size, 191_877_254);
    assert_eq!(SPEC_FAST.expected_size, 4_574_861);
}

#[test]
fn test_extract_source_host() {
    assert_eq!(
        extract_source_host("https://huggingface.co/studioludens/birefnet-lite-512"),
        "huggingface.co"
    );
    assert_eq!(
        extract_source_host("http://127.0.0.1:8080/model.onnx"),
        "127.0.0.1:8080"
    );
    assert_eq!(extract_source_host("invalid-url"), "unknown");
}

#[test]
fn test_url_scheme_security_enforcement() {
    let dummy_spec = ModelSpec {
        tier: ModelTier::Quality,
        id: "test",
        name: "Test",
        default_filename: "test.onnx",
        expected_sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        expected_size: 10,
        license: "MIT",
        source_description: "test",
        source_urls: &["http://evil-attacker.com/model.onnx"],
        width: 512,
        height: 512,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::BiRefNet,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let temp_dir = std::env::temp_dir().join(format!("rustmatte_sec_test_{}", std::process::id()));
    let res = download_model(&dummy_spec, &temp_dir, |_, _| {});

    match res {
        Err(CoreError::InsecureHttpProhibited { url }) => {
            assert!(url.contains("evil-attacker.com"));
        }
        other => panic!("Expected InsecureHttpProhibited, got {other:?}"),
    }
}

#[test]
fn test_lookup_order_missing_reports_expected_cache_path() {
    let non_existent = PathBuf::from("non_existent_folder_xyz/model.onnx");
    let status = check_model_status_with_custom(Some(&non_existent), ModelTier::Quality);

    assert_eq!(
        status,
        ModelStatus::Missing {
            expected_cache_path: non_existent
        }
    );
}

// Spawns a lightweight local HTTP mock server for the duration of a test
fn spawn_mock_server(
    response_header: &'static str,
    response_body: Vec<u8>,
) -> (String, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TcpListener");
    let addr = listener.local_addr().expect("Failed to get local addr");
    let url = format!("http://127.0.0.1:{}/model.onnx", addr.port());

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    let handle = thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");
        loop {
            if shutdown_clone.load(Ordering::Relaxed) {
                break;
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut req_buf = [0u8; 1024];
                    let _ = stream.read(&mut req_buf);
                    let _ = stream.write_all(response_header.as_bytes());
                    let _ = stream.write_all(&response_body);
                    let _ = stream.flush();
                    let _ = stream.shutdown(std::net::Shutdown::Write);
                    let mut drain = [0u8; 128];
                    while let Ok(n) = stream.read(&mut drain) {
                        if n == 0 {
                            break;
                        }
                    }
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    });

    (url, shutdown, handle)
}

#[test]
fn test_mock_server_successful_download_and_atomic_rename() {
    std::env::set_var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST", "1");

    let payload = b"onnx_model_payload_12345";
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let expected_hex = format!("{:x}", hasher.finalize());

    let (url, shutdown, server_thread) = spawn_mock_server(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n",
        payload.to_vec(),
    );

    let test_dir = std::env::temp_dir().join(format!("rustmatte_test_ok_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let spec = ModelSpec {
        tier: ModelTier::Quality,
        id: "test_ok",
        name: "Test OK",
        default_filename: "test_model_ok.onnx",
        expected_sha256: Box::leak(expected_hex.into_boxed_str()),
        expected_size: payload.len() as u64,
        license: "MIT",
        source_description: "mock",
        source_urls: Box::leak(
            vec![Box::leak(url.into_boxed_str()) as &'static str].into_boxed_slice(),
        ),
        width: 512,
        height: 512,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::BiRefNet,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let result = download_model(&spec, &test_dir, |_, _| {});
    shutdown.store(true, Ordering::Relaxed);
    let _ = server_thread.join();

    assert!(result.is_ok(), "Download failed: {result:?}");
    let downloaded_file = result.unwrap();
    assert!(downloaded_file.exists());
    assert_eq!(std::fs::read(&downloaded_file).unwrap(), payload);

    // Verify .part file does not exist
    let part_pattern = format!(".{}.part", spec.default_filename);
    for entry in std::fs::read_dir(&test_dir).unwrap().flatten() {
        assert!(
            !entry.file_name().to_string_lossy().contains(&part_pattern),
            "Leftover .part file found: {:?}",
            entry.path()
        );
    }

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_mock_server_checksum_mismatch_cleans_up() {
    std::env::set_var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST", "1");

    let payload = b"corrupt_model_payload";
    let wrong_hex = "0000000000000000000000000000000000000000000000000000000000000000";

    let (url, shutdown, server_thread) = spawn_mock_server(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n",
        payload.to_vec(),
    );

    let test_dir =
        std::env::temp_dir().join(format!("rustmatte_test_bad_sha_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let spec = ModelSpec {
        tier: ModelTier::Quality,
        id: "test_bad_sha",
        name: "Test Bad SHA",
        default_filename: "test_bad.onnx",
        expected_sha256: wrong_hex,
        expected_size: payload.len() as u64,
        license: "MIT",
        source_description: "mock",
        source_urls: Box::leak(
            vec![Box::leak(url.into_boxed_str()) as &'static str].into_boxed_slice(),
        ),
        width: 512,
        height: 512,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::BiRefNet,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let result = download_model(&spec, &test_dir, |_, _| {});
    shutdown.store(true, Ordering::Relaxed);
    let _ = server_thread.join();

    match result {
        Err(CoreError::ModelChecksumMismatch { .. }) => {
            // Expected checksum mismatch
        }
        other => panic!("Expected ModelChecksumMismatch, got {other:?}"),
    }

    // Verify target file was NOT created and .part file was deleted
    let target_file = test_dir.join(spec.default_filename);
    assert!(!target_file.exists());

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_mock_server_404_not_found() {
    std::env::set_var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST", "1");

    let (url, shutdown, server_thread) = spawn_mock_server(
        "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nFile Not Found",
        Vec::new(),
    );

    let test_dir = std::env::temp_dir().join(format!("rustmatte_test_404_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let spec = ModelSpec {
        tier: ModelTier::Quality,
        id: "test_404",
        name: "Test 404",
        default_filename: "test_404.onnx",
        expected_sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        expected_size: 100,
        license: "MIT",
        source_description: "mock",
        source_urls: Box::leak(
            vec![Box::leak(url.into_boxed_str()) as &'static str].into_boxed_slice(),
        ),
        width: 512,
        height: 512,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::BiRefNet,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let result = download_model(&spec, &test_dir, |_, _| {});
    shutdown.store(true, Ordering::Relaxed);
    let _ = server_thread.join();

    match result {
        Err(CoreError::DownloadHttpStatus { status, .. }) => {
            assert_eq!(status, 404);
        }
        other => panic!("Expected DownloadHttpStatus 404, got {other:?}"),
    }

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_mock_server_truncated_body() {
    std::env::set_var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST", "1");

    let (url, shutdown, server_thread) = spawn_mock_server(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n",
        b"short".to_vec(),
    );

    let test_dir =
        std::env::temp_dir().join(format!("rustmatte_test_trunc_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let spec = ModelSpec {
        tier: ModelTier::Quality,
        id: "test_trunc",
        name: "Test Truncated",
        default_filename: "test_trunc.onnx",
        expected_sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        expected_size: 50, // Expecting 50, received 5
        license: "MIT",
        source_description: "mock",
        source_urls: Box::leak(
            vec![Box::leak(url.into_boxed_str()) as &'static str].into_boxed_slice(),
        ),
        width: 512,
        height: 512,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::BiRefNet,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let result = download_model_with_options(
        &spec,
        &test_dir,
        DownloadOptions {
            retry_delays: &[],
            check_disk: false,
            ..Default::default()
        },
        |_, _| {},
    );
    shutdown.store(true, Ordering::Relaxed);
    let _ = server_thread.join();

    match result {
        Err(CoreError::DownloadTruncated {
            received, expected, ..
        }) => {
            assert_eq!(received, 5);
            assert_eq!(expected, 50);
        }
        other => panic!("Expected DownloadTruncated, got {other:?}"),
    }

    // Truncation keeps .part for subsequent resume until deleted or completed.
    assert!(partial_download_info(&spec, &test_dir).is_some());
    assert!(delete_partial_download(&spec, &test_dir).unwrap());
    assert!(partial_download_info(&spec, &test_dir).is_none());

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_mock_server_oversized_body() {
    std::env::set_var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST", "1");

    let payload = b"this_is_a_longer_payload_than_expected";

    let (url, shutdown, server_thread) = spawn_mock_server(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n",
        payload.to_vec(),
    );

    let test_dir =
        std::env::temp_dir().join(format!("rustmatte_test_oversize_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let spec = ModelSpec {
        tier: ModelTier::Quality,
        id: "test_oversize",
        name: "Test Oversize",
        default_filename: "test_oversize.onnx",
        expected_sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        expected_size: 10, // Expecting 10, received 39
        license: "MIT",
        source_description: "mock",
        source_urls: Box::leak(
            vec![Box::leak(url.into_boxed_str()) as &'static str].into_boxed_slice(),
        ),
        width: 512,
        height: 512,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::BiRefNet,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let result = download_model(&spec, &test_dir, |_, _| {});
    shutdown.store(true, Ordering::Relaxed);
    let _ = server_thread.join();

    match result {
        Err(CoreError::DownloadOversized { expected, .. }) => {
            assert_eq!(expected, 10);
        }
        other => panic!("Expected DownloadOversized, got {other:?}"),
    }

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_manifest_matches_model_specs() {
    let entries = manifest_entries();
    assert_eq!(validate_manifest(entries), Ok(()));
    assert_eq!(entries.len(), 3);
    for entry in entries {
        let spec = spec_for_tier(entry.tier);
        assert_eq!(entry.file_name, spec.default_filename);
        assert_eq!(entry.size_bytes, spec.expected_size);
        assert_eq!(entry.sha256, spec.expected_sha256);
    }
}

fn spawn_range_aware_server(
    full_payload: Vec<u8>,
    support_range: bool,
    truncate_first_at: Option<usize>,
) -> (String, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TcpListener");
    let addr = listener.local_addr().expect("Failed to get local addr");
    let url = format!("http://127.0.0.1:{}/model.onnx", addr.port());

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    let handle = thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let mut req_idx = 0usize;
        loop {
            if shutdown_clone.load(Ordering::Relaxed) {
                break;
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut req_buf = [0u8; 2048];
                    let n = stream.read(&mut req_buf).unwrap_or(0);
                    let req_str = String::from_utf8_lossy(&req_buf[..n]);
                    let range_start = req_str
                        .lines()
                        .find(|l| l.to_ascii_lowercase().starts_with("range:"))
                        .and_then(|l| l.split('=').nth(1))
                        .and_then(|v| v.split('-').next())
                        .and_then(|s| s.trim().parse::<usize>().ok());

                    if req_idx == 0 {
                        if let Some(cut) = truncate_first_at {
                            let hdr =
                                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n";
                            let _ = stream.write_all(hdr.as_bytes());
                            let _ = stream.write_all(&full_payload[..cut]);
                            let _ = stream.flush();
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            req_idx += 1;
                            continue;
                        }
                    }

                    if support_range {
                        if let Some(start) = range_start {
                            let total = full_payload.len();
                            let end = total.saturating_sub(1);
                            let hdr = format!(
                                "HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nContent-Range: bytes {start}-{end}/{total}\r\n\r\n"
                            );
                            let _ = stream.write_all(hdr.as_bytes());
                            let _ = stream.write_all(&full_payload[start..]);
                            let _ = stream.flush();
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            req_idx += 1;
                            continue;
                        }
                    }

                    let hdr = "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n";
                    let _ = stream.write_all(hdr.as_bytes());
                    let _ = stream.write_all(&full_payload);
                    let _ = stream.flush();
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    req_idx += 1;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    });

    (url, shutdown, handle)
}

#[test]
fn test_mock_server_resume_206_after_truncation() {
    std::env::set_var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST", "1");

    let payload = b"0123456789abcdef_resumed_model_payload_xyz".to_vec();
    let mut hasher = Sha256::new();
    hasher.update(&payload);
    let expected_hex = format!("{:x}", hasher.finalize());

    let (url, shutdown, server_thread) = spawn_range_aware_server(payload.clone(), true, Some(16));
    let test_dir =
        std::env::temp_dir().join(format!("rustmatte_test_resume_206_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let spec = ModelSpec {
        tier: ModelTier::Fast,
        id: "test_resume_206",
        name: "Test Resume 206",
        default_filename: "resume_206.onnx",
        expected_sha256: Box::leak(expected_hex.into_boxed_str()),
        expected_size: payload.len() as u64,
        license: "Apache-2.0",
        source_description: "mock",
        source_urls: Box::leak(
            vec![Box::leak(url.into_boxed_str()) as &'static str].into_boxed_slice(),
        ),
        width: 320,
        height: 320,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::U2Netp,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let delays = [std::time::Duration::from_millis(5)];
    let result = download_model_with_options(
        &spec,
        &test_dir,
        DownloadOptions {
            retry_delays: &delays,
            check_disk: false,
            ..Default::default()
        },
        |_, _| {},
    );
    shutdown.store(true, Ordering::Relaxed);
    let _ = server_thread.join();

    let path = result.expect("206 resume should complete cleanly");
    assert_eq!(std::fs::read(&path).unwrap(), payload);
    assert!(partial_download_info(&spec, &test_dir).is_none());

    let deleted = delete_model_in_dir(&spec, &test_dir).unwrap();
    assert!(deleted.is_some());
    assert!(!path.exists());

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_mock_server_resume_falls_back_to_200_cleanly() {
    std::env::set_var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST", "1");

    let payload = b"full_payload_without_range_support_987654321".to_vec();
    let mut hasher = Sha256::new();
    hasher.update(&payload);
    let expected_hex = format!("{:x}", hasher.finalize());

    let (url, shutdown, server_thread) = spawn_range_aware_server(payload.clone(), false, Some(12));
    let test_dir =
        std::env::temp_dir().join(format!("rustmatte_test_resume_200_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let spec = ModelSpec {
        tier: ModelTier::Fast,
        id: "test_resume_200",
        name: "Test Resume 200",
        default_filename: "resume_200.onnx",
        expected_sha256: Box::leak(expected_hex.into_boxed_str()),
        expected_size: payload.len() as u64,
        license: "Apache-2.0",
        source_description: "mock",
        source_urls: Box::leak(
            vec![Box::leak(url.into_boxed_str()) as &'static str].into_boxed_slice(),
        ),
        width: 320,
        height: 320,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::U2Netp,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let delays = [std::time::Duration::from_millis(5)];
    let result = download_model_with_options(
        &spec,
        &test_dir,
        DownloadOptions {
            retry_delays: &delays,
            check_disk: false,
            ..Default::default()
        },
        |_, _| {},
    );
    shutdown.store(true, Ordering::Relaxed);
    let _ = server_thread.join();

    let path = result.expect("200 fallback should restart cleanly");
    assert_eq!(std::fs::read(&path).unwrap(), payload);

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_download_cancel_keeps_partial_and_disk_check_blocks_low_space() {
    let payload = b"disk_and_cancel_payload_1234567890".to_vec();
    let mut hasher = Sha256::new();
    hasher.update(&payload);
    let expected_hex = format!("{:x}", hasher.finalize());

    let spec = ModelSpec {
        tier: ModelTier::Fast,
        id: "test_disk_cancel",
        name: "Test Disk Cancel",
        default_filename: "disk_cancel.onnx",
        expected_sha256: Box::leak(expected_hex.into_boxed_str()),
        expected_size: 1_000,
        license: "Apache-2.0",
        source_description: "mock",
        source_urls: &["https://example.invalid/model.onnx"],
        width: 320,
        height: 320,
        input_name: "in",
        output_name: "out",
        kind: ModelKind::U2Netp,
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    };

    let test_dir =
        std::env::temp_dir().join(format!("rustmatte_test_disk_cancel_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    // 1. Disk check blocks when available < size + 10% (1100 bytes)
    let low_disk = |_: &std::path::Path| Some(1_099);
    let res_disk = download_model_with_options(
        &spec,
        &test_dir,
        DownloadOptions {
            check_disk: true,
            disk_probe: Some(&low_disk),
            ..Default::default()
        },
        |_, _| {},
    );
    match res_disk {
        Err(CoreError::InsufficientDiskSpace {
            required_bytes,
            available_bytes,
            ..
        }) => {
            assert_eq!(required_bytes, 1_100);
            assert_eq!(available_bytes, 1_099);
        }
        other => panic!("Expected InsufficientDiskSpace, got {other:?}"),
    }

    // 2. Cancellation returns DownloadCancelled without deleting pre-existing .part
    std::fs::create_dir_all(&test_dir).unwrap();
    let part = test_dir.join(".disk_cancel.onnx.part");
    let meta = test_dir.join(".disk_cancel.onnx.part.meta.json");
    std::fs::write(&part, b"partial_bytes").unwrap();
    std::fs::write(
        &meta,
        format!(
            r#"{{"url":"https://example.invalid/model.onnx","expected_size":1000,"expected_sha256":"{}"}}"#,
            spec.expected_sha256
        ),
    )
    .unwrap();

    let cancel = AtomicBool::new(true);
    let res_cancel = download_model_with_options(
        &spec,
        &test_dir,
        DownloadOptions {
            cancel: Some(&cancel),
            check_disk: false,
            ..Default::default()
        },
        |_, _| {},
    );
    assert!(matches!(
        res_cancel,
        Err(CoreError::DownloadCancelled { .. })
    ));
    assert!(part.exists());
    assert!(partial_download_info(&spec, &test_dir).is_some());

    assert!(delete_partial_download(&spec, &test_dir).unwrap());
    assert!(!part.exists());

    let _ = std::fs::remove_dir_all(&test_dir);
}
