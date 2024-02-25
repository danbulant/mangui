use std::sync::{Arc, Mutex, RwLock};
use mangui::nodes::layout::Layout;
use mangui::nodes::primitives::Rectangle;
use rusalka::component::Slot;
use rusalka::store::{DerefGuardExt, ReadableStore, Signal, StoreUnsubscribe, Writable, WritableStore};

pub struct SlotAcceptDemo {
    comp0: rusalka::SharedNodeComponent<Layout>,
    comp1: Mutex<Option<Slot>>,
    selfref: rusalka::WeakSharedComponent<Self>,
    attrs: ReactiveSlotAcceptDemoAttributes,
}

type SlotArgs = ();
type DSlot = Option<Mutex<Box<dyn FnMut(SlotArgs) -> Slot>>>;

#[derive(Default)]
pub struct SlotAcceptDemoAttributes {
    pub __default_slot: DSlot
}

pub struct ReactiveSlotAcceptDemoAttributes {
    __default_slot: DSlot
}

#[derive(Default)]
pub struct PartialSlotAcceptDemoAttributes {
    pub __default_slot: Option<DSlot>
}
impl From<SlotAcceptDemoAttributes> for PartialSlotAcceptDemoAttributes {
    fn from(attrs: SlotAcceptDemoAttributes) -> Self {
        Self { __default_slot: Some(attrs.__default_slot) }
    }
}

impl From<SlotAcceptDemoAttributes> for ReactiveSlotAcceptDemoAttributes {
    fn from(attrs: SlotAcceptDemoAttributes) -> Self {
        Self { __default_slot: attrs.__default_slot }
    }
}

impl rusalka::component::Component for SlotAcceptDemo {
    type ComponentAttrs = SlotAcceptDemoAttributes;
    type ReactiveComponentAttrs = ReactiveSlotAcceptDemoAttributes;
    type PartialComponentAttrs = PartialSlotAcceptDemoAttributes;
    const UPDATE_LENGTH: usize = 1;
    fn new(
        attrs: Self::ComponentAttrs,
        selfref: rusalka::WeakSharedComponent<Self>,
    ) -> Self {
        let this = Self {
            comp0: std::sync::Arc::new(
                std::sync::RwLock::new(Layout { ..Default::default() }),
            ),
            comp1: Mutex::new(None),
            attrs: attrs.into(),
            selfref
        };
        this
    }
    fn set(&mut self, attrs: Self::PartialComponentAttrs) {

    }
    fn get(&self) -> &Self::ReactiveComponentAttrs {
        &self.attrs
    }
    fn mount(
        &self,
        parent: &mangui::SharedNode,
        before: Option<&mangui::SharedNode>,
    ) {
        rusalka::nodes::insert(parent, &{ self.comp0.clone() }, before);
        match &self.attrs.__default_slot {
            Some(slot) => {
                *self.comp1.lock().unwrap() = Some(slot.lock().unwrap()(()));
                (*self.comp1.lock().unwrap().as_mut().unwrap().mount)(parent, before);
            }
            None => {}
        }
    }
    fn unmount(&self) {
        rusalka::nodes::detach(&{ self.comp0.clone() });
        match &self.attrs.__default_slot {
            Some(slot) => {
                (*self.comp1.lock().unwrap().as_mut().unwrap().unmount)();
                *self.comp1.lock().unwrap() = None;
            }
            None => {}
        }
    }
}

struct SlotDemoSlot1 {
    comp0: rusalka::SharedNodeComponent<Rectangle>,
    sub0: Box<dyn StoreUnsubscribe>
}

pub struct SlotDemo {
    comp0: rusalka::SharedNodeComponent<Layout>,
    comp1: rusalka::SharedComponent<SlotAcceptDemo>,
    test_: std::sync::Arc<std::sync::Mutex<rusalka::store::Writable<bool>>>,
    sub0: Box<dyn StoreUnsubscribe>,
    selfref: rusalka::WeakSharedComponent<Self>,
    attrs: ReactiveSlotDemoAttributes,
}
#[derive(Default)]
pub struct SlotDemoAttributes {
    pub test: f32
}

pub struct ReactiveSlotDemoAttributes {
    pub test: Arc<Mutex<Writable<f32>>>
}

#[derive(Default)]
pub struct PartialSlotDemoAttributes {
    pub test: Option<f32>
}
impl From<SlotDemoAttributes> for PartialSlotDemoAttributes {
    fn from(attrs: SlotDemoAttributes) -> Self {
        Self { test: Some(attrs.test) }
    }
}

impl From<ReactiveSlotDemoAttributes> for SlotDemoAttributes {
    fn from(attrs: ReactiveSlotDemoAttributes) -> Self {
        Self { test: *attrs.test.lock().unwrap().get() }
    }
}
impl From<&ReactiveSlotDemoAttributes> for SlotDemoAttributes {
    fn from(attrs: &ReactiveSlotDemoAttributes) -> Self {
        Self { test: *attrs.test.lock().unwrap().get() }
    }
}
impl From<SlotDemoAttributes> for ReactiveSlotDemoAttributes {
    fn from(attrs: SlotDemoAttributes) -> Self {
        Self { test: Arc::new(Mutex::new(Writable::new(attrs.test))) }
    }
}

