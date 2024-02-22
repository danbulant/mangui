use std::collections::HashMap;

use proc_macro::{TokenStream, TokenTree, Ident, Group, Span, Literal};
use quote::quote;

#[derive(Debug, Clone)]
struct Attribute {
    name: Ident,
    /// Default value - ignored in Attributes, required in variables
    default: Option<TokenStream>,
    type_: TokenStream,
    variable_type: VariableType
}

#[derive(Debug, Clone)]
enum VariableType {
    Variable,
    Attribute
}

#[derive(Debug)]
struct EventListener {
    /// The callback itself, as the group
    callback: Group,
    /// The identifier of 'event' argument in callback (usually just event)
    identifier: Ident
}

#[derive(Debug)]
enum ComponentType {
    RealComponent,
    Node
}

#[derive(Debug)]
struct ComponentUsed {
    name: Ident,
    contents: TokenStream,
    parent: Option<usize>,
    component_type: ComponentType,
    event_listeners: Vec<EventListener>,
    reactive_props: HashMap<String, ReactiveBlock>
}

#[derive(Debug)]
struct ReactiveBlock {
    variables: Vec<Ident>,
    contents: TokenStream,
    prop_ident: Option<Ident>
}

#[proc_macro]
/// If you have syntax errors because of attributes, wrap the default value in parentheses.
pub fn make_component(item: TokenStream) -> TokenStream {
    dbg!(&item);

    let mut last_identifier = None;
    let mut item = item.into_iter();
    let name = item.next().unwrap();
    item.next().unwrap();
    let name_ident = match name {
        TokenTree::Ident(ident) => ident,
        _ => panic!("Expected ident")
    };
    let str_name = name_ident.to_string();

    dbg!(&name_ident);

    let mut attributes: Vec<Attribute> = Vec::new();

    // let mut struct_values = Vec::new();

    let mut main_logic = Vec::new();

    let mut reactive_variables = Vec::new();

    let mut components_used: Vec<ComponentUsed> = Vec::new();

    let mut reactive_blocks = Vec::new();

    for token in item {
        match token {
            TokenTree::Ident(ident) => {
                last_identifier = Some(ident.to_string());
                let ident = ident.to_string();

                match ident.as_str() {
                    "MainLogic" | "Component" | "Attributes" | "Variables" | "Reactive" => {},
                    _ => panic!("Unknown identifier: {:?}", ident)
                }
            },
            TokenTree::Group(group) => {
                match &last_identifier {
                    Some(ident) => {
                        match ident.as_str() {
                            "Attributes" | "Variables" => {
                                // A struct-like definition of attributes
                                // Example syntax:
                                // Attributes { // we're here
                                //   radius: f32 = 5.,
                                //   fill: Paint,
                                //   something: Something<(dyn Test)>
                                // }
                                // we need to match <> together, other groupings are already done by rust

                                let mut stream = group.stream().into_iter();

                                while let Some(token) = stream.next() {
                                    let name = match token {
                                        TokenTree::Ident(ident) => ident,
                                        _ => panic!("Expected ident")
                                    };

                                    let colon = stream.next().unwrap();
                                    let _colon = match colon {
                                        TokenTree::Punct(punct) => {
                                            if punct.as_char() != ':' {
                                                panic!("Expected :");
                                            }
                                            punct
                                        },
                                        _ => panic!("Expected :")
                                    };

                                    let mut type_ = TokenStream::new();

                                    let mut last_was_set = false;
                                    let mut ltgt_count = 0;

                                    while let Some(token) = stream.next() {
                                        match token {
                                            TokenTree::Ident(ident) => {
                                                type_.extend(Some(TokenTree::Ident(ident)));
                                            },
                                            TokenTree::Punct(punct) => {
                                                if ltgt_count == 0 && punct.as_char() == ',' {
                                                    break;
                                                } else if ltgt_count == 0 && punct.as_char() == '=' {
                                                    last_was_set = true;
                                                    break;
                                                } else {
                                                    if punct.as_char() == '<' {
                                                        ltgt_count += 1;
                                                    } else if punct.as_char() == '>' {
                                                        ltgt_count -= 1;
                                                    }
                                                    type_.extend(Some(TokenTree::Punct(punct)));
                                                }
                                            },
                                            _ => {
                                                type_.extend(Some(token));
                                            }
                                        }
                                    }

                                    let variable_type;

                                    let array = if ident.as_str() == "Variables" {
                                        variable_type = VariableType::Variable;
                                        &mut reactive_variables
                                    } else {
                                        variable_type = VariableType::Attribute;
                                        &mut attributes
                                    };

                                    if last_was_set {
                                        let mut default = TokenStream::new();

                                        while let Some(token) = stream.next() {
                                            match token {
                                                TokenTree::Ident(ident) => {
                                                    default.extend(Some(TokenTree::Ident(ident)));
                                                },
                                                TokenTree::Punct(punct) => {
                                                    if punct.as_char() == ',' {
                                                        break;
                                                    } else {
                                                        default.extend(Some(TokenTree::Punct(punct)));
                                                    }
                                                },
                                                _ => {
                                                    default.extend(Some(token));
                                                }
                                            }
                                        }

                                        array.push(Attribute {
                                            name,
                                            default: Some(default),
                                            type_,
                                            variable_type
                                        });
                                    } else {
                                        array.push(Attribute {
                                            name,
                                            default: None,
                                            type_,
                                            variable_type
                                        });
                                    }
                                }
                            },
                            "MainLogic" => {
                                main_logic.push(group.stream());
                            },
                            "Reactive" => {
                                let (variables, contents) = replace_variables(group.stream());
                                reactive_blocks.push(ReactiveBlock {
                                    variables,
                                    contents,
                                    prop_ident: None
                                });
                            },
                            "Component" => {
                                // Example syntax:
                                // @Layout {
                                //     style: Style { ... },
                                //     @Rectangle {
                                //         fill: Paint::color(Color::rgb(0, 0, 255)),
                                //     }
                                //     @Rectangle {
                                //         fill: Paint::color(Color::rgb(0, 0, 255)),
                                //     }
                                // }
                                // non-component properties cannot contain components
                                // top level must contain only components
                                // components can contain components

                                let mut stream = group.stream().into_iter();

                                while let Some(token) = stream.next() {
                                    match token {
                                        TokenTree::Punct(punct) => {
                                            if punct.as_char() != '@' {
                                                panic!("Expected @");
                                            }
                                        },
                                        _ => panic!("Expected @")
                                    };

                                    let ident = stream.next().unwrap();
                                    let ident = match ident {
                                        TokenTree::Ident(ident) => ident,
                                        _ => panic!("Expected ident after @")
                                    };

                                    let group = stream.next().unwrap();
                                    let group = match group {
                                        TokenTree::Group(group) => group,
                                        _ => panic!("Expected group after ident")
                                    };

                                    let components = parse_components(ident, group, components_used.len(), None);
                                    components_used.extend(components);
                                }

                            },
                            _ => unreachable!()
                        }
                    },
                    None => {
                        panic!("Unexpected group: {:?}", group);
                    }
                }
            },
            _ => {
                panic!("Unknown token: {:?}", token);
            }
        }
    }

    let mut output = TokenStream::new();

    // Component struct

    output.extend(TokenStream::from(quote!(pub struct)));
    output.extend(Some(TokenTree::Ident(name_ident.clone())));

    let mut component_struct_stream = TokenStream::new();

    let mut i = 0;
    for component in &components_used {

        let ident = Ident::new(&format!("comp{}", i), component.name.span());

        let mut midstream = TokenStream::from(quote!(: rusalka::));

        match component.component_type {
            ComponentType::RealComponent => {
                midstream.extend(TokenStream::from(quote!(SharedComponent<)));
            },
            ComponentType::Node => {
                midstream.extend(TokenStream::from(quote!(SharedNodeComponent<)));
            }
        }

        let component_name = component.name.clone();

        component_struct_stream.extend(Some(TokenTree::Ident(ident)));
        component_struct_stream.extend(midstream);
        component_struct_stream.extend(Some(TokenTree::Ident(component_name)));
        component_struct_stream.extend(TokenStream::from(quote!(>,)));

        i+=1;
    }

    for variable in &reactive_variables {
        component_struct_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        component_struct_stream.extend(TokenStream::from(quote!(:)));
        component_struct_stream.extend(TokenStream::from(quote!(std::sync::Arc<std::sync::Mutex<rusalka::invalidator::Invalidator<)));
        component_struct_stream.extend(variable.type_.clone());
        component_struct_stream.extend(TokenStream::from(quote!(>>>)));
        component_struct_stream.extend(TokenStream::from(quote!(,)));
    }

    // selfref

    component_struct_stream.extend(TokenStream::from(quote!(selfref: rusalka::WeakSharedComponent<Self>,)));

    // Attributes struct

    let attributes_ident = Ident::new(&format!("{str_name}Attributes"), name_ident.span());

    component_struct_stream.extend(TokenStream::from(quote!(attrs: )));
    component_struct_stream.extend(Some(TokenTree::Ident(attributes_ident.clone())));

    let component_struct_group = Group::new(proc_macro::Delimiter::Brace, component_struct_stream);
    output.extend(Some(TokenTree::Group(component_struct_group)));


    // attributes

    let mut attributes_struct_stream = TokenStream::new();
    output.extend(TokenStream::from(quote!(#[derive(Default)] pub struct)));
    output.extend(Some(TokenTree::Ident(attributes_ident.clone())));

    for attribute in &attributes {
        attributes_struct_stream.extend(TokenStream::from(quote!(pub)));
        attributes_struct_stream.extend(Some(TokenTree::Ident(attribute.name.clone())));
        attributes_struct_stream.extend(TokenStream::from(quote!(:)));
        attributes_struct_stream.extend(attribute.type_.clone());
        attributes_struct_stream.extend(TokenStream::from(quote!(,)));
    }

    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, attributes_struct_stream))));

    // partial attributes

    let partial_attributes_ident = Ident::new(&format!("Partial{str_name}Attributes"), name_ident.span());
    output.extend(TokenStream::from(quote!(#[derive(Default)] pub struct)));
    output.extend(Some(TokenTree::Ident(partial_attributes_ident.clone())));
    let mut attributes_default_struct_stream = TokenStream::new();

    for attribute in &attributes {
        attributes_default_struct_stream.extend(TokenStream::from(quote!(pub)));
        attributes_default_struct_stream.extend(Some(TokenTree::Ident(attribute.name.clone())));
        attributes_default_struct_stream.extend(TokenStream::from(quote!(: Option<)));
        attributes_default_struct_stream.extend(attribute.type_.clone());
        attributes_default_struct_stream.extend(TokenStream::from(quote!(>,)));
    }

    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, attributes_default_struct_stream))));

    // impl From<Attributes> for PartialAttributes

    output.extend(TokenStream::from(quote!(impl From<)));
    output.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    output.extend(TokenStream::from(quote!(> for)));
    output.extend(Some(TokenTree::Ident(partial_attributes_ident.clone())));

    let mut from_stream = TokenStream::new();

    from_stream.extend(TokenStream::from(quote!(fn from)));

    let mut from_args = TokenStream::new();
    from_args.extend(TokenStream::from(quote!(attrs:)));
    from_args.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    from_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, from_args))));
    from_stream.extend(TokenStream::from(quote!(-> Self)));

    let mut from_fn_stream = TokenStream::from(quote!(Self));

    let mut from_fn_stream_inner = TokenStream::new();

    for attribute in &attributes {
        from_fn_stream_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));
        from_fn_stream_inner.extend(TokenStream::from(quote!(:)));
        from_fn_stream_inner.extend(TokenStream::from(quote!(Some)));

        let mut from_fn_stream_inner_inner = TokenStream::new();

        from_fn_stream_inner_inner.extend(TokenStream::from(quote!(attrs.)));
        from_fn_stream_inner_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));

        from_fn_stream_inner.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, from_fn_stream_inner_inner))));
    }

    from_fn_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, from_fn_stream_inner))));

    from_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, from_fn_stream))));
    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, from_stream))));

    // Component impl

    output.extend(TokenStream::from(quote!(impl rusalka::component::Component for)));
    output.extend(Some(TokenTree::Ident(name_ident.clone())));

    let mut component_impl_stream = TokenStream::new();

    component_impl_stream.extend(TokenStream::from(quote!(type ComponentAttrs =)));
    component_impl_stream.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    component_impl_stream.extend(TokenStream::from(quote!(;)));
    component_impl_stream.extend(TokenStream::from(quote!(type PartialComponentAttrs =)));
    component_impl_stream.extend(Some(TokenTree::Ident(partial_attributes_ident.clone())));
    component_impl_stream.extend(TokenStream::from(quote!(;)));
    component_impl_stream.extend(TokenStream::from(quote!(const UPDATE_LENGTH : usize =)));
    component_impl_stream.extend(Some(TokenTree::Literal(Literal::usize_unsuffixed(f64::ceil((attributes.len() + reactive_variables.len()) as f64 / 32 as f64) as usize))));
    component_impl_stream.extend(TokenStream::from(quote!(;)));

    // fn new

    component_impl_stream.extend(TokenStream::from(quote!(fn new(attrs: Self::ComponentAttrs, selfref: rusalka::WeakSharedComponent<Self>) -> Self)));

    let mut new_stream = TokenStream::new();

    for variable in &reactive_variables {
        new_stream.extend(TokenStream::from(quote!(let)));
        new_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        new_stream.extend(TokenStream::from(quote!(:)));
        new_stream.extend(TokenStream::from(quote!(std::sync::Arc<std::sync::Mutex<rusalka::invalidator::Invalidator<)));
        new_stream.extend(variable.type_.clone());
        new_stream.extend(TokenStream::from(quote!(>>>)));
        new_stream.extend(TokenStream::from(quote!(=)));

        let mut invalidator = TokenStream::from(quote!(rusalka::invalidator::Invalidator::new));
        let mut invalidator_inner = TokenStream::new();
        if let Some(def) = &variable.default {
            invalidator_inner.extend(replace_variables(def.clone()).1);
        } else {
            invalidator_inner.extend(TokenStream::from(quote!(Default::default())));
        }
        invalidator.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, invalidator_inner))));
        new_stream.extend(wrap_in_arc_mutex(invalidator));

        new_stream.extend(TokenStream::from(quote!(;)));
    }

    new_stream.extend(main_logic);

    new_stream.extend(TokenStream::from(quote!(let this = Self)));

    let mut new_returnvalue_stream = TokenStream::new();

    i = 0;
    for component in &components_used {
        let ident = Ident::new(&format!("comp{}", i), component.name.span());
        let component_name = component.name.clone();

        new_returnvalue_stream.extend(Some(TokenTree::Ident(ident)));
        new_returnvalue_stream.extend(TokenStream::from(quote!(:)));

        let mut component_stream = TokenStream::new();

        component_stream.extend(Some(TokenTree::Ident(component_name.clone())));

        match component.component_type {
            ComponentType::RealComponent => {
                component_stream.extend(TokenStream::from(quote!(::new)));

                let mut component_new_stream = TokenStream::new();
        
                // The following would allow not importing ComponentAttributes, but rust doesn't support it outside of nightly just yet
                // component_new_stream.extend(Some(TokenTree::Punct(Punct::new('<', proc_macro::Spacing::Alone))));
                // component_new_stream.extend(Some(TokenTree::Ident(component_name.clone())));
                // component_new_stream.extend(TokenStream::from(quote!(as Component>::ComponentAttrs)));
        
                component_new_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("{}Attributes", component_name), component_name.span()))));

                let (_reactive_variables, subcomponent_stream) = replace_variables(component.contents.clone());
                let components_attributes_group = Group::new(proc_macro::Delimiter::Brace, subcomponent_stream);

                component_new_stream.extend(Some(TokenTree::Group(components_attributes_group)));

                component_new_stream.extend(TokenStream::from(quote!(, cselfref.clone())));
        
                let component_new_group = Group::new(proc_macro::Delimiter::Parenthesis, component_new_stream);
        
                component_stream.extend(Some(TokenTree::Group(component_new_group)));
        
                new_returnvalue_stream.extend(wrap_in_arcmutex_cyclic(component_stream));
            },
            ComponentType::Node => {
                let (_reactive_variables, subcomponent_stream) = replace_variables(component.contents.clone());
                let node_group = Group::new(proc_macro::Delimiter::Brace, subcomponent_stream);
                component_stream.extend(Some(TokenTree::Group(node_group)));
                new_returnvalue_stream.extend(wrap_in_arcrwlock(component_stream));
            }
        }
        new_returnvalue_stream.extend(TokenStream::from(quote!(,)));

        i+=1;
    }

    new_returnvalue_stream.extend(TokenStream::from(quote!(attrs, selfref,)));

    for variable in &reactive_variables {
        new_returnvalue_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        new_returnvalue_stream.extend(TokenStream::from(quote!(,)));
    }

    let new_returnvalue_group = Group::new(proc_macro::Delimiter::Brace, new_returnvalue_stream);
    new_stream.extend(Some(TokenTree::Group(new_returnvalue_group)));

    new_stream.extend(TokenStream::from(quote!(;)));

    i = 0;
    for component in &components_used {
        match component.component_type {
            ComponentType::RealComponent => continue,
            ComponentType::Node => {}
        };
        for event_listener in &component.event_listeners {
            new_stream.extend(TokenStream::from(quote!(let selfref = this.selfref.clone();)));
            new_stream.extend(TokenStream::from(quote!(this.)));
            new_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("comp{}", i), component.name.span()))));

            // Change the following line according to how realcomponents want it - this is for nodes only
            new_stream.extend(TokenStream::from(quote!(.write().unwrap().events.add_handler)));

            let mut box_stream = TokenStream::new();

            box_stream.extend(TokenStream::from(quote!(Box::new)));

            let mut callback_stream = TokenStream::new();

            callback_stream.extend(TokenStream::from(quote!(move |)));
            callback_stream.extend(Some(TokenTree::Ident(event_listener.identifier.clone())));
            callback_stream.extend(TokenStream::from(quote!(|)));

            let mut inner_callback_stream = TokenStream::new();

            inner_callback_stream.extend(TokenStream::from(quote!(
                let selfref = selfref.upgrade().unwrap();
                let mut this = selfref.lock().unwrap();
                let attrs = &this.attrs;
            )));

            for variable in &reactive_variables {
                inner_callback_stream.extend(TokenStream::from(quote!(let)));
                inner_callback_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
                inner_callback_stream.extend(TokenStream::from(quote!(= &this.)));
                inner_callback_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
                inner_callback_stream.extend(TokenStream::from(quote!(;)));
            }

            inner_callback_stream.extend(replace_variables(event_listener.callback.clone().stream()).1);

            inner_callback_stream.extend(TokenStream::from(quote!(this.tick(None);)));

            let callback_group = Group::new(proc_macro::Delimiter::Brace, inner_callback_stream);

            callback_stream.extend(Some(TokenTree::Group(callback_group)));

            let callback_group = Group::new(proc_macro::Delimiter::Parenthesis, callback_stream);

            box_stream.extend(Some(TokenTree::Group(callback_group)));

            let box_group = Group::new(proc_macro::Delimiter::Parenthesis, box_stream);

            new_stream.extend(Some(TokenTree::Group(box_group)));

            new_stream.extend(TokenStream::from(quote!(;)));
        }
        i += 1;
    }

    new_stream.extend(TokenStream::from(quote!(this)));

    let new_group = Group::new(proc_macro::Delimiter::Brace, new_stream);
    component_impl_stream.extend(Some(TokenTree::Group(new_group)));

    // fn set

    component_impl_stream.extend(TokenStream::from(quote!(fn set(&mut self, attrs: Self::PartialComponentAttrs))));
    let mut set_stream = TokenStream::new();

    if attributes.len() > 0 {
        set_stream.extend(TokenStream::from(quote!(let mut to_update = [0; Self::UPDATE_LENGTH];)));
        let mut i = 0;
        for attribute in &attributes {
            set_stream.extend(TokenStream::from(quote!(if let Some)));

            let mut some_inner = TokenStream::new();
            some_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));

            set_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, some_inner))));

            set_stream.extend(TokenStream::from(quote!(= attrs.)));
            set_stream.extend(Some(TokenTree::Ident(attribute.name.clone())));

            let mut set_stream_inner = TokenStream::new();

            set_stream_inner.extend(TokenStream::from(quote!(self.attrs.)));
            set_stream_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));
            set_stream_inner.extend(TokenStream::from(quote!(=)));
            set_stream_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));
            set_stream_inner.extend(TokenStream::from(quote!(; to_update)));

            let mut to_update_stream = TokenStream::new();

            to_update_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(i / 32))));

            set_stream_inner.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Bracket, to_update_stream))));

            set_stream_inner.extend(TokenStream::from(quote!(|= )));
            set_stream_inner.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(1 << i % 32))));

            set_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, set_stream_inner))));

            i+=1;
        }

        set_stream.extend(TokenStream::from(quote!(if to_update.into_iter().reduce(|a,b| a+b).unwrap() != 0 { self.tick(Some(&to_update)); })));
    }


    component_impl_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, set_stream))));

    // fn get

    component_impl_stream.extend(TokenStream::from(quote!(fn get(&self) -> &Self::ComponentAttrs { &self.attrs })));

    // fn mount

    component_impl_stream.extend(TokenStream::from(quote!(fn mount(&self, parent: &mangui::SharedNode, before: Option<&mangui::SharedNode>))));

    let mut mount_stream = TokenStream::new();

    for i in 0..components_used.len() {
        let component = &components_used.get(i).unwrap();
        let ident = Ident::new(&format!("comp{}", i), component.name.span());

        match component.component_type {
            ComponentType::RealComponent => {
                // mount
                mount_stream.extend(TokenStream::from(quote!(self.)));
                mount_stream.extend(Some(TokenTree::Ident(ident)));
                mount_stream.extend(TokenStream::from(quote!(.lock().unwrap().mount)));

                let mut component_mount_stream = TokenStream::new();

                match component.parent {
                    Some(parent) => {
                        let parent_ident = Ident::new(&format!("comp{}", parent), Span::call_site());

                        let mut node_insert_self_stream = TokenStream::new();

                        node_insert_self_stream.extend(TokenStream::from(quote!(self.)));
                        node_insert_self_stream.extend(Some(TokenTree::Ident(parent_ident)));
                        node_insert_self_stream.extend(TokenStream::from(quote!(.clone())));

                        let node_insert_self_group = Group::new(proc_macro::Delimiter::Brace, node_insert_self_stream);
                        component_mount_stream.extend(TokenStream::from(quote!(&)));
                        component_mount_stream.extend(Some(TokenTree::Group(node_insert_self_group)));
                    },
                    None => {
                        component_mount_stream.extend(TokenStream::from(quote!(parent)));
                    }
                }
                component_mount_stream.extend(TokenStream::from(quote!(,)));
                component_mount_stream.extend(TokenStream::from(quote!(before)));
        
                let component_mount_group = Group::new(proc_macro::Delimiter::Parenthesis, component_mount_stream);
        
                mount_stream.extend(Some(TokenTree::Group(component_mount_group)));

                mount_stream.extend(TokenStream::from(quote!(;)));
            },
            ComponentType::Node => {
                let mut node_stream = TokenStream::new();

                node_stream.extend(TokenStream::from(quote!(rusalka::nodes::insert)));

                let mut node_insert_stream = TokenStream::new();

                node_insert_stream.extend(TokenStream::from(quote!(parent,)));
                node_insert_stream.extend(TokenStream::from(quote!(&)));

                let mut node_insert_self_stream = TokenStream::new();

                node_insert_self_stream.extend(TokenStream::from(quote!(self.)));
                node_insert_self_stream.extend(Some(TokenTree::Ident(ident)));
                node_insert_self_stream.extend(TokenStream::from(quote!(.clone())));

                let node_insert_self_group = Group::new(proc_macro::Delimiter::Brace, node_insert_self_stream);
                node_insert_stream.extend(Some(TokenTree::Group(node_insert_self_group)));

                node_insert_stream.extend(TokenStream::from(quote!(,)));
                node_insert_stream.extend(TokenStream::from(quote!(before)));

                node_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, node_insert_stream))));

                mount_stream.extend(node_stream);
                mount_stream.extend(TokenStream::from(quote!(;)));
            }
        }

    }

    let mount_group = Group::new(proc_macro::Delimiter::Brace, mount_stream);
    component_impl_stream.extend(Some(TokenTree::Group(mount_group)));

    // fn unmount

    component_impl_stream.extend(TokenStream::from(quote!(fn unmount(&self))));

    let mut unmount_stream = TokenStream::new();

    for i in 0..components_used.len() {
        let component = &components_used.get(i).unwrap();
        let ident = Ident::new(&format!("comp{}", i), component.name.span());

        match component.component_type {
            ComponentType::RealComponent => {
                let mut component_stream = TokenStream::new();
        
                component_stream.extend(TokenStream::from(quote!(self.)));
                component_stream.extend(Some(TokenTree::Ident(ident)));
                component_stream.extend(TokenStream::from(quote!(.lock().unwrap().unmount())));
        
                unmount_stream.extend(component_stream);
                unmount_stream.extend(TokenStream::from(quote!(;)));
            },
            ComponentType::Node => {
                unmount_stream.extend(TokenStream::from(quote!(rusalka::nodes::detach)));

                let mut node_detach_stream = TokenStream::new();
                node_detach_stream.extend(TokenStream::from(quote!(&)));

                let mut node_detach_self_stream = TokenStream::new();

                node_detach_self_stream.extend(TokenStream::from(quote!(self.)));
                node_detach_self_stream.extend(Some(TokenTree::Ident(ident)));
                node_detach_self_stream.extend(TokenStream::from(quote!(.clone())));

                let node_detach_self_group = Group::new(proc_macro::Delimiter::Brace, node_detach_self_stream);
                node_detach_stream.extend(Some(TokenTree::Group(node_detach_self_group)));

                let node_detach_group = Group::new(proc_macro::Delimiter::Parenthesis, node_detach_stream);
                unmount_stream.extend(Some(TokenTree::Group(node_detach_group)));
                unmount_stream.extend(TokenStream::from(quote!(;)));
            }
        }
    }

    let unmount_group = Group::new(proc_macro::Delimiter::Brace, unmount_stream);
    component_impl_stream.extend(Some(TokenTree::Group(unmount_group)));

    // fn update

    let mut all_variables = Vec::new();
    all_variables.extend(attributes.clone());
    all_variables.extend(reactive_variables.clone());

    component_impl_stream.extend(TokenStream::from(quote!(fn update(&self, bitmap: &[u32]))));
    let mut update_stream = TokenStream::new();
    update_stream.extend(TokenStream::from(quote!(
        self.check_update(bitmap);
        let attrs = &self.attrs;
    )));

    for variable in &reactive_variables {
        update_stream.extend(TokenStream::from(quote!(let)));
        update_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        update_stream.extend(TokenStream::from(quote!(=)));
        update_stream.extend(TokenStream::from(quote!(&self.)));
        update_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        update_stream.extend(TokenStream::from(quote!(;)));
    }

    'block: for block in &reactive_blocks {
        let mut keys: Vec<u32> = vec![0; all_variables.len() / 32 + 1];
        for variable in &block.variables {
            let index = all_variables.iter().position(|x| x.name.to_string() == variable.to_string());
            if let None = index {
                eprintln!("Warning: variable {} not found in component {}", variable, str_name);
                continue 'block;
            }
            let index = index.unwrap();
            let array_offset = index / 32;
            let num_offset = index % 32;
            *keys.get_mut(array_offset).unwrap() |= 1 << num_offset;
        }
        update_stream.extend(TokenStream::from(quote!(if)));

        let mut i = 0;
        for key in keys {
            if i > 0 {
                update_stream.extend(TokenStream::from(quote!(||)));
            }
            update_stream.extend(TokenStream::from(quote!(bitmap)));
            let mut ifgroup_stream = TokenStream::new();
            // update_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(key))));
            ifgroup_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(i))));
            update_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Bracket, ifgroup_stream))));
            update_stream.extend(TokenStream::from(quote!(&)));
            update_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(key))));
            update_stream.extend(TokenStream::from(quote!(!= 0)));
            i += 1;
        }

        let inner_group = Group::new(proc_macro::Delimiter::Brace, block.contents.clone());
        update_stream.extend(Some(TokenTree::Group(inner_group)));
    }

    let mut component_index = 0;
    'block: for component in &components_used {
        for (prop, block) in &component.reactive_props {
            let mut keys: Vec<u32> = vec![0; all_variables.len() / 32 + 1];
            for variable in &block.variables {
                let index = all_variables.iter().position(|x| x.name.to_string() == variable.to_string());
                if let None = index {
                    eprintln!("Warning: variable {} not found in component {}", variable, str_name);
                    continue 'block;
                }
                let index = index.unwrap();
                let array_offset = index / 32;
                let num_offset = index % 32;
                *keys.get_mut(array_offset).unwrap() |= 1 << num_offset;
            }
            update_stream.extend(TokenStream::from(quote!(if)));
    
            let mut i = 0;
            for key in keys {
                if i > 0 {
                    update_stream.extend(TokenStream::from(quote!(||)));
                }
                update_stream.extend(TokenStream::from(quote!(bitmap)));
                let mut ifgroup_stream = TokenStream::new();
                // update_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(key))));
                ifgroup_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(i))));
                update_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Bracket, ifgroup_stream))));
                update_stream.extend(TokenStream::from(quote!(&)));
                update_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(key))));
                update_stream.extend(TokenStream::from(quote!(!= 0)));
                i += 1;
            }

            let mut inner_stream = TokenStream::new();

            match component.component_type {
                ComponentType::Node => {
                    inner_stream.extend(TokenStream::from(quote!(self.)));
                    inner_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("comp{}", component_index), Span::call_site()))));
                    inner_stream.extend(TokenStream::from(quote!(.write().unwrap().)));
                    inner_stream.extend(Some(TokenTree::Ident(block.prop_ident.clone().unwrap())));
                    inner_stream.extend(TokenStream::from(quote!( = )));
                    inner_stream.extend(replace_variables(block.contents.clone()).1);
                },
                ComponentType::RealComponent => {
                    inner_stream.extend(TokenStream::from(quote!(self.)));
                    inner_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("comp{}", component_index), Span::call_site()))));
                    inner_stream.extend(TokenStream::from(quote!(.lock().unwrap().set)));

                    let mut component_set_stream = TokenStream::new();

                    component_set_stream.extend(Some(TokenTree::Ident(block.prop_ident.clone().unwrap())));
                    component_set_stream.extend(TokenStream::from(quote!(: Option::Some)));

                    let component_set_some_stream = replace_variables(block.contents.clone()).1;

                    let mut component_set_group = Group::new(proc_macro::Delimiter::Parenthesis, component_set_some_stream);
                    component_set_stream.extend(Some(TokenTree::Group(component_set_group)));

                    component_set_stream.extend(TokenStream::from(quote!(, ..Default::default())));

                    let component_set_group = Group::new(proc_macro::Delimiter::Brace, component_set_stream);
                    let mut component_set_outer_stream = TokenStream::new();

                    let name = component.name.clone().to_string();

                    component_set_outer_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("Partial{}Attributes", name), component.name.span()))));

                    component_set_outer_stream.extend(Some(TokenTree::Group(component_set_group)));
                    let component_set_outer_group = Group::new(proc_macro::Delimiter::Parenthesis, component_set_outer_stream);

                    inner_stream.extend(Some(TokenTree::Group(component_set_outer_group)));
                }
            }

            let inner_group = Group::new(proc_macro::Delimiter::Brace, inner_stream);
            update_stream.extend(Some(TokenTree::Group(inner_group)));
        }
        component_index += 1;
    }

    let update_group = Group::new(proc_macro::Delimiter::Brace, update_stream);
    component_impl_stream.extend(Some(TokenTree::Group(update_group)));


    // fn tick

    component_impl_stream.extend(TokenStream::from(quote!(fn tick(&mut self, inbitmap: Option<&[u32]>))));

    let mut tick_stream = TokenStream::new();

    tick_stream.extend(TokenStream::from(quote!(
        let mut bitmap = [0; Self::UPDATE_LENGTH];
        if let Some(inbitmap) = inbitmap {
            bitmap.clone_from_slice(inbitmap);
        }
        self.check_update(&bitmap);
    )));

    let mut i = attributes.len() as u32;
    for variable in &reactive_variables {
        tick_stream.extend(TokenStream::from(quote!(if self.)));
        tick_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        tick_stream.extend(TokenStream::from(quote!(.lock().unwrap().invalidated())));

        let mut if_stream = TokenStream::new();

        let array_offset = i / 32;
        let num_offset = i % 32;

        if_stream.extend(TokenStream::from(quote!(bitmap)));
        let mut ifgroup_stream = TokenStream::new();
        ifgroup_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(array_offset))));
        if_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Bracket, ifgroup_stream))));
        if_stream.extend(TokenStream::from(quote!(|= )));
        if_stream.extend(Some(TokenTree::Literal(Literal::u32_unsuffixed(1 << num_offset))));

        tick_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, if_stream))));
        i += 1;
    }

    tick_stream.extend(TokenStream::from(quote!(
        if bitmap.into_iter().reduce(|a, b| a + b).unwrap() != 0 {
            self.update(&bitmap);
        }
    )));

    let tick_group = Group::new(proc_macro::Delimiter::Brace, tick_stream);
    component_impl_stream.extend(Some(TokenTree::Group(tick_group)));

    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, component_impl_stream))));


    // dbg!(&output);

    dbg!(attributes);

    dbg!(reactive_variables);

    println!("{}", output.to_string());

    println!();

    output
}

