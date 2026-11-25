export type Lang = "en" | "id";

export interface Translations {
  appTitle: string;
  appSubtitle: string;
  offlineBadge: string;
  aboutBtn: string;
  languageLabel: string;
  themeLabel: string;
  themeSystem: string;
  themeLight: string;
  themeDark: string;

  dropTitle: string;
  dropSubtitle: string;
  browsePhotos: string;
  browseFolder: string;

  queueTitle: string;
  clearQueue: string;
  statusWaiting: string;
  statusProcessing: string;
  statusDone: string;
  statusWarning: string;
  statusFailed: string;

  optionsTitle: string;
  tierLabel: string;
  tierQualityName: string;
  tierQualityDesc: string;
  tierFastName: string;
  tierFastDesc: string;

  refineLabel: string;
  refineNone: string;
  refineLight: string;
  refineStandard: string;

  deviceLabel: string;
  deviceReadOnlyLine: string;
  deviceAuto: string;
  deviceCpu: string;
  deviceDirectml: string;
  deviceCoreml: string;

  maxSideLabel: string;
  maxSideOriginal: string;
  maxSide2048: string;
  maxSide1024: string;
  maxSide512: string;

  backgroundLabel: string;
  bgTransparent: string;
  bgSolid: string;
  bgColorPickerLabel: string;

  outputFolderLabel: string;
  outputFolderAuto: string;
  chooseFolder: string;
  resetFolder: string;

  neverUseNetworkLabel: string;
  neverUseNetworkHint: string;

  removeBackgroundBtn: string;
  saveBtn: string;
  saveAllBtn: string;
  cancelBtn: string;
  openOutputFolderBtn: string;
  addMoreBtn: string;

  progressLoadingModel: string;
  progressProcessing: string;
  progressDownloading: string;
  stateEmptyHint: string;
  stateDoneSummary: string;
  statePartialSummary: string;
  stateCancelledSummary: string;
  oomWarningBanner: string;
  damagedWarningPrefix: string;
  rejectedFilesBanner: string;

  originalLabel: string;
  cutoutLabel: string;
  compareToggleBtn: string;
  compareToggleActiveBtn: string;
  compareToggleHint: string;
  sliderAriaLabel: string;
  zoomIn: string;
  zoomOut: string;
  zoomReset: string;
  panHint: string;
  thumbPlaceholder: string;
  statsTitle: string;
  activeDeviceLabel: string;
  dimensionsLabel: string;
  inferenceTimeLabel: string;
  totalTimeLabel: string;

  filterSupportedImages: string;
  filterOnnxModel: string;
  filterPngImage: string;
  dialogBrowsePhotosTitle: string;
  dialogBrowseFolderTitle: string;
  dialogOutputFolderTitle: string;
  dialogImportModelTitle: string;
  dialogSavePngTitle: string;

  firstRunTitle: string;
  firstRunIntro: string;
  firstRunFileLabel: string;
  firstRunSizeLabel: string;
  firstRunHostLabel: string;
  firstRunFallbackLabel: string;
  firstRunLicenseLabel: string;
  firstRunDownloadBtn: string;
  firstRunImportBtn: string;
  firstRunRetryBtn: string;
  firstRunCancelBtn: string;
  firstRunBlockedOffline: string;
  modelReadyBadge: string;
  modelMissingBadge: string;
  setupModelBtn: string;

  aboutTitle: string;
  aboutVersionLabel: string;
  aboutLicenseLabel: string;
  aboutNetworkTitle: string;
  aboutNetworkBody: string;
  aboutModelsTitle: string;
  aboutThirdPartyTitle: string;
  copyDiagnosticsBtn: string;
  diagnosticsCopied: string;
  closeBtn: string;

  errorDetailsLabel: string;
  dismissBtn: string;
  savedSingleNotice: string;
  savedAllNotice: string;
  bannerJobDone: string;
  bannerJobPartial: string;
  bannerSavedSingle: string;
  bannerSavedAll: string;
  rejectedReasonInaccessible: string;
  rejectedReasonReadDir: string;
  rejectedReasonNotFileOrDir: string;
  rejectedReasonUnsupported: string;
  warningDamagedInput: string;

