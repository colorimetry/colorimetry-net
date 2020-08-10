use palette::Pixel;

pub fn transform_colors(data: &mut [u8]) {
    // Technically, it is probably wrong to load as linear, as
    // the images are likely in sRGB colorspace. However, this
    // gives a better match to the results (visually inspected)
    // of operations with "Color Inspector 3D" by Kai Uwe
    // Barthel. Furthermore, apparently [it is not specified
    // what colorspace browsers use to draw images in the canvas
    // element](https://wiki.whatwg.org/wiki/CanvasColorSpace).

    let color_buffer: &mut [palette::rgb::Rgba<
        palette::encoding::Linear<_>,
        u8,
    >] = Pixel::from_raw_slice_mut(data);

    for pix in color_buffer.iter_mut() {
        // See
        // https://github.com/erisir/FIJI/blob/a30ce62566b7a441bc315c8fff365b9985779b27/src-plugins/Color_Inspector_3D/src/main/java/Color_Inspector_3D.java#L4391-L4472

        let rgb: palette::rgb::Rgb<_, u8> = pix.color;
        let rgb_f32: palette::rgb::Rgb<_, f32> = rgb.into_format();

        use palette::ConvertInto;
        let mut hsl_f32: palette::Hsl<palette::encoding::Srgb, f32> =
            rgb_f32.convert_into();

        hsl_f32.hue =
            palette::RgbHue::from_degrees(hsl_f32.hue.to_degrees() + 180.0);
        // hsl_f32.clamp_self();

        hsl_f32.saturation *= 4.0;
        // hsl_f32.clamp_self();

        let rgb_f32: palette::rgb::Rgb<_, f32> = hsl_f32.convert_into();
        let rgb_u8: palette::rgb::Rgb<_, u8> = rgb_f32.into_format();
        pix.color = rgb_u8;
    }
}
