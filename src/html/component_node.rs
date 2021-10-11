use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use component::assembled_component::AssembledComponentInstance;
use component::{AssembledChildComponent, Render, Update};

impl<ThisComp: Update + Render, DemirootComp: Component>
    PackedComponentNodeInstance<ThisComp, DemirootComp>
{
    pub fn new(
        constructor: fn(&ThisComp::Props) -> ThisComp,
        props: ThisComp::Props,
        sub_mapper: component::Sub<ThisComp::Sub, DemirootComp::Msg>,
        children: Vec<Html<DemirootComp>>,
    ) -> Self {
        Self {
            data: Some(PackedComponentNodeInstanceData {
                constructor,
                props,
                sub_mapper,
                children,
            }),
        }
    }
}

impl<ThisComp: Update + Render, DemirootComp: Component> PackedComponentNode
    for PackedComponentNodeInstance<ThisComp, DemirootComp>
{
    type DemirootComp = DemirootComp;

    fn wrap(&mut self) -> Box<dyn WrappedPackedComponentNode> {
        let data = self.data.take();
        Box::new(WrappedPackedComponentNodeInstance {
            data: Box::new(Self { data }),
        })
    }

    fn assemble(
        &mut self,
        before: Option<Rc<RefCell<dyn AssembledChildComponent<DemirootComp = Self::DemirootComp>>>>,
    ) -> AssembledComponentNode<Self::DemirootComp> {
        let before = before.and_then(|before| {
            before
                .borrow_mut()
                .as_any()
                .downcast_mut::<AssembledComponentInstance<ThisComp, DemirootComp>>()
                .map(|before_instance| {
                    let data = self.data.take().unwrap();
                    before_instance.set_props(data.props);
                    before_instance.set_sub_mapper(data.sub_mapper);

                    (Rc::clone(&before), data.children)
                })
        });

        if let Some((data, children)) = before {
            AssembledComponentNode::new(data, children)
        } else {
            let data = self.data.take().unwrap();
            let props = data.props;
            let sub_mapper = data.sub_mapper;
            let children = data.children;
            let data = (data.constructor)(&props);
            let data =
                AssembledComponentInstance::new_ref(Rc::new(RefCell::new(data)), props, sub_mapper);

            AssembledComponentNode::new(data, children)
        }
    }
}

impl<SuperDemirootComp: Component> WrappedPackedComponentNodeInstance<SuperDemirootComp> {
    pub fn assemble(
        &mut self,
        before: Option<Rc<RefCell<dyn AssembledChildComponent<DemirootComp = SuperDemirootComp>>>>,
    ) -> AssembledComponentNode<SuperDemirootComp> {
        self.data.assemble(before)
    }
}

impl<SuperDemirootComp: Component> WrappedPackedComponentNode
    for WrappedPackedComponentNodeInstance<SuperDemirootComp>
{
}

impl<DemirootComp: Component> AssembledComponentNode<DemirootComp> {
    pub fn new(
        data: Rc<RefCell<dyn AssembledChildComponent<DemirootComp = DemirootComp>>>,
        children: Vec<Html<DemirootComp>>,
    ) -> Self {
        Self { data, children }
    }

    pub fn wrap(self) -> Box<dyn WrappedAssembledComponentNode> {
        Box::new(WrappedAssembledComponentNodeInstance { data: Some(self) })
    }
}

impl<SuperDemirootComp: Component> WrappedAssembledComponentNodeInstance<SuperDemirootComp> {
    pub fn take(&mut self) -> AssembledComponentNode<SuperDemirootComp> {
        self.data.take().unwrap()
    }
}

impl<SuperDemirootComp: Component> WrappedAssembledComponentNode
    for WrappedAssembledComponentNodeInstance<SuperDemirootComp>
{
}
