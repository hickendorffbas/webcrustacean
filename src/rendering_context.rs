use crate::fonts::FontCache;


pub struct RenderingContext<'a> {  //TODO: I do not understand the lifetimes here yet
    pub font_cache: FontCache<'a, 'a>,
}
