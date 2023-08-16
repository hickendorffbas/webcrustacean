use std::{collections::HashMap, rc::Rc};

use crate::layout::LayoutNode;
use crate::renderer::Color;  //TODO: color does not belong in the renderer


#[cfg(test)]
mod tests;


#[derive(PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Style {
    pub name: String,
    pub value: String, //TODO: eventually we want different types here, probably also via an enum?
}


pub fn get_default_styles() -> Vec<Style> {
    //These are the styles that are applied to the outer most node, and are used when no styling is specified.
    return vec![
        Style { name: "font-size".to_owned(), value: "20".to_owned() },
        Style { name: "font-color".to_owned(), value: "black".to_owned() },
    ];
}


//TODO: we are now doing this when rendering. It might make more sense to do this earlier, cache the result on the node, and recompute only when needed
//TODO: we now do this on layout nodes, shouldn't we compute styles on DOM nodes? I think so
pub fn resolve_full_styles_for_layout_node<'a>(layout_node: &'a LayoutNode, all_nodes: &'a HashMap<usize, Rc<LayoutNode>>) -> Vec<&'a Style> {
    let mut resolved_styles: Vec<&Style> = Vec::new();
    let mut resolved_style_names: Vec<String> = Vec::new();

    let mut node_to_check: &LayoutNode = layout_node;

    loop {
        for local_style in &node_to_check.styles {
            if !resolved_style_names.contains(&&local_style.name) {
                resolved_styles.push(&local_style);
                resolved_style_names.push(local_style.name.clone());
            }
        }

        let parent_id = node_to_check.parent_id;
        if parent_id == node_to_check.internal_id {
            //the top node has itself as parent
            break;
        }
        let parent_node = all_nodes.get(&parent_id).expect(format!("id {} not present in all nodes", parent_id).as_str());
        node_to_check = parent_node;
    }


    return resolved_styles;
}

pub fn has_style_value(styles: &Vec<&Style>, style_name: &str, style_value: &String) -> bool {
    let results = styles.iter().filter(|style| style.name == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    return results.contains(style_value);
}

pub fn get_numeric_style_value(styles: &Vec<&Style>, style_name: &str) -> u16 {
    let results = styles.iter().filter(|style| style.name == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    return results.first().unwrap().parse::<u16>().unwrap(); //TODO: this should handle errors, return a Result?

}

pub fn get_color_style_value(styles: &Vec<&Style>, style_name: &str) -> Option<Color> {
    let colors = styles.iter().filter(|style| style.name == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    return Color::from_string(colors.first().unwrap()); //TODO: we need to handle the case where the style_name does not exist, 
                                                        //      and where the color does not exist
}