  errOfflineTitle: string;
  errOfflineFix: string;
  errNetworkTitle: string;
  errNetworkFix: string;
  errChecksumTitle: string;
  errChecksumFix: string;
  errDiskTitle: string;
  errDiskFix: string;
  errOomTitle: string;
  errOomFix: string;
  errUnsupportedTitle: string;
  errUnsupportedFix: string;
  errOverwriteTitle: string;
  errOverwriteFix: string;
  errGenericTitle: string;
  errGenericFix: string;

  shortcutsHint: string;
}

export const dictionaries: Record<Lang, Translations> = {
  en: {
    appTitle: "RustMatte",
    appSubtitle: "Offline Background Remover",
    offlineBadge: "100% Local Processing",
    aboutBtn: "About & Licenses",
    languageLabel: "Language",
    themeLabel: "Theme",
    themeSystem: "Follow System",
    themeLight: "Light",
    themeDark: "Dark",

    dropTitle: "Drop photos or a folder here, or click to browse",
    dropSubtitle: "Supports JPG, JPEG, PNG, WebP, and BMP",
    browsePhotos: "Browse Photos",
    browseFolder: "Browse Folder",

    queueTitle: "Photo Queue",
    clearQueue: "Clear",
    statusWaiting: "Waiting",
    statusProcessing: "Processing",
    statusDone: "Done",
    statusWarning: "Done with warning",
    statusFailed: "Failed",

    optionsTitle: "Options",
    tierLabel: "Model Quality",
    tierQualityName: "Quality (BiRefNet Lite 512)",
    tierQualityDesc: "Best detail, slower (about 3 s per photo on a mid-range PC).",
    tierFastName: "Fast (U-2-Netp)",
    tierFastDesc:
      "Very quick (under 1 s); best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds.",

    refineLabel: "Edge Refinement",
    refineNone: "None (raw mask)",
    refineLight: "Light (guided filter)",
    refineStandard: "Standard (guided filter + defringing)",

    deviceLabel: "Hardware Acceleration",
    deviceReadOnlyLine: "Processing: CPU",
    deviceAuto: "Auto (GPU with CPU fallback)",
    deviceCpu: "CPU only",
    deviceDirectml: "DirectML GPU (Windows)",
    deviceCoreml: "CoreML (macOS)",

    maxSideLabel: "Max Output Size",
    maxSideOriginal: "Original size (full resolution)",
    maxSide2048: "Max 2048 px",
    maxSide1024: "Max 1024 px (saves RAM)",
    maxSide512: "Max 512 px",

    backgroundLabel: "Background (PNG Export)",
    bgTransparent: "Transparent",
    bgSolid: "Solid Color",
    bgColorPickerLabel: "Background color",

    outputFolderLabel: "Output Folder",
    outputFolderAuto: "Default (rustmatte-output next to photos)",
    chooseFolder: "Choose Folder",
    resetFolder: "Use Default",

    neverUseNetworkLabel: "Never use the network",
    neverUseNetworkHint: "Blocks all model downloads; only local cached or imported files can be used.",

    removeBackgroundBtn: "Remove background",
    saveBtn: "Save",
    saveAllBtn: "Save all",
    cancelBtn: "Cancel",
    openOutputFolderBtn: "Open output folder",
    addMoreBtn: "Add photos",

    progressLoadingModel: "Loading AI model into memory...",
    progressProcessing: "Removing backgrounds...",
    progressDownloading: "Downloading AI model...",
    stateEmptyHint: "Add one or more photos to begin.",
    stateDoneSummary: "Finished cleanly.",
    statePartialSummary: "Finished with some failed photos. Check the error list below.",
    stateCancelledSummary: "Stopped before finishing the queue.",
    oomWarningBanner:
      "Large image detected (16 MP or higher). Consider setting Max Output Size to 1024 px or 2048 px if memory is tight.",
    damagedWarningPrefix: "Image file had a non-fatal warning",
    rejectedFilesBanner: "Skipped unsupported or unreadable items",

    originalLabel: "Original",
    cutoutLabel: "Cutout",
    compareToggleBtn: "Compare with original",
    compareToggleActiveBtn: "Show cutout only",
    compareToggleHint: "Compare with original photo (C)",
    sliderAriaLabel: "Before and after comparison position",
    zoomIn: "Zoom in",
    zoomOut: "Zoom out",
    zoomReset: "Reset view",
    panHint: "Drag image to pan while zoomed",
    thumbPlaceholder: "IMG",
    statsTitle: "Run Summary",
    activeDeviceLabel: "Device used",
    dimensionsLabel: "Resolution",
    inferenceTimeLabel: "Inference",
    totalTimeLabel: "Total",

    filterSupportedImages: "Supported Images",
    filterOnnxModel: "ONNX Model",
    filterPngImage: "PNG Image",
    dialogBrowsePhotosTitle: "Select Photos",
    dialogBrowseFolderTitle: "Select Folder",
    dialogOutputFolderTitle: "Choose Output Folder",
    dialogImportModelTitle: "Select ONNX Model File",
    dialogSavePngTitle: "Save Cutout PNG",

    firstRunTitle: "One-Time AI Model Setup",
    firstRunIntro:
      "RustMatte needs to download its AI model once. After that it works fully offline.",
    firstRunFileLabel: "File",
    firstRunSizeLabel: "Size",
    firstRunHostLabel: "Source host",
    firstRunFallbackLabel: "fallback",
    firstRunLicenseLabel: "Weight license",
    firstRunDownloadBtn: "Download",
    firstRunImportBtn: "Choose a file I already have",
    firstRunRetryBtn: "Retry Download",
    firstRunCancelBtn: "Cancel",
    firstRunBlockedOffline:
      "'Never use the network' is switched on. Turn it off in Options or choose a local .onnx file.",
    modelReadyBadge: "Model ready offline",
    modelMissingBadge: "Model needed",
    setupModelBtn: "Set up model",

    aboutTitle: "About RustMatte & Licenses",
    aboutVersionLabel: "Version",
    aboutLicenseLabel: "App License",
    aboutNetworkTitle: "Network Use",
    aboutNetworkBody:
      "RustMatte processes all images 100% locally. It contains zero telemetry, analytics, or image uploads. Network access occurs only when you explicitly confirm a one-time AI model download, and can be disabled completely via 'Never use the network' in Options.",
    aboutModelsTitle: "AI Models & Dataset Notes",
    aboutThirdPartyTitle: "Third-Party Licenses",
    copyDiagnosticsBtn: "Copy diagnostic info",
    diagnosticsCopied: "Diagnostic info copied to clipboard (no file paths included).",
    closeBtn: "Close",

    errorDetailsLabel: "Technical details",
    dismissBtn: "Dismiss",
    savedSingleNotice: "Saved PNG to",
    savedAllNotice: "Saved cutouts to",
    bannerJobDone: "Finished cleanly ({processed}/{total} in {sec}s).",
    bannerJobPartial: "Finished with some failures ({processed}/{total} ok in {sec}s).",
    bannerSavedSingle: "Saved PNG to {path}",
    bannerSavedAll: "Saved {count} cutouts to {dir}",
    rejectedReasonInaccessible: "File or folder could not be accessed",
    rejectedReasonReadDir: "Failed to read folder contents",
    rejectedReasonNotFileOrDir: "Item is not a regular file or folder",
    rejectedReasonUnsupported: "Unsupported image format (use JPG, JPEG, PNG, WebP, or BMP)",
    warningDamagedInput: "Image file may be damaged or incomplete",

    errOfflineTitle: "Network access is turned off in Settings.",
    errOfflineFix:
      "Turn off 'Never use the network' to download, or click 'Choose a file I already have' to import an .onnx model from disk.",
    errNetworkTitle: "Could not download the AI model.",
    errNetworkFix:
      "Check your internet connection or firewall and click Retry, or import the .onnx file manually.",
    errChecksumTitle: "The model file failed integrity verification.",
    errChecksumFix:
      "The file may be incomplete or from a different version. Try downloading again or select the exact pinned .onnx file.",
    errDiskTitle: "Could not read or write files on disk.",
    errDiskFix:
      "Check available disk space and make sure you have write permission for the chosen folder.",
    errOomTitle: "Not enough memory to process this image at full size.",
    errOomFix:
      "Set Max Output Size to 1024 px or 2048 px, close heavy background apps, or switch to the Fast model.",
    errUnsupportedTitle: "This file could not be decoded as a supported image.",
    errUnsupportedFix:
      "Use a valid JPG, JPEG, PNG, WebP, or BMP photo.",
    errOverwriteTitle: "Cannot save over the original photo.",
    errOverwriteFix:
      "Choose a different filename or output folder so your original photo stays untouched.",
    errGenericTitle: "Something went wrong during processing.",
    errGenericFix:
      "Review the technical details below or try again with a smaller Max Output Size.",

    shortcutsHint:
      "Shortcuts: Ctrl+O open, Ctrl+Enter process, C compare, Ctrl+S save, Up/Down switch photo, Esc cancel",
  },
  id: {
    appTitle: "RustMatte",
    appSubtitle: "Penghapus Latar Belakang Offline",
    offlineBadge: "100% Proses Lokal",
    aboutBtn: "Tentang & Lisensi",
    languageLabel: "Bahasa",
    themeLabel: "Tema",
    themeSystem: "Ikuti Sistem",
    themeLight: "Terang",
    themeDark: "Gelap",

    dropTitle: "Tarik foto atau folder ke sini, atau klik untuk memilih",
    dropSubtitle: "Mendukung JPG, JPEG, PNG, WebP, dan BMP",
    browsePhotos: "Pilih Foto",
    browseFolder: "Pilih Folder",

    queueTitle: "Antrean Foto",
    clearQueue: "Bersihkan",
    statusWaiting: "Menunggu",
    statusProcessing: "Diproses",
    statusDone: "Selesai",
    statusWarning: "Selesai (ada peringatan)",
    statusFailed: "Gagal",

    optionsTitle: "Pengaturan",
    tierLabel: "Kualitas Model",
    tierQualityName: "Quality (BiRefNet Lite 512)",
    tierQualityDesc: "Detail paling halus, lebih lambat (sekitar 3 detik per foto di PC menengah).",
    tierFastName: "Fast (U-2-Netp)",
    tierFastDesc:
      "Sangat cepat (di bawah 1 detik); cocok untuk foto orang atau produk berlatar bersih; bisa meninggalkan noda halus pada bulu atau latar ramai.",

    refineLabel: "Kehalusan Tepi",
    refineNone: "Tanpa penghalusan",
    refineLight: "Ringan (guided filter)",
    refineStandard: "Standar (guided filter + bersih warna tepi)",

    deviceLabel: "Perangkat Proses",
    deviceReadOnlyLine: "Perangkat proses: CPU",
    deviceAuto: "Otomatis (GPU, cadangan CPU)",
    deviceCpu: "CPU saja",
    deviceDirectml: "GPU DirectML (Windows)",
    deviceCoreml: "CoreML (macOS)",

    maxSideLabel: "Batas Resolusi Output",
    maxSideOriginal: "Ukuran asli (resolusi penuh)",
    maxSide2048: "Maksimal 2048 px",
    maxSide1024: "Maksimal 1024 px (hemat RAM)",
    maxSide512: "Maksimal 512 px",

    backgroundLabel: "Latar Belakang (Ekspor PNG)",
    bgTransparent: "Transparan",
    bgSolid: "Warna Polos",
    bgColorPickerLabel: "Pilih warna latar",

    outputFolderLabel: "Folder Simpan",
    outputFolderAuto: "Bawaan (rustmatte-output di sebelah foto)",
    chooseFolder: "Pilih Folder",
    resetFolder: "Pakai Bawaan",

    neverUseNetworkLabel: "Jangan pernah pakai internet",
    neverUseNetworkHint:
      "Memblokir semua unduhan model; hanya memakai file yang sudah tersimpan atau diimpor manual.",

    removeBackgroundBtn: "Hapus latar belakang",
    saveBtn: "Simpan",
    saveAllBtn: "Simpan semua",
    cancelBtn: "Batal",
    openOutputFolderBtn: "Buka folder hasil",
    addMoreBtn: "Tambah foto",

    progressLoadingModel: "Memuat model AI ke memori...",
    progressProcessing: "Menghapus latar belakang...",
    progressDownloading: "Mengunduh model AI...",
    stateEmptyHint: "Tambahkan satu atau beberapa foto untuk mulai.",
    stateDoneSummary: "Semua foto selesai diproses.",
    statePartialSummary: "Selesai, tetapi ada foto yang gagal. Lihat daftar kendala di bawah.",
    stateCancelledSummary: "Proses dihentikan sebelum antrean selesai.",
    oomWarningBanner:
      "Foto berukuran sangat besar (16 MP ke atas). Sebaiknya batasi resolusi output ke 1024 px atau 2048 px bila RAM terbatas.",
    damagedWarningPrefix: "File gambar memiliki peringatan ringan",
    rejectedFilesBanner: "Sebagian item dilewati karena tidak didukung atau tidak terbaca",

    originalLabel: "Asli",
    cutoutLabel: "Hasil",
    compareToggleBtn: "Bandingkan dengan asli",
    compareToggleActiveBtn: "Lihat hasil saja",
    compareToggleHint: "Bandingkan dengan foto asli (C)",
    sliderAriaLabel: "Geser perbandingan foto asli dan hasil",
    zoomIn: "Perbesar",
    zoomOut: "Perkecil",
    zoomReset: "Reset tampilan",
    panHint: "Geser gambar untuk menggeser area saat diperbesar",
    thumbPlaceholder: "FOTO",
    statsTitle: "Ringkasan Proses",
    activeDeviceLabel: "Perangkat dipakai",
    dimensionsLabel: "Resolusi",
    inferenceTimeLabel: "Inferensi",
    totalTimeLabel: "Total",

    filterSupportedImages: "Gambar yang Didukung",
    filterOnnxModel: "Model ONNX",
    filterPngImage: "Gambar PNG",
    dialogBrowsePhotosTitle: "Pilih Foto",
    dialogBrowseFolderTitle: "Pilih Folder",
    dialogOutputFolderTitle: "Pilih Folder Simpan",
    dialogImportModelTitle: "Pilih File Model ONNX",
    dialogSavePngTitle: "Simpan Hasil PNG",

    firstRunTitle: "Penyiapan Model AI Pertama Kali",
    firstRunIntro:
      "RustMatte perlu mengunduh model AI satu kali. Setelah itu aplikasi berjalan sepenuhnya offline.",
    firstRunFileLabel: "Nama file",
    firstRunSizeLabel: "Ukuran",
    firstRunHostLabel: "Sumber host",
    firstRunFallbackLabel: "cadangan",
    firstRunLicenseLabel: "Lisensi bobot",
    firstRunDownloadBtn: "Unduh",
    firstRunImportBtn: "Pilih file yang sudah ada",
    firstRunRetryBtn: "Coba Unduh Lagi",
    firstRunCancelBtn: "Batal",
    firstRunBlockedOffline:
      "Mode 'Jangan pernah pakai internet' sedang aktif. Matikan di Pengaturan atau pilih file .onnx lokal.",
    modelReadyBadge: "Model siap offline",
    modelMissingBadge: "Perlu model",
    setupModelBtn: "Siapkan model",

    aboutTitle: "Tentang RustMatte & Lisensi",
    aboutVersionLabel: "Versi",
    aboutLicenseLabel: "Lisensi Aplikasi",
    aboutNetworkTitle: "Penggunaan Jaringan",
    aboutNetworkBody:
      "RustMatte memproses semua foto 100% secara lokal tanpa telemetri, analitik, atau unggahan ke server. Jaringan hanya digunakan jika Anda menyetujui unduhan model AI pertama kali, dan bisa dimatikan penuh lewat opsi 'Jangan pernah pakai internet'.",
    aboutModelsTitle: "Model AI & Catatan Dataset",
    aboutThirdPartyTitle: "Lisensi Pihak Ketiga",
    copyDiagnosticsBtn: "Salin info diagnostik",
    diagnosticsCopied: "Info diagnostik disalin ke clipboard (tanpa path atau nama file).",
    closeBtn: "Tutup",

    errorDetailsLabel: "Detail teknis",
    dismissBtn: "Tutup",
    savedSingleNotice: "PNG disimpan ke",
    savedAllNotice: "Hasil disimpan ke",
    bannerJobDone: "Selesai diproses ({processed}/{total} dalam {sec} dtk).",
    bannerJobPartial: "Selesai dengan kendala ({processed}/{total} berhasil dalam {sec} dtk).",
    bannerSavedSingle: "PNG disimpan ke {path}",
    bannerSavedAll: "{count} hasil disimpan ke {dir}",
    rejectedReasonInaccessible: "File atau folder tidak dapat diakses",
    rejectedReasonReadDir: "Gagal membaca isi folder",
    rejectedReasonNotFileOrDir: "Item bukan file atau folder biasa",
    rejectedReasonUnsupported: "Format gambar tidak didukung (gunakan JPG, JPEG, PNG, WebP, atau BMP)",
    warningDamagedInput: "File gambar tampak rusak atau tidak utuh",

    errOfflineTitle: "Akses internet dimatikan di Pengaturan.",
    errOfflineFix:
      "Matikan opsi 'Jangan pernah pakai internet' untuk mengunduh, atau klik 'Pilih file yang sudah ada' untuk memakai file .onnx lokal.",
    errNetworkTitle: "Gagal mengunduh model AI.",
    errNetworkFix:
      "Periksa koneksi internet atau firewall lalu klik Coba Unduh Lagi, atau impor file .onnx secara manual.",
    errChecksumTitle: "File model gagal lolos pemeriksaan integritas.",
    errChecksumFix:
      "File mungkin belum utuh atau bukan versi yang sesuai. Ulangi unduhan atau pilih file .onnx yang tepat.",
    errDiskTitle: "Gagal membaca atau menyimpan file ke disk.",
    errDiskFix:
      "Pastikan ruang penyimpanan mencukupi dan folder tujuan memiliki izin tulis.",
    errOomTitle: "Memori tidak cukup untuk memproses foto sebesar ini.",
    errOomFix:
      "Turunkan Batas Resolusi Output ke 1024 px atau 2048 px, tutup aplikasi lain, atau gunakan model Fast.",
    errUnsupportedTitle: "File ini tidak bisa dibaca sebagai gambar yang didukung.",
    errUnsupportedFix:
      "Gunakan foto berformat JPG, JPEG, PNG, WebP, atau BMP yang tidak rusak.",
    errOverwriteTitle: "Tidak bisa menimpa foto asli.",
    errOverwriteFix:
      "Pilih nama file atau folder lain agar foto asli Anda tetap aman.",
    errGenericTitle: "Terjadi kendala saat memproses foto.",
    errGenericFix:
      "Buka detail teknis di bawah atau coba ulang dengan Batas Resolusi Output lebih kecil.",

    shortcutsHint:
      "Pintasan: Ctrl+O buka, Ctrl+Enter proses, C bandingkan, Ctrl+S simpan, Atas/Bawah pindah foto, Esc batal",
  },
};