impl rusalka::component::Component for SlotDemo {
    type ComponentAttrs = SlotDemoAttributes;
    type ReactiveComponentAttrs = ReactiveSlotDemoAttributes;
    type PartialComponentAttrs = PartialSlotDemoAttributes;
    const UPDATE_LENGTH: usize = 1;
    fn new(
        attrs: Self::ComponentAttrs,
        selfref: rusalka::WeakSharedComponent<Self>,
    ) -> Self {
        let attrs: Self::ReactiveComponentAttrs = attrs.into();
        let test_: std::sync::Arc<
            std::sync::Mutex<rusalka::store::Writable<bool>>,
        > = std::sync::Arc::new(
            std::sync::Mutex::new(rusalka::store::Writable::new(false)),
        );
        let test = attrs.test.clone();
        let this = Self {
            comp0: std::sync::Arc::new(
                std::sync::RwLock::new(Layout { ..Default::default() }),
            ),
            comp1: std::sync::Arc::new_cyclic(|cselfref2| std::sync::Mutex::new(
                SlotAcceptDemo::new(
                    SlotAcceptDemoAttributes {
                        __default_slot: Some(Mutex::new(Box::new(move |_| {
                            let comp0: Arc<Mutex<Option<SlotDemoSlot1>>> = Arc::new(Mutex::new(None));
                            Slot {
                                mount: {
                                    let comp0 = comp0.clone();
                                    let test = test.clone();
                                    Box::new(move |parent, before| {
                                        if let None = comp0.lock().unwrap().as_ref() {
                                            let slot = Some(
                                                SlotDemoSlot1 {
                                                    comp0: std::sync::Arc::new(
                                                        std::sync::RwLock::new(Rectangle { radius: *test.clone().lock().unwrap().get(), ..Default::default() }),
                                                    ),
                                                    sub0: {
                                                        let comp0 = comp0.clone();
                                                        let test = test.clone();
                                                        [test.clone().lock().unwrap()].subscribe(Box::new(move || {
                                                            let comp1 = comp0.clone();
                                                            let test1 = test.clone();
                                                            let mut comp1l = comp1.lock().unwrap();
                                                            if let Some(comp1) = comp1l.as_mut() {
                                                                comp1.comp0.write().unwrap().radius = *test1.lock().unwrap().get();
                                                            }
                                                        }))
                                                    },
                                                }
                                            );
                                            *comp0.lock().unwrap() = slot;
                                        }
                                        rusalka::nodes::insert(parent, &{ comp0.lock().as_ref().unwrap().as_ref().unwrap().comp0.clone() }, before);
                                    })
                                },
                                unmount: {
                                    let comp0 = comp0.clone();
                                    Box::new(move || {
                                        let comp0 = comp0.clone();
                                        if let Some(comp0) = comp0.lock().unwrap().as_mut() {
                                            rusalka::nodes::detach(&{ comp0.comp0.clone() });
                                        }
                                        *comp0.lock().unwrap() = None;
                                    })
                                },
                            }
                        })))
                    },
                    cselfref2.clone(),
                ),
            )),
            sub0: {
                let test = test_.clone();
                [test.clone().lock().unwrap()].subscribe(Box::new(move || {
                    let test = test.clone();
                    dbg!(test.lock().unwrap().get());
                }))
            },
            attrs,
            selfref,
            test_
        };
        let selfref = this.selfref.clone();
        this.comp0
            .write()
            .unwrap()
            .events
            .add_handler(
                Box::new(move |event| {
                    let selfref = selfref.upgrade().unwrap();
                    let mut this = selfref.lock().unwrap();
                    let attrs = &this.attrs;
                    let test_ = &this.test_;
                    match event.event {
                        mangui::events::InnerEvent::MouseDown(_) => {
                            **test_.lock().unwrap().guard() = true;
                        }
                        mangui::events::InnerEvent::MouseUp(_) => {
                            **test_.lock().unwrap().guard() = false;
                        }
                        _ => {}
                    }
                }),
            );
        this
    }
    fn set(&mut self, attrs: Self::PartialComponentAttrs) {
        if let Some(test) = attrs.test {
            **self.attrs.test.lock().unwrap().guard() = test;
            // self.attrs.test.lock().unwrap().set(test);
        }
    }
    fn get(&self) -> &Self::ReactiveComponentAttrs {
        &self.attrs
    }
    fn mount(
        &self,
        parent: &mangui::SharedNode,
        before: Option<&mangui::SharedNode>,
    ) {
        rusalka::nodes::insert(parent, &{ self.comp0.clone() }, before);
    }
    fn unmount(&self) {
        rusalka::nodes::detach(&{ self.comp0.clone() });
    }
}