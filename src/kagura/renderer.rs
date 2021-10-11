use super::*;
use std::collections::VecDeque;
use wasm_bindgen::{prelude::*, JsCast};

pub struct Renderer {
    befores: VecDeque<Node>,
    document: web_sys::Document,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            befores: VecDeque::new(),
            document: web_sys::window().unwrap().document().unwrap(),
        }
    }

    pub fn render(&mut self, afters: VecDeque<Node>, r_befores_parent: &web_sys::Node) {
        let mut befores = VecDeque::new();
        std::mem::swap(&mut self.befores, &mut befores);
        let afters: VecDeque<_> = afters.into_iter().collect();

        self.befores = self.render_node_list(befores, afters, r_befores_parent);
    }

    fn create_element(&self, tag_name: &str) -> web_sys::Element {
        self.document.create_element(tag_name).unwrap()
    }

    fn create_text_node(&self, data: &str) -> web_sys::Text {
        self.document.create_text_node(data)
    }

    fn render_node_list(
        &self,
        mut befores: VecDeque<Node>,
        afters: VecDeque<Node>,
        r_befores_parent: &web_sys::Node,
    ) -> VecDeque<Node> {
        let r_befores = r_befores_parent.child_nodes();
        let r_befores_len = r_befores.length();
        let mut res = VecDeque::new();
        let mut idx = 0;

        for after in afters {
            let before = befores.pop_front();
            let r_before = r_befores.get(idx);

            let (res_node, r_node) = self.diff_render_node(before, after, r_before.as_ref());

            if let Some(r_node) = r_node {
                if r_befores_len > idx {
                    let _ = r_befores_parent.replace_child(&r_node, &r_before.unwrap());
                } else {
                    let _ = r_befores_parent.append_child(&r_node);
                }
            }

            res.push_back(res_node);
            idx += 1;
        }

        for _before in befores {
            let r_before = r_befores.get(idx);
            if let Some(r_before) = r_before {
                let _ = r_befores_parent.remove_child(&r_before);
            } else {
                break;
            }
        }

        res
    }

    fn diff_render_node(
        &self,
        before: Option<Node>,
        after: Node,
        r_before: Option<&web_sys::Node>,
    ) -> (Node, Option<web_sys::Node>) {
        match after {
            Node::Element(after_element_node) => match before {
                Some(Node::Element(before_elememnt_node))
                    if before_elememnt_node.tag_name == after_element_node.tag_name =>
                {
                    if let Some(r_before_element) =
                        r_before.and_then(|x| x.dyn_ref::<web_sys::Element>())
                    {
                        let res = self.diff_render_element(
                            before_elememnt_node,
                            after_element_node,
                            r_before_element,
                        );
                        (res, None)
                    } else {
                        let (res, r_node) = self.force_render_element(after_element_node);
                        (res, Some(r_node))
                    }
                }
                _ => {
                    let (res, r_node) = self.force_render_element(after_element_node);
                    (res, Some(r_node))
                }
            },
            Node::Text(mut after_text_node) => match before {
                Some(Node::Text(before_text_node))
                    if before_text_node.text == after_text_node.text =>
                {
                    if let Some(r_before_text) = r_before.and_then(|x| x.dyn_ref::<web_sys::Text>())
                    {
                        after_text_node.events = self.force_render_node_event(
                            before_text_node.events,
                            after_text_node.events,
                            &r_before_text,
                        );
                        (Node::Text(after_text_node), None)
                    } else {
                        let r_text = self.create_text_node(&after_text_node.text);
                        after_text_node.events = self.force_render_node_event(
                            node::Events::new(),
                            after_text_node.events,
                            &r_text,
                        );
                        (Node::Text(after_text_node), Some(r_text.into()))
                    }
                }
                _ => {
                    let r_text = self.create_text_node(&after_text_node.text);
                    after_text_node.events = self.force_render_node_event(
                        node::Events::new(),
                        after_text_node.events,
                        &r_text,
                    );
                    (Node::Text(after_text_node), Some(r_text.into()))
                }
            },
        }
    }

    fn diff_render_element(
        &self,
        before: node::ElementNode,
        mut after: node::ElementNode,
        r_before: &web_sys::Element,
    ) -> Node {
        after.attributes =
            self.diff_render_element_attribute(before.attributes, after.attributes, &r_before);
        after.events = self.force_render_node_event(before.events, after.events, &r_before);
        after.children = self.render_node_list(before.children, after.children, &r_before);
        Node::Element(after)
    }

    fn force_render_element(&self, mut after: node::ElementNode) -> (Node, web_sys::Node) {
        let r_element = self.create_element(&after.tag_name);
        after.attributes = self.force_render_element_attribute(after.attributes, &r_element);
        after.events = self.force_render_node_event(node::Events::new(), after.events, &r_element);
        (Node::Element(after), r_element.into())
    }

    fn diff_render_element_attribute(
        &self,
        before: node::Attributes,
        after: node::Attributes,
        r_before: &web_sys::Element,
    ) -> node::Attributes {
        for (a, _) in &before.attributes {
            if !after.attributes.contains_key(a) {
                let _ = r_before.remove_attribute(a);
            }
        }

        let mut diff = node::Attributes::new();
        for (name, after_values) in &after.attributes {
            if !self.compare_elment_attribute(&before, &after, name, after_values) {
                for after_value in after_values {
                    diff.add(name, after_value.clone());
                }
                if let Some(d) = after.delimiters.get(name) {
                    diff.delimit(name, d);
                }
            }
        }

        self.force_render_element_attribute(diff, r_before);

        after
    }

    fn compare_elment_attribute(
        &self,
        before: &node::Attributes,
        after: &node::Attributes,
        name: &String,
        after_values: &Vec<node::Value>,
    ) -> bool {
        before
            .attributes
            .get(name)
            .map(|before_values| {
                let a = after.delimiters.get(name).map(String::as_str).unwrap_or("");
                let b = before
                    .delimiters
                    .get(name)
                    .map(String::as_str)
                    .unwrap_or("");

                *a == *b && *before_values == *after_values
            })
            .unwrap_or(false)
    }

    fn force_render_element_attribute(
        &self,
        after: node::Attributes,
        r_before: &web_sys::Element,
    ) -> node::Attributes {
        for (name, after_values) in &after.attributes {
            if after_values.is_empty() {
                let _ = r_before.set_attribute(name, "");
            } else {
                let delimiter = after.delimiters.get(name).map(String::as_str).unwrap_or("");
                let after_values = after_values
                    .iter()
                    .map(node::Value::as_rc_string)
                    .collect::<Vec<_>>();
                let after_values = after_values
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<Vec<&str>>()
                    .join(delimiter);
                if name == "value" {
                    if let Some(r_before) = r_before.dyn_ref::<web_sys::HtmlInputElement>() {
                        r_before.set_value(&after_values);
                    } else if let Some(r_before) =
                        r_before.dyn_ref::<web_sys::HtmlTextAreaElement>()
                    {
                        r_before.set_value(&after_values);
                    } else {
                        let _ = r_before.set_attribute(name, &after_values);
                    }
                } else {
                    let _ = r_before.set_attribute(name, &after_values);
                }
            }
        }

        after
    }

    fn force_render_node_event(
        &self,
        before: node::Events,
        mut after: node::Events,
        r_before: &web_sys::Node,
    ) -> node::Events {
        for (event_name, before_handlers) in &before.handler_table {
            let mut idx = 0;
            for before_handler in before_handlers {
                if let node::Event::HandlerId(before_hid) = before_handler {
                    crate::env::remove_event_handler(&before_hid);
                    if let Some(after_handler) = after
                        .handler_table
                        .get_mut(event_name)
                        .and_then(|x| x.get_mut(idx))
                        .and_then(|x| x.take_with_id(*before_hid))
                    {
                        crate::env::add_event_handler(*before_hid, after_handler);
                    } else if let Some(after_handlers) = after.handler_table.get_mut(event_name) {
                        after_handlers.push(node::Event::HandlerId(*before_hid));
                    } else {
                        after.handler_table.insert(
                            event_name.clone(),
                            vec![node::Event::HandlerId(*before_hid)],
                        );
                    }
                }
                idx += 1;
            }
        }

        for (event_name, after_handlers) in &mut after.handler_table {
            for after_handler in after_handlers {
                if after_handler.is_handler() {
                    let after_hid = crate::env::gen_id();
                    let after_handler = after_handler.take_with_id(after_hid).unwrap();

                    crate::env::add_event_handler(after_hid, after_handler);

                    let a = Closure::wrap(Box::new(move |e| {
                        crate::env::dispatch_event(after_hid, e);
                    }) as Box<dyn FnMut(web_sys::Event)>);
                    let _ = r_before
                        .add_event_listener_with_callback(event_name, a.as_ref().unchecked_ref());
                    a.forget();
                }
            }
        }

        after
    }
}
