use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::color::Color;
use crate::layout::LayoutNode;


#[cfg(test)]
mod tests;


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StyleRule {
    pub selector: Selector,
    pub style: Style,
}

#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Selector {
    //TODO: this should become more complex (we don't want the whole selector as text, but as actual parsed info, for now we just support nodes thought)
    pub nodes: Option<Vec<String>>,
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Style {
    pub property: String,
    pub value: String, //TODO: eventually we want different types here, probably also via an enum?
}


pub fn get_default_styles() -> Vec<Style> {
    //These are the styles that are applied to the outer most node, and are used when no styling is specified.
    //TODO: we should specify these as actual hardcoded CSS rules, with selectors
    return vec![
        Style { property: "font-size".to_owned(), value: "20".to_owned() },
        Style { property: "font-color".to_owned(), value: "black".to_owned() },
    ];
}


//TODO: we are now doing this when rendering. It might make more sense to do this earlier, cache the result on the node, and recompute only when needed
//TODO: we now do this on layout nodes, shouldn't we compute styles on DOM nodes? I think so
pub fn resolve_full_styles_for_layout_node<'a>(layout_node: &'a LayoutNode, all_nodes: &'a HashMap<usize, Rc<LayoutNode>>,
                                               style_rules: &Vec<StyleRule>) -> Vec<Style> {
    let mut resolved_style_names: HashSet<String> = HashSet::new();
    let mut node_to_check: &LayoutNode = layout_node;
    let is_root_node = node_to_check.parent_id == node_to_check.internal_id;

    let mut resolved_styles = apply_all_styles(node_to_check, style_rules);

    //TODO: not all properties should be inherited: https://developer.mozilla.org/en-US/docs/Web/CSS/Inheritance
    loop {

        let parent_id = node_to_check.parent_id;
        if parent_id == node_to_check.internal_id {
            //the top node has itself as parent
            break;
        }
        let parent_node = all_nodes.get(&parent_id).expect(format!("id {} not present in all nodes", parent_id).as_str());
        node_to_check = parent_node;

        //TODO: we also need to compute styles for our parents, so we should only do this if we did not do this yet...
        //      (store it somewhere and even persist across frames)
        let styles_of_parent = resolve_full_styles_for_layout_node(node_to_check, all_nodes, style_rules);

        for parent_style in styles_of_parent {
            if !resolved_style_names.contains(&parent_style.property) {
                resolved_style_names.insert(parent_style.property.clone());
                resolved_styles.push(parent_style);
            }
        }

    }

    if is_root_node {
        for default_style in get_default_styles() {
            if !resolved_style_names.contains(&default_style.property) {
                resolved_style_names.insert(default_style.property.clone());
                resolved_styles.push(default_style);
            }
        }
    }

    return resolved_styles;
}


fn apply_all_styles(node_to_check: &LayoutNode, styles: &Vec<StyleRule>) -> Vec<Style> {
    let mut applied_styles = Vec::new();

    if node_to_check.from_dom_node.is_some() {

        match node_to_check.from_dom_node.as_ref().unwrap().as_ref() {
            crate::dom::DomNode::Document(_) => { /* TODO: should this indeed be empty, or are there styles we need to apply here? */},
            crate::dom::DomNode::Attribute(_) => {},
            crate::dom::DomNode::Text(_) => {},
            crate::dom::DomNode::Element(element_node) => {

                for style in styles {
                    if style.selector.nodes.is_some() {
                        if style.selector.nodes.as_ref().unwrap().contains(&element_node.name.as_ref().unwrap()) {
                            applied_styles.push(style.style.clone());
                        }
                    }
                }

            }
        }

    }

    return applied_styles;
}


pub fn has_style_value(styles: &Vec<Style>, style_name: &str, style_value: &String) -> bool {
    let results = styles.iter().filter(|style| style.property == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    return results.contains(style_value);
}


pub fn get_numeric_style_value(styles: &Vec<Style>, style_name: &str) -> u16 {
    let results = styles.iter().filter(|style| style.property == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    return results.first().unwrap().parse::<u16>().unwrap(); //TODO: this should handle errors, return a Result?
}


pub fn get_color_style_value(styles: &Vec<Style>, style_name: &str) -> Option<Color> {
    let colors = styles.iter().filter(|style| style.property == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    if colors.len() == 0 {
        return None;
    }
    return Color::from_string(colors.first().unwrap());
}
