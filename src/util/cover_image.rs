use std::collections::HashMap;

use gtk::{gdk::TextureDownloader, glib::property::PropertyGet};

use crate::util::{color::RGBAColor, gradient_extractor::GradientColorExtractor};

thread_local! {
    static IMAGE_MEM_CACHE: std::cell::RefCell<HashMap<String, adw::gdk::Texture>> = std::cell::RefCell::new(HashMap::new());
}

// 1. UPDATED STRUCT: Fields wrapped in Option so dimensions can be omitted individually
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSize {
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl ImageSize {
    pub fn new(width: Option<i32>, height: Option<i32>) -> Self {
        Self { width, height }
    }

    pub fn from_dimesion(dimension: i32) -> Self {
        Self {
            width: Some(dimension),
            height: Some(dimension),
        }
    }
}

// 2. DEFAULT IMPLEMENTATION: Returns original sizing bounds when entirely unspecified
impl Default for ImageSize {
    fn default() -> Self {
        Self {
            height: None,
            width: None,
        }
    }
}

/// Fetches an image, caching the original file on disk and generating specific layout sizes on demand.
/// - `url`: The remote artwork link.
/// - `size`: The desired target layout boundaries. Pass `ImageSize::default()` for full quality.
pub async fn fetch_cached_image(url: &str, size: ImageSize) -> Option<adw::gdk::Texture> {
    if url.is_empty() {
        return None;
    }

    // 3. Create a unique RAM cache key that captures width and height configurations uniquely
    let size_suffix = match (size.width, size.height) {
        (Some(w), Some(h)) => format!("_{}x{}", w, h),
        (Some(w), None) => format!("_w{}", w),
        (None, Some(h)) => format!("_h{}", h),
        (None, None) => "_original".to_string(),
    };
    let cache_key = format!("{}{}", url, size_suffix);

    // 4. RAM Cache Check
    let ram_match = IMAGE_MEM_CACHE.with(|cache| cache.borrow().get(&cache_key).cloned());
    if let Some(texture) = ram_match {
        return Some(texture);
    }

    // 5. Disk Directory Setup
    let mut cache_path = adw::glib::user_cache_dir();
    cache_path.push("xpodcasts");
    cache_path.push("covers");
    let _ = std::fs::create_dir_all(&cache_path);

    // 6. Generate SHA-256 Hash based strictly on URL
    let glib_url_bytes = adw::glib::Bytes::from(url.as_bytes());
    let hashed_name =
        adw::glib::compute_checksum_for_bytes(adw::glib::ChecksumType::Sha256, &glib_url_bytes)
            .map(|g_string| g_string.to_string())
            .unwrap_or_else(|| "fallback".to_string());

    let disk_file_target = cache_path.join(hashed_name);

    // 7. Scenario A: File found on local disk
    if disk_file_target.exists() {
        if let Ok(bytes) = std::fs::read(&disk_file_target) {
            if let Some(texture) = decode_bytes_to_texture(&bytes, size) {
                IMAGE_MEM_CACHE.with(|cache| cache.borrow_mut().insert(cache_key, texture.clone()));
                return Some(texture);
            }
        }
    }

    // 8. Scenario B: Download full high-res target from web
    let client = reqwest::Client::new();
    if let Ok(response) = client.get(url).send().await {
        if let Ok(bytes) = response.bytes().await {
            let _ = std::fs::write(&disk_file_target, &bytes);

            if let Some(texture) = decode_bytes_to_texture(&bytes, size) {
                IMAGE_MEM_CACHE.with(|cache| cache.borrow_mut().insert(cache_key, texture.clone()));
                return Some(texture);
            }
        }
    }

    None
}

fn decode_bytes_to_texture(bytes: &[u8], size: ImageSize) -> Option<adw::gdk::Texture> {
    let glib_bytes = adw::glib::Bytes::from(bytes);

    // If both dimensions are None, skip the scaling stream step entirely
    if size.width.is_none() && size.height.is_none() {
        return adw::gdk::Texture::from_bytes(&glib_bytes).ok();
    }

    // GTK rule: Passing -1 for a dimension tells Pixbuf to preserve aspect ratio based on the other dimension
    let target_width = size.width.unwrap_or(-1);
    let target_height = size.height.unwrap_or(-1);

    let stream = gtk::gio::MemoryInputStream::from_bytes(&glib_bytes);
    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(
        &stream,
        target_width,
        target_height,
        true,
        gtk::gio::Cancellable::NONE,
    )
    .ok()?;

    Some(adw::gdk::Texture::for_pixbuf(&pixbuf))
}
