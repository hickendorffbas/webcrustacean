use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::color::Color;
use crate::dom::DomNode;


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
    //TODO: this should become more complex (we don't want the whole selector as text, but as actual parsed info, for now we just support nodes though)
    pub wildcard: bool,
    pub nodes: Option<Vec<String>>,
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Style {
    pub property: String,
    pub value: String, //TODO: eventually we want different types here, probably also via an enum?
}


pub fn get_default_styles() -> Vec<StyleRule> {
    //These are the styles that are applied to the outer most node, and are used when no styling is specified.
    return vec![
        StyleRule { selector: Selector { nodes: None, wildcard: true },
                    style: Style { property: "font-size".to_owned(), value: "20".to_owned() }},
        StyleRule { selector: Selector { nodes: None, wildcard: true },
                    style: Style { property: "font-color".to_owned(), value: "black".to_owned() }},
    ];
}


//TODO: we are now doing this when rendering. It might make more sense to do this earlier, cache the result on the node, and recompute only when needed
pub fn resolve_full_styles_for_layout_node<'a>(dom_node: &'a Rc<DomNode>, all_dom_nodes: &'a HashMap<usize, Rc<DomNode>>,
                                               style_rules: &Vec<StyleRule>) -> Vec<Style> {
    let mut resolved_style_properties: HashSet<String> = HashSet::new();
    let mut node_to_check = dom_node;

    let mut resolved_styles = apply_all_styles(node_to_check, style_rules, &mut resolved_style_properties);
    for style in &resolved_styles {
        resolved_style_properties.insert(style.property.clone());
    }

    //TODO: not all properties should be inherited: https://developer.mozilla.org/en-US/docs/Web/CSS/Inheritance
    loop {

        let parent_id = node_to_check.get_parent_id();
        if parent_id.is_none() {
            break;
        }
        let parent_node = all_dom_nodes.get(&parent_id.unwrap()).expect(format!("id {} not present in all nodes", parent_id.unwrap()).as_str());
        node_to_check = parent_node;

        //TODO: we also need to compute styles for our parents, so we should only do this if we did not do this yet...
        //      (store it somewhere and even persist across frames?)
        let styles_of_parent = resolve_full_styles_for_layout_node(node_to_check, all_dom_nodes, style_rules);

        for parent_style in styles_of_parent {
            if !resolved_style_properties.contains(&parent_style.property) {
                resolved_style_properties.insert(parent_style.property.clone());
                resolved_styles.push(parent_style);
            }
        }
    }

    return resolved_styles;
}


//TODO: Vec<Style> should be a hashmap, but keep the Style object in there, with the property? For easy filling from a StyleRule
fn apply_all_styles(node_to_check: &DomNode, style_rules: &Vec<StyleRule>, resolved_style_properties: &mut HashSet<String>) -> Vec<Style> {
    let mut applied_styles = Vec::new();

    //TODO: the rules are not checked by prio currently (specificity?, I think for example wildcard should have less prio)

    //TODO: we don't check for overlapping styles (i.e. having the same property) yet

    for style_rule in style_rules {
        if does_style_rule_apply(&style_rule, &node_to_check) {
            if !resolved_style_properties.contains(&style_rule.style.property) { //TODO: this is not great, because it will just take the first matching one
                applied_styles.push(style_rule.style.clone());
                resolved_style_properties.insert(style_rule.style.property.clone());
            }
        }
    }

    for style_rule in get_default_styles() {
        if does_style_rule_apply(&style_rule, &node_to_check) {
            if !resolved_style_properties.contains(&style_rule.style.property) { //TODO: this is not great, because it will just take the first matching one
                applied_styles.push(style_rule.style.clone());
                resolved_style_properties.insert(style_rule.style.property.clone());
            }
        }
    }

    return applied_styles;
}


fn does_style_rule_apply(style_rule: &StyleRule, dom_node: &DomNode) -> bool {
    if style_rule.selector.wildcard {
        return true;
    }

    match dom_node {
        DomNode::Document(_) => {
            return false;
        },
        DomNode::Element(element_node) => {
            if style_rule.selector.nodes.is_some() && style_rule.selector.nodes.as_ref().unwrap().contains(&element_node.name.as_ref().unwrap()) {
                return true;
            }
            return false;
        },
        DomNode::Attribute(_) => {
            return false;
        },
        DomNode::Text(_) => {
            return false;
        },
    }
}


pub fn has_style_value(styles: &Vec<Style>, style_name: &str, style_value: &String) -> bool {
    let results = styles.iter().filter(|style| style.property == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    return results.contains(style_value);
}


pub fn get_numeric_style_value(styles: &Vec<Style>, style_name: &str) -> Option<u16> {
    let results = styles.iter().filter(|style| style.property == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    if results.first().is_none() {
        return None;
    }
    return results.first().unwrap().parse::<u16>().ok();
}


pub fn get_color_style_value(styles: &Vec<Style>, style_name: &str) -> Option<Color> {
    let colors = styles.iter().filter(|style| style.property == style_name).map(|style| style.value.clone()).collect::<Vec<String>>();
    if colors.len() == 0 {
        return None;
    }
    return Color::from_string(colors.first().unwrap());
}
