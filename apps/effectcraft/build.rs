//! Windows only: embed the TokEffect app icon and version information in the executable.
//!
//! The project's source of truth is assets/app-icon/tokeffect.png. We generate a standards-compliant
//! 256 px ICO in OUT_DIR so Windows builds do not depend on a separately committed, easily-stale
//! icon file. The image is PNG-compressed inside the ICO, as supported by modern Windows.

use std::path::{Path, PathBuf};
use std::{env, fs};

fn write_tokeffect_ico(png_path: &Path, ico_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use image::ImageEncoder;

    let source = image::open(png_path)?;
    let rgba = source.resize_exact(256, 256, image::imageops::FilterType::Lanczos3).to_rgba8();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png).write_image(
        rgba.as_raw(),
        256,
        256,
        image::ExtendedColorType::Rgba8,
    )?;
    let image_size = u32::try_from(png.len())?;

    // ICONDIR + one ICONDIRENTRY + a PNG-compressed 256x256 image.
    let mut ico = Vec::with_capacity(22 + png.len());
    ico.extend_from_slice(&0_u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1_u16.to_le_bytes()); // resource type: icon
    ico.extend_from_slice(&1_u16.to_le_bytes()); // image count
    ico.extend_from_slice(&[0, 0, 0, 0]); // 256x256, palette count, reserved
    ico.extend_from_slice(&1_u16.to_le_bytes()); // colour planes
    ico.extend_from_slice(&32_u16.to_le_bytes()); // bits per pixel
    ico.extend_from_slice(&image_size.to_le_bytes());
    ico.extend_from_slice(&22_u32.to_le_bytes()); // image data offset
    ico.extend_from_slice(&png);
    fs::write(ico_path, ico)?;
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=EFFECTCRAFT_REQUIRE_WINRES");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from) else {
        println!("cargo:error=CARGO_MANIFEST_DIR is not set");
        return;
    };
    let Some(out_dir) = env::var_os("OUT_DIR").map(PathBuf::from) else {
        println!("cargo:error=OUT_DIR is not set");
        return;
    };
    let png_path = manifest_dir.join("../../assets/app-icon/tokeffect.png");
    let ico_path = out_dir.join("tokeffect.ico");
    println!("cargo:rerun-if-changed={}", png_path.display());

    if let Err(error) = write_tokeffect_ico(&png_path, &ico_path) {
        println!("cargo:error=Could not generate TokEffect Windows icon from {}: {error}", png_path.display());
        return;
    }

    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let icon_path = ico_path.to_string_lossy().into_owned();
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(&icon_path)
        .set("ProductName", "TokEffect")
        .set("FileDescription", "TokEffect video editor and motion graphics")
        .set("OriginalFilename", "TokEffect.exe")
        .set("InternalName", "TokEffect")
        .set("CompanyName", "TokEffect")
        .set("ProductVersion", &version)
        .set("FileVersion", &version)
        .set("LegalCopyright", "Copyright (c) 2026 TokEffect contributors");

    if let Err(error) = resource.compile() {
        if env::var_os("EFFECTCRAFT_REQUIRE_WINRES").is_some() {
            panic!("embedding the TokEffect Windows icon and version information failed: {error}");
        }
        println!("cargo:warning=TokEffect executable built without Windows icon/version resources: {error}");
    }
}