/// Replaces $variable with **variable.lock().unwrap()
/// Returns the found variables as well as tokenstream with replaced variables
/// Returned vec is sorted and deduplicated
fn replace_variables(stream: TokenStream) -> (Vec<Ident>, TokenStream) {
    let mut output = TokenStream::new();

    let mut stream = stream.into_iter();
    let mut idents = Vec::new();

    while let Some(token) = stream.next() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == '$' => {
                let ident = stream.next().unwrap();
                let ident = match ident {
                    TokenTree::Ident(ident) => ident,
                    _ => panic!("Expected ident after $")
                };
                idents.push(ident.clone());
                output.extend(TokenStream::from(quote!(**)));
                output.extend(Some(TokenTree::Ident(ident.clone())));
                output.extend(TokenStream::from(quote!(.lock().unwrap())));
            },
            TokenTree::Group(group) => {
                let group_delim = group.delimiter();
                let span = group.span();
                let groupstream = replace_variables(group.stream());
                let mut group = Group::new(group_delim, groupstream.1);
                idents.extend(groupstream.0);
                group.set_span(span);
                output.extend(Some(TokenTree::Group(group)));
            },
            _ => {
                output.extend(Some(token));
            }
        }
    }

    idents.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    idents.dedup_by(|a, b| a.to_string() == b.to_string());

    (idents, output)
}

