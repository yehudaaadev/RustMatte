//! Internal TIFF-IFD parser for extracting embedded JPEG previews from Camera RAW files.

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use image::metadata::Orientation;

use crate::error::CoreError;

const MAX_IFDS: usize = 64;
const MAX_IFD_DEPTH: usize = 8;
const MAX_IFD_ENTRIES: u16 = 512;
const MAX_SUBIFD_COUNT: u32 = 32;
const MAX_RAW_JPEG_BYTES: u64 = 128 * 1024 * 1024;

const TAG_STRIP_OFFSETS: u16 = 0x0111;
const TAG_ORIENTATION: u16 = 0x0112;
const TAG_STRIP_BYTE_COUNTS: u16 = 0x0117;
const TAG_SUB_IFDS: u16 = 0x014A;
const TAG_JPEG_INTERCHANGE_FORMAT: u16 = 0x0201;
const TAG_JPEG_INTERCHANGE_FORMAT_LENGTH: u16 = 0x0202;

const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_IFD: u16 = 13;

#[derive(Debug, Clone, Copy)]
enum Endian {
    Little,
    Big,
}

impl Endian {
    fn read_u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }

    fn read_u32(self, bytes: [u8; 4]) -> u32 {
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }
}

/// Extracted embedded JPEG preview and TIFF IFD0 orientation from a Camera RAW file.
pub(crate) struct RawPreview {
    pub(crate) jpeg_bytes: Vec<u8>,
    pub(crate) orientation: Orientation,
}

#[derive(Debug, Clone, Copy)]
struct JpegCandidate {
    offset: u64,
    length: u64,
}

struct TiffWalker<'a> {
    file: &'a mut File,
    file_len: u64,
    endian: Endian,
    visited: HashSet<u32>,
    ifd_count: usize,
    best_candidate: Option<JpegCandidate>,
    ifd0_orientation: Orientation,
}

