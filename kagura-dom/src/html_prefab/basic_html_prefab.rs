use super::HtmlPrefab;
use crate::html_node::BasicHtmlNode;
use crate::{Html, HtmlNode};
use kagura::component::{Render, Update};
use kagura::node::SubHandler;
use std::any::Any;

pub struct BasicHtmlPrefab<This: Update + Render<Html> + 'static> {
    constructor: Box<dyn FnOnce(This::Props) -> This>,
    children: This::Children,
    props: This::Props,
    index_id: Option<String>,
    sub_handler: Option<SubHandler<This>>,
}

impl<This: Update + Render<Html>> BasicHtmlPrefab<This> {
    pub fn new(
        constructor: impl FnOnce(This::Props) -> This + 'static,
        index_id: Option<String>,
        props: This::Props,
        sub_handler: Option<SubHandler<This>>,
        children: This::Children,
    ) -> Self {
        Self {
            constructor: Box::new(constructor),
            index_id,
            props,
            sub_handler,
            children,
        }
    }

    pub fn into_data(
        self,
    ) -> (
        This::Props,
        Option<String>,
        Option<SubHandler<This>>,
        This::Children,
    ) {
        (self.props, self.index_id, self.sub_handler, self.children)
    }
}

impl<This: Update + Render<Html>> HtmlPrefab for BasicHtmlPrefab<This> {
    fn component_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<This>()
    }

    fn index_id(&self) -> &Option<String> {
        &self.index_id
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(*self)
    }

    fn into_node(self: Box<Self>) -> Box<dyn HtmlNode> {
        let state = (self.constructor)(self.props);
        let index_id = self.index_id;
        let sub_handler = self.sub_handler;
        Box::new(BasicHtmlNode::new(index_id, sub_handler, Box::pin(state)))
    }
}