fn wrap_in_arc_mutex(stream: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(TokenStream::from(quote!(std::sync::Arc::new)));

    let mutex_group = Group::new(proc_macro::Delimiter::Parenthesis, stream);

    let mut mutex_stream = TokenStream::new();

    mutex_stream.extend(TokenStream::from(quote!(std::sync::Mutex::new)));

    mutex_stream.extend(Some(TokenTree::Group(mutex_group)));

    let arc_group = Group::new(proc_macro::Delimiter::Parenthesis, mutex_stream);

    output.extend(Some(TokenTree::Group(arc_group)));

    output
}

fn wrap_in_arcmutex_cyclic(stream: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(TokenStream::from(quote!(std::sync::Arc::new_cyclic)));

    let mutex_group = Group::new(proc_macro::Delimiter::Parenthesis, stream);

    let mut mutex_stream = TokenStream::new();

    mutex_stream.extend(TokenStream::from(quote!(|cselfref|)));

    mutex_stream.extend(TokenStream::from(quote!(std::sync::Mutex::new)));

    mutex_stream.extend(Some(TokenTree::Group(mutex_group)));

    let arc_group = Group::new(proc_macro::Delimiter::Parenthesis, mutex_stream);

    output.extend(Some(TokenTree::Group(arc_group)));

    output
}