impl<'a> TiffWalker<'a> {
    fn read_exact_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), ()> {
        let end = offset.checked_add(buf.len() as u64).ok_or(())?;
        if end > self.file_len {
            return Err(());
        }
        self.file.seek(SeekFrom::Start(offset)).map_err(|_| ())?;
        self.file.read_exact(buf).map_err(|_| ())
    }

    fn read_u16_at(&mut self, offset: u64) -> Result<u16, ()> {
        let mut buf = [0u8; 2];
        self.read_exact_at(offset, &mut buf)?;
        Ok(self.endian.read_u16(buf))
    }

    fn read_u32_at(&mut self, offset: u64) -> Result<u32, ()> {
        let mut buf = [0u8; 4];
        self.read_exact_at(offset, &mut buf)?;
        Ok(self.endian.read_u32(buf))
    }

    fn parse_scalar_u32(&mut self, field_type: u16, count: u32, raw_val: [u8; 4]) -> Option<u32> {
        if count == 0 {
            return None;
        }
        match field_type {
            TYPE_SHORT => {
                if count <= 2 {
                    Some(u32::from(self.endian.read_u16([raw_val[0], raw_val[1]])))
                } else {
                    let offset = u64::from(self.endian.read_u32(raw_val));
                    self.read_u16_at(offset).ok().map(u32::from)
                }
            }
            TYPE_LONG | TYPE_IFD => {
                if count == 1 {
                    Some(self.endian.read_u32(raw_val))
                } else {
                    let offset = u64::from(self.endian.read_u32(raw_val));
                    self.read_u32_at(offset).ok()
                }
            }
            _ => None,
        }
    }

    fn consider_candidate(&mut self, offset: u64, length: u64) {
        if offset == 0 || !(2..=MAX_RAW_JPEG_BYTES).contains(&length) {
            return;
        }
        let Some(end) = offset.checked_add(length) else {
            return;
        };
        if end > self.file_len {
            return;
        }
        if let Some(best) = self.best_candidate {
            if length <= best.length {
                return;
            }
        }
        let mut magic = [0u8; 2];
        if self.read_exact_at(offset, &mut magic).is_ok() && magic == [0xFF, 0xD8] {
            self.best_candidate = Some(JpegCandidate { offset, length });
        }
    }

    fn walk_ifd_chain(&mut self, start_offset: u32, depth: usize, is_root: bool) {
        if depth > MAX_IFD_DEPTH {
            return;
        }
        let mut current = start_offset;
        let mut first_in_chain = is_root;

        while current != 0 {
            if self.ifd_count >= MAX_IFDS || !self.visited.insert(current) {
                break;
            }
            self.ifd_count += 1;

            let ifd_off = u64::from(current);
            let Ok(entry_count) = self.read_u16_at(ifd_off) else {
                break;
            };
            if entry_count > MAX_IFD_ENTRIES {
                break;
            }

            let mut jpeg_offset: Option<u32> = None;
            let mut jpeg_length: Option<u32> = None;
            let mut strip_offset: Option<u32> = None;
            let mut strip_length: Option<u32> = None;
            let mut sub_ifds: Vec<u32> = Vec::new();

            for i in 0..entry_count {
                let entry_off = ifd_off + 2 + u64::from(i) * 12;
                let mut entry = [0u8; 12];
                if self.read_exact_at(entry_off, &mut entry).is_err() {
                    break;
                }
                let tag = self.endian.read_u16([entry[0], entry[1]]);
                let field_type = self.endian.read_u16([entry[2], entry[3]]);
                let count = self
                    .endian
                    .read_u32([entry[4], entry[5], entry[6], entry[7]]);
                let raw_val = [entry[8], entry[9], entry[10], entry[11]];

                match tag {
                    TAG_ORIENTATION if first_in_chain => {
                        if let Some(val) = self.parse_scalar_u32(field_type, count, raw_val) {
                            self.ifd0_orientation = exif_orientation_from_u32(val);
                        }
                    }
                    TAG_JPEG_INTERCHANGE_FORMAT => {
                        jpeg_offset = self.parse_scalar_u32(field_type, count, raw_val);
                    }
                    TAG_JPEG_INTERCHANGE_FORMAT_LENGTH => {
                        jpeg_length = self.parse_scalar_u32(field_type, count, raw_val);
                    }
                    TAG_STRIP_OFFSETS => {
                        if count == 1 {
                            strip_offset = self.parse_scalar_u32(field_type, count, raw_val);
                        }
                    }
                    TAG_STRIP_BYTE_COUNTS => {
                        if count == 1 {
                            strip_length = self.parse_scalar_u32(field_type, count, raw_val);
                        }
                    }
                    TAG_SUB_IFDS if matches!(field_type, TYPE_LONG | TYPE_IFD) => {
                        if count == 1 {
                            let sub_off = self.endian.read_u32(raw_val);
                            if sub_off != 0 {
                                sub_ifds.push(sub_off);
                            }
                        } else if count > 1 && count <= MAX_SUBIFD_COUNT {
                            let array_off = u64::from(self.endian.read_u32(raw_val));
                            for idx in 0..count {
                                if let Ok(sub_off) =
                                    self.read_u32_at(array_off + u64::from(idx) * 4)
                                {
                                    if sub_off != 0 {
                                        sub_ifds.push(sub_off);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            if let (Some(off), Some(len)) = (jpeg_offset, jpeg_length) {
                self.consider_candidate(u64::from(off), u64::from(len));
            }
            if let (Some(off), Some(len)) = (strip_offset, strip_length) {
                self.consider_candidate(u64::from(off), u64::from(len));
            }

            for sub_off in sub_ifds {
                self.walk_ifd_chain(sub_off, depth + 1, false);
            }

            first_in_chain = false;
            let next_ptr_off = ifd_off + 2 + u64::from(entry_count) * 12;
            current = self.read_u32_at(next_ptr_off).unwrap_or(0);
        }
    }
}

fn exif_orientation_from_u32(val: u32) -> Orientation {
    match val {
        2 => Orientation::FlipHorizontal,
        3 => Orientation::Rotate180,
        4 => Orientation::FlipVertical,
        5 => Orientation::Rotate90FlipH,
        6 => Orientation::Rotate90,
        7 => Orientation::Rotate270FlipH,
        8 => Orientation::Rotate270,
        _ => Orientation::NoTransforms,
    }
}

/// Extracts the largest embedded JPEG preview and IFD0 orientation from a TIFF-based RAW file.
pub(crate) fn extract_raw_jpeg_preview(
    path: &Path,
    file: &mut File,
    file_len: u64,
) -> Result<RawPreview, CoreError> {
    if file_len < 8 {
        return Err(CoreError::ImageDecode {
            path: path.to_path_buf(),
            reason: "RAW file is too small to contain a valid TIFF header".to_string(),
        });
    }

    file.seek(SeekFrom::Start(0))?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header)?;

    let endian = match [header[0], header[1]] {
        [0x49, 0x49] => Endian::Little,
        [0x4D, 0x4D] => Endian::Big,
        _ => {
            return Err(CoreError::ImageDecode {
                path: path.to_path_buf(),
                reason: "Invalid RAW/TIFF header byte order".to_string(),
            });
        }
    };

    let magic = endian.read_u16([header[2], header[3]]);
    if magic != 42 {
        return Err(CoreError::ImageDecode {
            path: path.to_path_buf(),
            reason: format!("Unsupported TIFF magic number {magic} in RAW file"),
        });
    }

    let ifd0_offset = endian.read_u32([header[4], header[5], header[6], header[7]]);
    if ifd0_offset == 0 {
        return Err(CoreError::ImageDecode {
            path: path.to_path_buf(),
            reason: "RAW file has no TIFF IFD0 directory".to_string(),
        });
    }

    let mut walker = TiffWalker {
        file,
        file_len,
        endian,
        visited: HashSet::new(),
        ifd_count: 0,
        best_candidate: None,
        ifd0_orientation: Orientation::NoTransforms,
    };

    walker.walk_ifd_chain(ifd0_offset, 0, true);

    let Some(candidate) = walker.best_candidate else {
        return Err(CoreError::ImageDecode {
            path: path.to_path_buf(),
            reason: "No embedded JPEG preview found in RAW file".to_string(),
        });
    };

    let len_usize = usize::try_from(candidate.length).map_err(|_| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: "Embedded JPEG preview size exceeds addressable memory".to_string(),
    })?;

    let mut jpeg_bytes = vec![0u8; len_usize];
    walker.file.seek(SeekFrom::Start(candidate.offset))?;
    walker.file.read_exact(&mut jpeg_bytes)?;

    Ok(RawPreview {
        jpeg_bytes,
        orientation: walker.ifd0_orientation,
    })
}
