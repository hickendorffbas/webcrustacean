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
    Flex,
    FlexDirection,
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
            "flex" => CssProperty::Flex,
            "flex-direction" => CssProperty::FlexDirection,
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
impl StyleRule {
    fn make_for_tag_name(tag_name: &str, property: CssProperty, value: &str) -> StyleRule {
        return StyleRule { selector: Selector { elements: vec![(CssCombinator::None, SelectorType::Name, tag_name.to_owned())], pseudoclasses: None },
                           property, value: value.to_owned() }
    }
}


#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum CssCombinator {
    Descendent,
    Child,
    GeneralSibling,
    NextSibling,
    None,
}


#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum SelectorType {
    Name,
    Id,
    Class,
}


#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Selector {
    pub elements: Vec<(CssCombinator, SelectorType, String)>, //Note: the elements are in reverse order, to make evaluating them more performant
    #[allow(unused)] pub pseudoclasses: Option<Vec<String>>,  //TODO: implement
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


pub fn resolve_full_styles_for_dom_node(dom_node: &Rc<RefCell<ElementDomNode>>, all_dom_nodes: &HashMap<usize, Rc<RefCell<ElementDomNode>>>,
                                        style_context: &StyleContext) -> HashMap<CssProperty, String> {

    //TODO: we are doing the cascade here by first doing the ua sheet, and then the author sheet. We need to make this more general in cascades
    //      because we need to support @layer, which adds an arbitrary amount of cascades

    let mut rule_idx = 1;

    let mut active_style_rules = Vec::new();
    for style_rule in &style_context.user_agent_sheet {
        if style_rule_applies(&style_rule, &dom_node, all_dom_nodes) {
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
        if style_rule_applies(&style_rule, &dom_node, all_dom_nodes) {
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

    if dom_node.borrow().parent_id != 0 {
        let parent_node = all_dom_nodes.get(&dom_node.borrow().parent_id).expect(format!("id {} not present in all nodes", dom_node.borrow().parent_id).as_str());
        let parent_styles = resolve_full_styles_for_dom_node(&parent_node, all_dom_nodes, style_context);

        for (parent_style_property, parent_style_value) in parent_styles {

            //Some styles should not be inherited:  //TODO: this list is not complete yet
            //TODO: maybe we should list the ones that _are_ inherited, or make a method on the CssProperty
            if parent_style_property == CssProperty::Display ||
               parent_style_property == CssProperty::BackgroundColor ||
               parent_style_property == CssProperty::Flex ||
               parent_style_property == CssProperty::FlexDirection ||
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
    let computed_styles = resolve_full_styles_for_dom_node(&dom_node, all_dom_nodes, style_context);

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

        StyleRule::make_for_tag_name("h1", CssProperty::FontSize, "32"),
        StyleRule::make_for_tag_name("h2", CssProperty::FontSize, "30"),
        StyleRule::make_for_tag_name("h3", CssProperty::FontSize, "28"),
        StyleRule::make_for_tag_name("h4", CssProperty::FontSize, "26"),
        StyleRule::make_for_tag_name("h5", CssProperty::FontSize, "24"),
        StyleRule::make_for_tag_name("h6", CssProperty::FontSize, "22"),

        StyleRule::make_for_tag_name("b", CssProperty::FontWeight, "bold"),
        StyleRule::make_for_tag_name("i", CssProperty::FontStyle, "italic"),

        StyleRule::make_for_tag_name("a", CssProperty::Color, "blue"),
        StyleRule::make_for_tag_name("a", CssProperty::TextDecoration, "underline"),
    ];

    for element in HTML_BLOCK_ELEMENTS {
        rules.push(StyleRule::make_for_tag_name(element, CssProperty::Display, "block"));
    }

    return rules;
}


pub fn get_property_from_computed_styles(styles: &HashMap<CssProperty, String>, property: CssProperty) -> &str {
    let computed_prop = styles.get(&property);
    if computed_prop.is_some() {
        return computed_prop.unwrap().as_str();
    }

    return default_css_value(property);
}


pub fn default_css_value(property: CssProperty) -> &'static str {
    return match property {
        CssProperty::BackgroundColor => "white", //TODO: the actual default is "transparent", but we don't support that yet
        CssProperty::Color => "black",
        CssProperty::Display => "inline",
        CssProperty::Flex => "0 1 auto",
        CssProperty::FlexDirection => "row",
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


fn style_rule_applies(style_rule: &StyleRule, element_dom_node: &Rc<RefCell<ElementDomNode>>,
                      all_dom_nodes: &HashMap<usize, Rc<RefCell<ElementDomNode>>>) -> bool {

    return check_selector_for_match(&style_rule.selector, 0, element_dom_node, all_dom_nodes);
}


fn check_selector_for_match(selector: &Selector, starting_idx: usize, node_being_checked: &Rc<RefCell<ElementDomNode>>,
                            all_dom_nodes: &HashMap<usize, Rc<RefCell<ElementDomNode>>>) -> bool {

    let mut node_being_checked = node_being_checked.clone();
    let mut current_idx = starting_idx;

    for (combinator, selector_type, selector_part) in selector.elements.iter().skip(starting_idx) {

        {
            let node_being_checked_borr = node_being_checked.borrow();

            match selector_type {
                SelectorType::Name => {
                    if node_being_checked_borr.name.is_none() || node_being_checked_borr.name.as_ref().unwrap() != selector_part {
                        return false;
                    }
                },
                SelectorType::Id => {
                    let idx_first_char = selector_part.char_indices().nth(1).map(|(i, _)| i).unwrap_or(selector_part.len());
                    let id_to_search = &selector_part[idx_first_char..];

                    let node_id = node_being_checked_borr.get_attribute_value("id");
                    if node_id.is_none() || node_id.unwrap().as_str() != id_to_search {
                        return false;
                    }
                },
                SelectorType::Class => {
                    let idx_first_char = selector_part.char_indices().nth(1).map(|(i, _)| i).unwrap_or(selector_part.len());
                    let class_to_search = &selector_part[idx_first_char..];

                    let all_classes = node_being_checked_borr.get_attribute_value("class");
                    let any_class_match = all_classes.is_some() && all_classes.unwrap().split_whitespace().any(|class| class == class_to_search);
                    if !any_class_match {
                        return false;
                    }
                },
            }
        }

        match combinator {
            CssCombinator::Descendent => {
                let mut node_descended_from = node_being_checked;

                loop {
                    let parent_id = node_descended_from.borrow().parent_id;
                    if parent_id == 0 {
                        return false;
                    }
                    node_descended_from = all_dom_nodes.get(&parent_id).unwrap().clone();

                    let match_from_node = check_selector_for_match(selector, current_idx+1, &node_descended_from, all_dom_nodes);
                    if match_from_node {
                        return true;
                    }
                }
            },
            CssCombinator::GeneralSibling => {
                //TODO: this will generate a set, apply the same kind of recursion as for descendent
                todo!(); //TODO: implement
            },
            CssCombinator::Child => {
                let parent_id = node_being_checked.borrow().parent_id;
                if parent_id == 0 {
                    return false;
                }
                let parent_node = all_dom_nodes.get(&parent_id).unwrap();
                node_being_checked = parent_node.clone();
            },
            CssCombinator::NextSibling => {
                let parent_id = node_being_checked.borrow().parent_id;
                if parent_id == 0 {
                    return false;
                }
                let parent_node = all_dom_nodes.get(&parent_id).unwrap();

                let mut found = false;
                let mut node_updated = false;
                for child in parent_node.borrow().children.as_ref().unwrap().iter().rev() {
                    if child.borrow().text.is_some() && child.borrow().text.as_ref().unwrap().text_content.trim().is_empty() {
                        //Its not strictly according to spec, but all the big browsers ignore whitespace text in between nodes for this combinator
                        continue;
                    }
                    if found {
                        node_being_checked = child.clone();
                        node_updated = true;
                        break;
                    }
                    if child.borrow().internal_id == node_being_checked.borrow().internal_id {
                        found = true;
                    }
                }
                if !found {
                    panic!("Node needs to be a child of its own parent");
                }
                if !node_updated {
                    return false; //No previous sibling was found
                }
            },
            CssCombinator::None => {
                //This is the case for the last element (first we encounter), so we stay on the current node
            },
        }
        current_idx += 1;
    }

    return true;
}