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
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum CssProperty {
    BackgroundColor,
    Color,
    Display,
    FontWeight,
    FontSize,
    FontStyle,
    TextDecoration,
}
impl CssProperty {
    pub fn from_string(value: &String) -> Option<CssProperty> {
        let prop = match value.as_str() {
            "background-color" => CssProperty::BackgroundColor,
            "color" => CssProperty::Color,
            "display" => CssProperty::Display,
            "font-weight" => CssProperty::FontWeight,
            "font-size" => CssProperty::FontSize,
            "font-style" => CssProperty::FontStyle,
            "text-decoration" => CssProperty::TextDecoration,
            _ => return None,
        };
        return Some(prop);
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StyleContext {
    pub user_agent_sheet: Vec<StyleRule>,
    pub author_sheet: Vec<StyleRule>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StyleRule {
    pub selector: Selector,
    pub property: CssProperty,
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
    property: CssProperty,
    property_value: &'a String,
    origin: Origin,
    specificity_attribute: u8,
    specificity_id: u8,
    specificity_class: u8,
    specificity_type: u8,
    definition_order: u32,
}


pub fn resolve_full_styles_for_layout_node(dom_node: &ElementDomNode, all_dom_nodes: &HashMap<usize, Rc<RefCell<ElementDomNode>>>,
                                           style_context: &StyleContext) -> HashMap<CssProperty, String> {

    //TODO: we are doing the cascade here by first doing the ua sheet, and then the author sheet. We need to make this more general in cascades
    //      because we need to support @layer, which adds an arbitrary amount of cascades

    let mut rule_idx = 1;

    let mut active_style_rules = Vec::new();
    for style_rule in &style_context.user_agent_sheet {
        if style_rule_does_apply(&style_rule, &dom_node) {
            active_style_rules.push(
                ActiveStyleRule {
                    property: style_rule.property,
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
                    property: style_rule.property,
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
        resolved_styles.insert(active_style_rule.property, (*active_style_rule.property_value).clone());
    }

    if dom_node.parent_id != 0 {
        let parent_node = all_dom_nodes.get(&dom_node.parent_id).expect(format!("id {} not present in all nodes", dom_node.parent_id).as_str());
        let parent_styles = resolve_full_styles_for_layout_node(&parent_node.borrow(), all_dom_nodes, style_context);

        for (parent_style_property, parent_style_value) in parent_styles {

            //Some styles should not be inherited:  //TODO: this list is not complete yet
            if parent_style_property == CssProperty::Display ||
               parent_style_property == CssProperty::BackgroundColor ||
               parent_style_property == CssProperty::TextDecoration {
                    continue;
            }

            if !resolved_styles.contains_key(&parent_style_property) {
                resolved_styles.insert(parent_style_property.clone(), parent_style_value.clone());
            }
        }
    }

    return resolved_styles;
}


pub fn compute_styles(dom_node: &Rc<RefCell<ElementDomNode>>, all_dom_nodes: &HashMap<usize, Rc<RefCell<ElementDomNode>>>, style_context: &StyleContext) {
    let computed_styles = resolve_full_styles_for_layout_node(&dom_node.borrow(), all_dom_nodes, style_context);
    dom_node.borrow_mut().styles = computed_styles;

    if dom_node.borrow().children.is_some() {
        for child in dom_node.borrow().children.as_ref().unwrap() {
            compute_styles(child, all_dom_nodes, style_context);
        }
    }
}


const HTML_BLOCK_ELEMENTS: [&'static str;11] = ["div", "form", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "p", "table"];


pub fn get_user_agent_style_sheet() -> Vec<StyleRule> {
    let mut rules = vec![
        //TODO: convert to an actual stylesheet (CSS string) we load in (or maybe not, but a better other format?)

        StyleRule { selector: Selector { nodes: Some(vec!["h1".to_owned()]) }, property: CssProperty::FontSize, value: "32".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h2".to_owned()]) }, property: CssProperty::FontSize, value: "30".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h3".to_owned()]) }, property: CssProperty::FontSize, value: "28".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h4".to_owned()]) }, property: CssProperty::FontSize, value: "26".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h5".to_owned()]) }, property: CssProperty::FontSize, value: "24".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["h6".to_owned()]) }, property: CssProperty::FontSize, value: "22".to_owned() },

        StyleRule { selector: Selector { nodes: Some(vec!["b".to_owned()]) }, property: CssProperty::FontWeight, value: "bold".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["i".to_owned()]) }, property: CssProperty::FontStyle, value: "italic".to_owned() },

        StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) }, property: CssProperty::Color, value: "blue".to_owned() },
        StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) }, property: CssProperty::TextDecoration, value: "underline".to_owned() },
    ];

    for element in HTML_BLOCK_ELEMENTS {
        rules.push(StyleRule { selector:Selector { nodes: Some(vec![element.to_owned()]) }, property: CssProperty::Display, value: "block".to_owned() });
    }

    return rules;
}


pub fn get_property_from_computed_styles(styles: &HashMap<CssProperty, String>, property: CssProperty) -> &str {
    let computed_prop = styles.get(&property);
    if computed_prop.is_some() {
        return computed_prop.unwrap().as_str();
    }

    //Defaults per css property:
    return match property {
        CssProperty::BackgroundColor => "white", //TODO: the actual default is "transparent", but we don't support that yet
        CssProperty::Color => "black",
        CssProperty::Display => "inline",
        CssProperty::FontSize => "18",
        CssProperty::FontStyle => "normal",
        CssProperty::FontWeight => "normal",
        CssProperty::TextDecoration => "none", //TODO: this is in reality a more complicated structure
    };
}


pub fn resolve_css_numeric_type_value(value: &str) -> f32 {
    //TODO: see https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/Values_and_units for many missing things here
    if value.chars().last() == Some('%') {
        //TODO: implement this case (we probably need to bring in more context)
        todo!("css percentages implemented");
    } else if value.len() > 3 && &value[value.len() - 3..] == "rem" {
        //TODO: implement this case (we probably need to bring in more context)
        todo!("css rem unit not implemented");
    } else if value.len() > 5 && &value[0..4] == "var(" && value.chars().last() == Some(')') {
        //TODO: These kind of things should be parsed in the parser already, into some structure that we can just evaluate here (the eval should happen here)
        //      for now we are just ignoring this case and returning a temp value, because it happens a lot
        18.0
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


pub fn has_style_value(styles: &HashMap<CssProperty, String>, property: CssProperty, style_value: &String) -> bool {
    return get_property_from_computed_styles(styles, property) == *style_value;
}


pub fn get_color_style_value(styles: &HashMap<CssProperty, String>, property: CssProperty) -> Color {
    let item = get_property_from_computed_styles(styles, property);

    if item.len() > 5 && &item[0..4] == "var(" && item.chars().last() == Some(')') {
        //TODO: These kind of things should be parsed in the parser already, into some structure that we can just evaluate here (the eval should happen here)
        //      for now we are just ignoring this case because it happens a lot
        return Color::BLACK;
    }

    let color = Color::from_string(item);

    if color.is_none() {
        //color is none, but item was something, so this means a color value is set, but we could not parse it. We fall back to black here
        //note this this is not the css default, because those might be different per property and are implemented elsewere
        debug_log_warn(format!("css value could not be parsed as a color: {:?}", item));
        return Color::BLACK;
    }
    return color.unwrap();
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