fn wrap_in_arcrwlock(stream: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(TokenStream::from(quote!(std::sync::Arc::new)));

    let mutex_group = Group::new(proc_macro::Delimiter::Parenthesis, stream);

    let mut rwlock_stream = TokenStream::new();

    rwlock_stream.extend(TokenStream::from(quote!(std::sync::RwLock::new)));

    rwlock_stream.extend(Some(TokenTree::Group(mutex_group)));

    let arc_group = Group::new(proc_macro::Delimiter::Parenthesis, rwlock_stream);

    output.extend(Some(TokenTree::Group(arc_group)));

    output
}

/// Call this after @
/// Will return the main component as well as any sub-components
/// name: the name of the component
/// group: the group of tokens that make up the component
/// next: the index of this component in the components_used vector
/// parent: the index of the parent component in the components_used vector
fn parse_components(name: Ident, group: Group, next: usize, parent: Option<usize>) -> Vec<ComponentUsed> {
    let mut components_found = Vec::new();

    let mut group = group.stream().into_iter();
    let mut self_stream = TokenStream::new();

    // dbg!(&name);

    let name_starts_lowercase = name.to_string().chars().next().unwrap().is_lowercase();

    let this_component = ComponentUsed {
        name: if name_starts_lowercase {
            let mut str = name.to_string();
            str.replace_range(0..1, &str[0..1].to_uppercase());
            let span = name.span();
            Ident::new(&str, span)
        } else { name },
        contents: self_stream.clone(),
        parent,
        component_type: if name_starts_lowercase { ComponentType::Node } else { ComponentType::RealComponent },
        event_listeners: Vec::new(),
        reactive_props: HashMap::new()
    };

    components_found.push(this_component);

    while let Some(token) = group.next() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == '@' => {
                let ident = group.next().unwrap();
                let ident = match ident {
                    TokenTree::Ident(ident) => ident,
                    _ => panic!("Expected ident after @")
                };

                let group = group.next().unwrap();
                let group = match group {
                    TokenTree::Group(group) => group,
                    _ => panic!("Expected group after ident")
                };

                let components = parse_components(ident, group, next + components_found.len() + 1, Some(next));
                components_found.extend(components);
            },
            TokenTree::Punct(punct) if punct.as_char() == '$' => {
                // event handler
                let fn_start = group.next().unwrap();
                match fn_start {
                    TokenTree::Punct(punct) if punct.as_char() == '|' => {},
                    _ => panic!("Expected | after $ (event handlers). Move is added automatically. If you want to use a reactive variable, use variable: $variable instead")
                }
                let fn_param = group.next().unwrap();
                let fn_param = match fn_param {
                    TokenTree::Ident(ident) => ident,
                    _ => panic!("Expected ident after |")
                };
                let fn_end = group.next().unwrap();
                match fn_end {
                    TokenTree::Punct(punct) if punct.as_char() == '|' => {},
                    _ => panic!("Expected | after fn param")
                }

                let fn_group = group.next().unwrap();
                let fn_group = match fn_group {
                    TokenTree::Group(group) => group,
                    _ => panic!("Expected group after |param|")
                };

                let this_component = components_found.get_mut(0).unwrap();
                this_component.event_listeners.push(EventListener {
                    callback: fn_group,
                    identifier: fn_param
                });
            },
            TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();
                self_stream.extend(Some(TokenTree::Ident(ident.clone())));
                let nexttoken = group.next();
                match nexttoken {
                    None => {},
                    Some(token) => match token {
                        TokenTree::Punct(punct) if punct.as_char() == ':' => {
                            self_stream.extend(Some(TokenTree::Punct(punct)));
                            // likely reactive property
                            let mut property_stream = TokenStream::new();
                            while let Some(token) = group.next() {
                                match token {
                                    TokenTree::Punct(punct) => {
                                        let char = punct.as_char();
                                        property_stream.extend(Some(TokenTree::Punct(punct)));
                                        if char == ',' {
                                            break;
                                        }
                                    },
                                    _ => {
                                        property_stream.extend(Some(token));
                                    }
                                }
                            }
                            let (reactive_variables, property_stream) = replace_variables(property_stream);
                            if reactive_variables.len() > 0 {
                                let this_component = components_found.get_mut(0).unwrap();
                                this_component.reactive_props.insert(ident_str, ReactiveBlock {
                                    variables: reactive_variables,
                                    contents: property_stream.clone(),
                                    prop_ident: Some(ident)
                                });
                            }
                            self_stream.extend(property_stream);
                        },
                        _ => {
                            self_stream.extend(Some(token));
                        }
                    }
                }
            },
            any => {
                // skip until next ',', writing to self_stream
                self_stream.extend(Some(any));
                while let Some(token) = group.next() {
                    match token {
                        TokenTree::Punct(punct) => {
                            let char = punct.as_char();
                            self_stream.extend(Some(TokenTree::Punct(punct)));
                            if char == ',' {
                                break;
                            }
                        },
                        _ => {
                            self_stream.extend(Some(token));
                        }
                    }
                }
            }
        }
    }

    components_found.get_mut(0).unwrap().contents = self_stream;

    components_found
}