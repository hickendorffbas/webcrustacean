use rusttype::{point, Font, Scale};

use crate::{color::Color, platform::Platform};

//TODO: the paths below will be shorter once we include the ttf's in the repository (but we still need to check what font's we need)
static FONT_DATA: [u8; include_bytes!("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf").len()] =
    *include_bytes!("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf");


pub fn render_text(platform: &mut Platform, text: &str, size: u16, color: Color, x: f32, y: f32) {

    //TODO: how expensive it is to do this every time? Store it in a context maybe?
    let font: Font<'static> = Font::try_from_bytes(&FONT_DATA).expect("Failure loading font data");

    let scale = Scale::uniform(size as f32);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, v_metrics.ascent)).collect();

    platform.enable_blending(); //TODO: what if we always have blending on? Maybe more expensive?

    //TODO: to speed this up, we would need to save the resulting bitmap including alpha somewhere
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|g_x, g_y, g_v| {

                let absolute_x = g_x as i32 + bounding_box.min.x + x as i32;
                let absolute_y = g_y as i32 + bounding_box.min.y + y as i32;
                platform.set_pixel(absolute_x, absolute_y, color, (g_v * 255.0) as u8);

            });
        }
    }

    platform.disable_blending();

}


