use cv::{
    feature::akaze::{Akaze, KeyPoint},
    image::{
        image::{self, DynamicImage, Rgba},
        imageproc::drawing,
    },
};

use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, CameraFormat, FrameFormat},
};

use minifb::{Key, Window, WindowOptions};
use std::time::Duration;

fn main() {

    // On macOS, request camera permission at runtime
    #[cfg(target_os = "macos")]
    nokhwa::nokhwa_initialize(|granted| assert!(granted, "camera permission denied"));

    // Selecting camera
    let index = CameraIndex::Index(0);

    // Selecting desired format
    let camera_format = CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30);
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format));

    // Creating instance of the camera
    let mut camera = Camera::new(index, requested).expect("camera");

    camera.open_stream().expect("open stream");

    // Prime one frame to size things
    let decoded = camera.frame().expect("frame").decode_image::<RgbFormat>().expect("decode");
    let (width, height) = (decoded.width(), decoded.height());

    let mut window = Window::new(
        "AKAZE Live (Esc quits)",
        width as usize,
        height as usize,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    ).expect("window");

    window.limit_update_rate(Some(Duration::from_micros(16_666))); // ~60 FPS

    // Reusable ARGB buffer for minifb: 0x00RRGGBB per pixel
    let mut argb_buf: Vec<u32> = vec![0; (width * height) as usize];

    while window.is_open() && !window.is_key_down(Key::Escape) {

        // Grab a frame
        let decoded = match camera.frame().and_then(|f| f.decode_image::<RgbFormat>()) {
            Ok(img) => img,
            Err(_) => continue, // skip a dropped frame
        };

        // decoded is RGB8 bytes in row-major order
        let rgb_bytes = decoded.as_raw();

        // Convert RGB -> 0x00RRGGBB
        for (i, chunk) in rgb_bytes.chunks_exact(3).enumerate() {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            argb_buf[i] = (r << 16) | (g << 8) | b; // 0x00RRGGBB
        }

        // Present to window
        if window.update_with_buffer(&argb_buf, width as usize, height as usize).is_err() {
            break;
        }
    }
}
