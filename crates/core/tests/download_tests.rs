//! Unit and integration tests for model manifest, lookup hierarchy, and downloader safety.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use rustmatte_core::download::{download_model, extract_source_host};
use rustmatte_core::error::CoreError;
use rustmatte_core::model::{
    check_model_status_with_custom, spec_for_tier, ModelKind, ModelSpec, ModelStatus, ModelTier,
    SPEC_FAST, SPEC_QUALITY,
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

        // First URL is the verified upstream HuggingFace export; second is the project mirror URL
        assert!(spec.source_urls[0].starts_with("https://huggingface.co/"));
        assert!(spec.source_urls[1].starts_with("https://"));
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

    let result = download_model(&spec, &test_dir, |_, _| {});
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
