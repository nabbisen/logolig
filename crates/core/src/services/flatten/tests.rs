use super::*;

fn make_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Rgba8 {
    Rgba8::try_from_raw(width, height, Arc::<[u8]>::from(pixels)).expect("valid Rgba8 fixture")
}

#[test]
fn fully_opaque_pixels_are_unchanged() {
    // All pixels fully opaque: RGB must be unchanged.
    let src = make_rgba(2, 1, vec![100, 150, 200, 255, 50, 75, 25, 255]);
    let out = flatten_to_white(&src);
    assert_eq!(out.pixels.as_ref(), &[100, 150, 200, 255, 50, 75, 25, 255]);
}

#[test]
fn fully_transparent_pixels_become_white() {
    // Fully transparent pixels become white regardless of their RGB.
    let src = make_rgba(1, 1, vec![100, 150, 200, 0]);
    let out = flatten_to_white(&src);
    assert_eq!(out.pixels.as_ref(), &[255, 255, 255, 255]);
}

#[test]
fn half_alpha_blends_halfway_to_white() {
    // alpha=128 (~50%) with R=0 → R' ≈ 127 (mid-grey), alpha becomes 255.
    let src = make_rgba(1, 1, vec![0, 0, 0, 128]);
    let out = flatten_to_white(&src);
    let bytes = out.pixels.as_ref();
    // Linear blend: round(0 * 128/255 + 255 * (1 - 128/255)) = round(127.0) = 127
    // Allow ±1 for floating-point rounding.
    assert!(
        (126..=128).contains(&bytes[0]),
        "expected ~127, got {}",
        bytes[0]
    );
    assert_eq!(bytes[3], 255, "alpha must be saturated");
}

#[test]
fn output_alpha_is_always_saturated() {
    // Regardless of input alpha, every output pixel must be alpha=255.
    let src = make_rgba(
        4,
        1,
        vec![
            0, 0, 0, 0, //
            10, 20, 30, 64, //
            40, 80, 120, 192, //
            200, 100, 50, 255,
        ],
    );
    let out = flatten_to_white(&src);
    let bytes = out.pixels.as_ref();
    assert_eq!(bytes[3], 255);
    assert_eq!(bytes[7], 255);
    assert_eq!(bytes[11], 255);
    assert_eq!(bytes[15], 255);
}

#[test]
fn dimensions_are_preserved() {
    // Dimensions must be preserved (trivial but guarded against regression).
    let src = make_rgba(7, 3, vec![0; 7 * 3 * 4]);
    let out = flatten_to_white(&src);
    assert_eq!(out.width, 7);
    assert_eq!(out.height, 3);
    assert_eq!(out.pixels.len(), 7 * 3 * 4);
}
