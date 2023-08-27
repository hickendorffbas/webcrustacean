use crate::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH,
};
use crate::color::Color;
use crate::fonts::Font;
use crate::platform::{Platform, Position};


pub const HEADER_HEIGHT: u32 = 50; //The height of the header of bbrowser, so below this point the actual page is rendered
pub const SIDE_SCROLLBAR_WIDTH: u32 = 20;
const UI_BASIC_COLOR: Color = Color::new(212, 208, 200); 


pub fn render_ui(platform: &mut Platform) {
    render_header(platform);
    render_scrollbar(platform);
}


fn render_header(platform: &mut Platform) {
    let font = Font::new(true, 14);
    platform.render_text(&"Bbrowser".to_owned(), 10, 10, &font, Color::BLACK);
    platform.draw_line(Position { x: 0, y: HEADER_HEIGHT - 5 },
                       Position { x: SCREEN_WIDTH, y: HEADER_HEIGHT - 5 },
                       Color::BLACK);
}


fn render_scrollbar(platform: &mut Platform) {
    let scrollbar_start_x = SCREEN_WIDTH - SIDE_SCROLLBAR_WIDTH;

    platform.fill_rect(scrollbar_start_x, HEADER_HEIGHT, SCREEN_WIDTH - scrollbar_start_x, SCREEN_HEIGHT - HEADER_HEIGHT, UI_BASIC_COLOR);


    //TODO: get data on how much we are seeing, relative, of the complete page
    //      and how far we are currently scrolled, and draw darker thing in the bar according to that

}
