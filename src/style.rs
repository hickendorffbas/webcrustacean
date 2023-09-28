use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::color::Color;
use crate::dom::DomNode;


#[cfg(test)]
mod tests;


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StyleContext {
    pub user_agent_sheet: Vec<StyleRule>,
    pub author_sheet: Vec<StyleRule>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StyleRule {
    pub selector: Selector,
    pub property: String,
    pub value: String,
}


#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Selector {
    //TODO: this should become more complex (we don't want the whole selector as text, but as actual parsed info, for now we just support nodes though)
    pub nodes: Option<Vec<String>>,
}


#[derive(PartialEq)]
enum Origin {
    Author,
    UserAgent,
    //we don't implement the USER origin
}


struct ActiveStyleRule<'a> {
    property: &'a String,
    property_value: &'a String,
    origin: Origin,
    specificity_attribute: u8,
    specificity_id: u8,
    specificity_class: u8,
    specificity_type: u8,
    definition_order: u32,
}


//TODO: we are now doing this when rendering. It might make more sense to do this earlier, cache the result on the node, and recompute only when needed
pub fn resolve_full_styles_for_layout_node<'a>(dom_node: &'a Rc<DomNode>, all_dom_nodes: &'a HashMap<usize, Rc<DomNode>>,
                                               style_context: &StyleContext) -> HashMap<String, String> {

    //TODO: we are doing the cascade here by first doing the ua sheet, and then the author sheet. We need to make this more general in cascades
    //      because we need to support @layer, which adds an arbitrary amount of cascades

    let mut rule_idx = 1;

    let mut active_style_rules = Vec::new();
    for style_rule in &style_context.user_agent_sheet {
        if does_style_rule_apply(&style_rule, dom_node) {
            active_style_rules.push(
                ActiveStyleRule {
                    property: &style_rule.property,
                    property_value: &style_rule.value,
                    origin: Origin::UserAgent,
                    specificity_attribute: 0,  //TODO: implement
                    specificity_id: 0,  //TODO: implement
                    specificity_class: 0,  //TODO: implement
                    specificity_type: 0,  //TODO: implement
                    definition_order: rule_idx,
                }
            );
        }
        rule_idx += 1;
    }

    for style_rule in &style_context.author_sheet {
        if does_style_rule_apply(&style_rule, dom_node) {
            active_style_rules.push(
                ActiveStyleRule {
                    property: &style_rule.property,
                    property_value: &style_rule.value,
                    origin: Origin::Author,
                    specificity_attribute: 0,  //TODO: implement
                    specificity_id: 0,  //TODO: implement
                    specificity_class: 0,  //TODO: implement
                    specificity_type: 0,  //TODO: implement
                    definition_order: rule_idx,
                }
            );
        }
        rule_idx += 1;
    }

    active_style_rules.sort_by(|rule_a, rule_b| compare_style_rules(rule_a, rule_b));

    let mut resolved_styles = HashMap::new();
    for active_style_rule in active_style_rules {
        resolved_styles.insert((*active_style_rule.property).clone(), (*active_style_rule.property_value).clone());
    }

    let parent_id = dom_node.get_parent_id();
    if parent_id.is_some() {
        let parent_node = all_dom_nodes.get(&parent_id.unwrap()).expect(format!("id {} not present in all nodes", parent_id.unwrap()).as_str());

        //TODO: not all properties should be inherited: https://developer.mozilla.org/en-US/docs/Web/CSS/Inheritance

        let parent_styles = resolve_full_styles_for_layout_node(parent_node, all_dom_nodes, style_context);

        for (parent_style_property, parent_style_value) in parent_styles {
            if !resolved_styles.contains_key(&parent_style_property) {
                resolved_styles.insert(parent_style_property.clone(), parent_style_value.clone());
            }
        }
    }

    return resolved_styles;
}


