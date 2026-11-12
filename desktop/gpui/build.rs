// The Mac's own video and PDF views (native/media_views.m): AVKit and PDFKit
// are Objective-C, so their small shim is compiled here rather than written
// as objc2 calls, as the web runtime's AppKit glue is (../web/build.rs).
fn main() {
    println!("cargo:rerun-if-changed=native/media_views.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        // The older asset-loading calls work on every macOS this runs on;
        // their deprecation warnings aren't worth an availability split.
        cc::Build::new().file("native/media_views.m").flag("-fobjc-arc").flag("-Wno-deprecated-declarations").compile("threadbear_media_views");
        for framework in ["AppKit", "Foundation", "AVFoundation", "AVKit", "PDFKit"] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
}
