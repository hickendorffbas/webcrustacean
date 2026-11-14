pub mod css_lexer;
pub mod css_parser;
pub mod css_properties;

#[cfg(test)] mod tests;
#[cfg(test)] mod test_parser;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::color::Color;
use crate::dom::ElementDomNode;
use crate::style::css_properties::{
    CssProperty,
    CssPropertyValue,
    Display,
    FontSize,
    FontStyle,
    FontWeight,
    Styles,
    TextDecoration,
};


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StyleContext {
    pub user_agent_sheet: Vec<StyleRule>,
    pub author_sheet: Vec<StyleRule>,
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum AtRule {
    Media(String),
    Supports(String),
    Container(String),
    Keyframes(String),
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct StyleRule {
    pub selector: Selector,
    pub declaration: Declaration,
    #[allow(unused)] pub at_rules: Option<Vec<AtRule>>,
}
impl StyleRule {
    fn make_for_tag_name(tag_name: &str, value: CssPropertyValue) -> StyleRule {
        return StyleRule { selector: Selector { elements: vec![(CssCombinator::None, SelectorType::Name, tag_name.to_owned())],
                                                pseudoclasses: None, pseudoelement: None },
                           declaration: Declaration { value, important: false }, at_rules: None }
    }
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Declaration {
    pub value: CssPropertyValue,
    pub important: bool,
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
    #[allow(unused)] pub pseudoclasses: Option<Vec<String>>,  //TODO: use
    #[allow(unused)] pub pseudoelement: Option<PseudoElement>,  //TODO: use
}


#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum PseudoElement {
    Before,
    After,
}


#[derive(PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
enum Origin {
    Author,
    UserAgent,
    //we don't implement the USER origin
}


#[cfg_attr(debug_assertions, derive(Debug))]
struct ActiveStyleRule {
    declaration: Declaration,
    origin: Origin,
    element_attached: bool,
    specificity_a_id: u8,
    specificity_b_class: u8,
    specificity_c_type: u8,
    definition_order: u32,
}


pub fn resolve_full_styles_for_dom_node(dom_node: &Rc<RefCell<ElementDomNode>>, all_dom_nodes: &HashMap<usize, Rc<RefCell<ElementDomNode>>>,
                                        style_context: &StyleContext) -> Styles {

    //TODO: we are doing the cascade here by first doing the ua sheet, and then the author sheet. We need to make this more general in cascades
    //      because we need to support @layer, which adds an arbitrary amount of cascades

    let mut rule_idx = 1;

    let mut active_style_rules = Vec::new();
    for style_rule in &style_context.user_agent_sheet {
        if style_rule_applies(&style_rule, &dom_node, all_dom_nodes) {
            active_style_rules.push(
                ActiveStyleRule {
                    declaration: style_rule.declaration.clone(),
                    origin: Origin::UserAgent,
                    definition_order: rule_idx,
                    element_attached: false,
                    specificity_a_id: 0, //TODO: implement
                    specificity_b_class: 0, //TODO: implement
                    specificity_c_type: 0, //TODO: implement
                }
            );
        }
        rule_idx += 1;
    }

    for style_rule in &style_context.author_sheet {
        if style_rule_applies(&style_rule, &dom_node, all_dom_nodes) {
            active_style_rules.push(
                ActiveStyleRule {
                    declaration: style_rule.declaration.clone(),
                    origin: Origin::Author,
                    definition_order: rule_idx,
                    element_attached: false,
                    specificity_a_id: 0, //TODO: implement
                    specificity_b_class: 0, //TODO: implement
                    specificity_c_type: 0, //TODO: implement
                }
            );
        }
        rule_idx += 1;
    }

    for declaration in &dom_node.borrow().inline_style {
        active_style_rules.push(
            ActiveStyleRule {
                declaration: declaration.clone(),
                origin: Origin::Author,
                definition_order: rule_idx,
                element_attached: true,
                specificity_a_id: 0,
                specificity_b_class: 0,
                specificity_c_type: 0,
            }
        );
        rule_idx += 1;
    }


    active_style_rules.sort_by(|rule_a, rule_b| compare_style_rules(rule_a, rule_b));

    let mut resolved_styles = Styles::new();
    for active_style_rule in active_style_rules {
        resolved_styles.insert(active_style_rule.declaration.value);
    }

    if dom_node.borrow().parent_id != 0 {
        let parent_node = all_dom_nodes.get(&dom_node.borrow().parent_id).expect(format!("id {} not present in all nodes", dom_node.borrow().parent_id).as_str());
        resolved_styles.inherit_from(&parent_node.borrow().styles);
    }

    return resolved_styles;
}


pub fn compute_styles(dom_node: &Rc<RefCell<ElementDomNode>>, all_dom_nodes: &HashMap<usize, Rc<RefCell<ElementDomNode>>>, style_context: &StyleContext) {
    let computed_styles = resolve_full_styles_for_dom_node(&dom_node, all_dom_nodes, style_context);

    dom_node.borrow_mut().styles = computed_styles;

    for child in &dom_node.borrow().children {
        compute_styles(child, all_dom_nodes, style_context);
    }
}


const HTML_BLOCK_ELEMENTS: &[&'static str] = &["div", "form", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "p", "table", "html", "body",
                                               "main", "header", "footer", "nav", "section", "ul"];


pub fn get_user_agent_style_sheet() -> Vec<StyleRule> {
    let mut rules = vec![
        //TODO: convert to an actual stylesheet (CSS string) we load in?

        //TODO: fontsizes here are probably not correct anymore after our style typing...
        StyleRule::make_for_tag_name("h1", CssPropertyValue::FontSize(FontSize::Px(32.0))),
        StyleRule::make_for_tag_name("h2", CssPropertyValue::FontSize(FontSize::Px(30.0))),
        StyleRule::make_for_tag_name("h3", CssPropertyValue::FontSize(FontSize::Px(28.0))),
        StyleRule::make_for_tag_name("h4", CssPropertyValue::FontSize(FontSize::Px(26.0))),
        StyleRule::make_for_tag_name("h5", CssPropertyValue::FontSize(FontSize::Px(24.0))),
        StyleRule::make_for_tag_name("h6", CssPropertyValue::FontSize(FontSize::Px(22.0))),

        StyleRule::make_for_tag_name("b", CssPropertyValue::FontWeight(FontWeight::Bold)),
        StyleRule::make_for_tag_name("i", CssPropertyValue::FontStyle(FontStyle::Italic)),

        StyleRule::make_for_tag_name("a", CssPropertyValue::Color(Color::BLUE)),
        StyleRule::make_for_tag_name("a", CssPropertyValue::TextDecoration(TextDecoration::Underline)),

        StyleRule::make_for_tag_name("script", CssPropertyValue::Display(Display::None)),
        StyleRule::make_for_tag_name("title", CssPropertyValue::Display(Display::None)),
        StyleRule::make_for_tag_name("style", CssPropertyValue::Display(Display::None)),
        StyleRule::make_for_tag_name("head", CssPropertyValue::Display(Display::None)),
        StyleRule::make_for_tag_name("meta", CssPropertyValue::Display(Display::None)),
        StyleRule::make_for_tag_name("link", CssPropertyValue::Display(Display::None)),
    ];

    for element in HTML_BLOCK_ELEMENTS {
        rules.push(StyleRule::make_for_tag_name(element, CssPropertyValue::Display(Display::Block)));
    }

    return rules;
}


// This function returns what rule_a is compared to rule_b (less, equal or greater), greater meaning having higher priority
fn compare_style_rules(rule_a: &ActiveStyleRule, rule_b: &ActiveStyleRule) -> Ordering {

    //TODO: this check needs to be different in the future, because we need to support an arbitrary set of cascades
    if rule_a.origin != rule_b.origin {
        if rule_a.origin == Origin::UserAgent {
            return Ordering::Less;
        }
        return Ordering::Greater;
    }

    if rule_a.declaration.important != rule_b.declaration.important {
        if rule_a.declaration.important { return Ordering::Greater } else { return Ordering::Less }
    }

    if rule_a.element_attached != rule_b.element_attached {
        if rule_a.element_attached { return Ordering::Greater } else { return Ordering::Less }
    }

    if rule_a.specificity_a_id > rule_b.specificity_a_id { return Ordering::Greater; }
    if rule_a.specificity_a_id < rule_b.specificity_a_id { return Ordering::Less; }

    if rule_a.specificity_b_class > rule_b.specificity_b_class { return Ordering::Greater; }
    if rule_a.specificity_b_class < rule_b.specificity_b_class { return Ordering::Less; }

    if rule_a.specificity_c_type > rule_b.specificity_c_type { return Ordering::Greater; }
    if rule_a.specificity_c_type < rule_b.specificity_c_type { return Ordering::Less; }

    if rule_a.definition_order > rule_b.definition_order { return Ordering::Greater; }
    if rule_a.definition_order < rule_b.definition_order { return Ordering::Less; }

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
                    let node_id = node_being_checked_borr.get_attribute_value("id");
                    if node_id.is_none() || node_id.unwrap().as_str() != selector_part {
                        return false;
                    }
                },
                SelectorType::Class => {
                    let all_classes = node_being_checked_borr.get_attribute_value("class");
                    let any_class_match = all_classes.is_some() && all_classes.unwrap().split_whitespace().any(|class| class == selector_part);
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
                for child in parent_node.borrow().children.iter().rev() {
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