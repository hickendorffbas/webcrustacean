pub mod css_lexer;
pub mod css_parser;


use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::dom::ElementDomNode;


#[cfg(test)] mod tests;
#[cfg(test)] mod test_lexer;
#[cfg(test)] mod test_parser;


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
pub fn resolve_full_styles_for_layout_node<'a>(dom_node: &'a Rc<RefCell<ElementDomNode>>, all_dom_nodes: &'a HashMap<usize, Rc<RefCell<ElementDomNode>>>,
                                               style_context: &StyleContext) -> HashMap<String, String> {

    //TODO: we are doing the cascade here by first doing the ua sheet, and then the author sheet. We need to make this more general in cascades
    //      because we need to support @layer, which adds an arbitrary amount of cascades

    let dom_node = dom_node.borrow();

    let mut rule_idx = 1;

    let mut active_style_rules = Vec::new();
    for style_rule in &style_context.user_agent_sheet {
        if style_rule_does_apply(&style_rule, &dom_node) {
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
        if style_rule_does_apply(&style_rule, &dom_node) {
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

    if dom_node.parent_id != 0 {
        let parent_node = all_dom_nodes.get(&dom_node.parent_id).expect(format!("id {} not present in all nodes", dom_node.parent_id).as_str());

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
        StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) },
                    property: "text-decoration".to_owned(), value: "underline".to_owned() },

    ];
}


pub fn get_property_from_computed_styles(styles: &HashMap<String, String>, property: &str) -> Option<String> {
    let computed_prop = styles.get(property);
    if computed_prop.is_some() {
        //TODO: not great that we clone here, but we seem to need to because we also could return new strings (for defaults), could we
        //      return references to static ones for those, and just always return a reference? We might need to return &str then
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


pub fn resolve_css_numeric_type_value(value: &String) -> f32 {
    //TODO: see https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/Values_and_units for many missing things here
    if value.chars().last() == Some('%') {
        //TODO: implement this case (we probably need to bring in more context)
        todo!("css percentages implemented");
    } else if value.len() > 3 && &value.as_str()[value.len() - 3..] == "rem" {
        //TODO: implement this case (we probably need to bring in more context)
        todo!("css rem unit not implemented");
    } else {
        let parsed_unwrapped = value.parse::<f32>();
        if parsed_unwrapped.is_err() {
            debug_log_warn(format!("could not parse css value: {:}", value));
            18.0  //this is a fairly random number, we should never really get here except by accident for unimplemented things
        } else {
            parsed_unwrapped.ok().unwrap()
        }
    }
}


pub fn has_style_value(styles: &HashMap<String, String>, style_name: &str, style_value: &String) -> bool {
    let item = get_property_from_computed_styles(styles, style_name);
    if item.is_none() {
        return false;
    }
    return item.unwrap() == *style_value;
}


pub fn get_color_style_value(styles: &HashMap<String, String>, property: &str) -> Option<Color> {
    let item = get_property_from_computed_styles(styles, property);
    if item.is_none() {
        return None; //this is not an error, it means the property was not set
    }
    let color = Color::from_string(item.as_ref().unwrap());

    if color.is_none() {
        //color is none, but item was something, so this means a color value is set, but we could not parse it. We fall back to black here
        //note this this is not the css default, because those might be different per property and are implemented elsewere
        debug_log_warn(format!("css value could not be parsed as a color: {:?}", item.unwrap()));
        return Some(Color::BLACK);
    }
    return color;
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


fn style_rule_does_apply(style_rule: &StyleRule, element_dom_node: &ElementDomNode) -> bool {
    if element_dom_node.name.is_none() {
        return false;
    }

    //TODO: currently this matches if any of the nodes matches, I'm not sure if this is correct, do they all need to match?
    return style_rule.selector.nodes.is_some() &&
           style_rule.selector.nodes.as_ref().unwrap().contains(&element_dom_node.name.as_ref().unwrap());
}