pub fn get_user_agent_style_sheet() -> Vec<StyleRule> {
    //These are the styles that are applied to the outer most node, and are used when no styling is specified.
    return vec![
        //TODO: convert to an actual stylesheet (CSS string) we load in (or maybe not, but a better other format?)

        StyleRule { selector: Selector { nodes: Some(vec!["h1".to_owned()]) },
                    property: "font-size".to_owned(), value: "32".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h2".to_owned()]) },
                    property: "font-size".to_owned(), value: "30".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h3".to_owned()]) },
                    property: "font-size".to_owned(), value: "28".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h4".to_owned()]) },
                    property: "font-size".to_owned(), value: "26".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h5".to_owned()]) },
                    property: "font-size".to_owned(), value: "24".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h6".to_owned()]) },
                    property: "font-size".to_owned(), value: "22".to_owned() },

        StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) },
                    property: "color".to_owned(), value: "blue".to_owned() },
    ];
}


// This function returns what rule_a is compare to rule_b (less, equal or greater), greater meaning having higher priority
fn compare_style_rules(rule_a: &ActiveStyleRule, rule_b: &ActiveStyleRule) -> Ordering {

    //TODO: this check needs to be different in the future, because we need to support an arbitrary set of cascades
    if rule_a.origin != rule_b.origin {
        if rule_a.origin == Origin::UserAgent {
            return Ordering::Less;
        }
        return Ordering::Greater;
    }

    if rule_a.specificity_attribute > rule_a.specificity_attribute { return Ordering::Greater; }
    if rule_a.specificity_attribute < rule_a.specificity_attribute { return Ordering::Less; }

    if rule_a.specificity_id > rule_a.specificity_id { return Ordering::Greater; }
    if rule_a.specificity_id < rule_a.specificity_id { return Ordering::Less; }

    if rule_a.specificity_class > rule_a.specificity_class { return Ordering::Greater; }
    if rule_a.specificity_class < rule_a.specificity_class { return Ordering::Less; }

    if rule_a.specificity_type > rule_a.specificity_type { return Ordering::Greater; }
    if rule_a.specificity_type < rule_a.specificity_type { return Ordering::Less; }

    if rule_a.definition_order > rule_a.definition_order { return Ordering::Greater; }
    if rule_a.definition_order < rule_a.definition_order { return Ordering::Less; }

    return Ordering::Equal;
}


pub fn get_property_from_computed_styles(property: &str, styles: &HashMap<String, String>) -> Option<String> {
    let computed_prop = styles.get(property);
    if computed_prop.is_some() {
        //TODO: not great that we clone here, but we seem to need to because we also could return new strings (for defaults), could we
        //      return references to static ones for those, and just always return a reference?
        return Some(computed_prop.unwrap().clone());
    }

    //Defaults per css property:
    match property {
        "color" => return Some(String::from("black")),
        "font-size" => return Some(String::from("18")),
        "font-weight" => return Some(String::from("normal")),
        _ => { return None }
    };

}


fn does_style_rule_apply(style_rule: &StyleRule, dom_node: &DomNode) -> bool {
    match dom_node {
        DomNode::Element(element_node) => {
            //TODO: currently this matches if any of the nodes matches, I'm not sure if this is correct, do they all need to match?
            if style_rule.selector.nodes.is_some() && style_rule.selector.nodes.as_ref().unwrap().contains(&element_node.name.as_ref().unwrap()) {
                return true;
            }
            return false;
        },
        _ => { return false; },
    }
}


pub fn has_style_value(styles: &HashMap<String, String>, style_name: &str, style_value: &String) -> bool {
    let item = get_property_from_computed_styles(style_name, styles);
    if item.is_none() {
        return false;
    }
    return item.unwrap() == *style_value;
}


pub fn get_numeric_style_value(styles: &HashMap<String, String>, style_name: &str) -> Option<u16> {
    let item = get_property_from_computed_styles(style_name, styles);
    if item.is_none() {
        return None;
    }
    return item.unwrap().parse::<u16>().ok();
}


pub fn get_color_style_value(styles: &HashMap<String, String>, style_name: &str) -> Option<Color> {
    let item = get_property_from_computed_styles(style_name, styles);
    if item.is_none() {
        return None;
    }
    return Color::from_string(&item.unwrap());
}
