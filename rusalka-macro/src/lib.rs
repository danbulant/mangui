use proc_macro2::{Ident, TokenStream, TokenTree, Span, Group, Delimiter};
use std::collections::HashMap;

use quote::{format_ident, quote};

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
    SlotDefinition,
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
pub fn make_component(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);
    let mut last_identifier = None;
    let mut item = item.into_iter();
    let name = item.next().unwrap();
    item.next().unwrap();
    let name_ident = match name {
        TokenTree::Ident(ident) => ident,
        _ => panic!("Expected ident")
    };
    let str_name = name_ident.to_string();

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

                                    for token in stream.by_ref() {
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

                                        for token in stream.by_ref() {
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
                                if !contents.is_empty() && !variables.is_empty() {
                                    reactive_blocks.push(ReactiveBlock {
                                        variables,
                                        contents,
                                        prop_ident: None
                                    });
                                }
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
    output.extend(quote!(
        use rusalka::store::DerefGuardExt;
        use rusalka::store::WritableStore;
        use rusalka::store::Signal;
    ));

    output.extend(quote!(pub struct));
    output.extend(Some(TokenTree::Ident(name_ident.clone())));

    let mut component_struct_stream = TokenStream::new();

    for (i, component) in components_used.iter().enumerate() {
        let ident = Ident::new(&format!("comp{}", i), component.name.span());

        let mut midstream = quote!(: rusalka::);

        match component.component_type {
            ComponentType::RealComponent => {
                midstream.extend(quote!(SharedComponent<));
            },
            ComponentType::Node => {
                midstream.extend(quote!(SharedNodeComponent<));
            },
            ComponentType::SlotDefinition => {
                midstream.extend(quote!(SharedSlot));
            }
        }
        
        match component.component_type {
            ComponentType::RealComponent | ComponentType::Node => {
                let component_name = component.name.clone();
        
                component_struct_stream.extend(Some(TokenTree::Ident(ident)));
                component_struct_stream.extend(midstream);
                component_struct_stream.extend(Some(TokenTree::Ident(component_name)));
                component_struct_stream.extend(quote!(>,));
            },
            ComponentType::SlotDefinition => {}
        }

    }

    component_struct_stream.extend(quote!(
        sub: Vec<Box<dyn rusalka::store::StoreUnsubscribe>>,
    ));

    for variable in &reactive_variables {
        component_struct_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        component_struct_stream.extend(quote!(:));
        component_struct_stream.extend(quote!(std::sync::Arc<rusalka::store::Writable<));
        component_struct_stream.extend(variable.type_.clone());
        component_struct_stream.extend(quote!(>>));
        component_struct_stream.extend(quote!(,));
    }

    // selfref

    component_struct_stream.extend(quote!(selfref: rusalka::WeakSharedComponent<Self>,));

    // Attributes struct

    let attributes_ident = Ident::new(&format!("{str_name}Attributes"), name_ident.span());
    let partial_attributes_ident = Ident::new(&format!("Partial{str_name}Attributes"), name_ident.span());
    let reactive_attributes_ident = Ident::new(&format!("Reactive{str_name}Attributes"), name_ident.span());

    component_struct_stream.extend(quote!(attrs: ));
    component_struct_stream.extend(Some(TokenTree::Ident(reactive_attributes_ident.clone())));

    let component_struct_group = Group::new(Delimiter::Brace, component_struct_stream);
    output.extend(Some(TokenTree::Group(component_struct_group)));


    // attributes

    let mut attributes_struct_stream = TokenStream::new();
    output.extend(quote!(#[derive(Default)] pub struct));
    output.extend(Some(TokenTree::Ident(attributes_ident.clone())));

    for attribute in &attributes {
        attributes_struct_stream.extend(quote!(pub));
        attributes_struct_stream.extend(Some(TokenTree::Ident(attribute.name.clone())));
        attributes_struct_stream.extend(quote!(:));
        attributes_struct_stream.extend(attribute.type_.clone());
        attributes_struct_stream.extend(quote!(,));
    }

    output.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, attributes_struct_stream))));

    // partial attributes

    output.extend(quote!(#[derive(Default)] pub struct));
    output.extend(Some(TokenTree::Ident(partial_attributes_ident.clone())));
    let mut attributes_default_struct_stream = TokenStream::new();

    for attribute in &attributes {
        attributes_default_struct_stream.extend(quote!(pub));
        attributes_default_struct_stream.extend(Some(TokenTree::Ident(attribute.name.clone())));
        attributes_default_struct_stream.extend(quote!(: Option<));
        attributes_default_struct_stream.extend(attribute.type_.clone());
        attributes_default_struct_stream.extend(quote!(>,));
    }

    output.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, attributes_default_struct_stream))));

    // reactive attributes

    output.extend(quote!(#[derive(Default)] pub struct));
    output.extend(Some(TokenTree::Ident(reactive_attributes_ident.clone())));
    let mut attributes_default_struct_stream = TokenStream::new();

    for attribute in &attributes {
        attributes_default_struct_stream.extend(quote!(pub));
        attributes_default_struct_stream.extend(Some(TokenTree::Ident(attribute.name.clone())));
        attributes_default_struct_stream.extend(quote!(:  std::sync::Arc<rusalka::store::Writable<));
        attributes_default_struct_stream.extend(attribute.type_.clone());
        attributes_default_struct_stream.extend(quote!(>>,));
    }

    output.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, attributes_default_struct_stream))));

    // impl From<Attributes> for PartialAttributes

    output.extend(quote!(impl From<));
    output.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    output.extend(quote!(> for));
    output.extend(Some(TokenTree::Ident(partial_attributes_ident.clone())));

    let mut from_stream = TokenStream::new();

    from_stream.extend(quote!(fn from));

    let mut from_args = TokenStream::new();
    from_args.extend(quote!(attrs:));
    from_args.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    from_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Parenthesis, from_args))));
    from_stream.extend(quote!(-> Self));

    let mut from_fn_stream = quote!(Self);

    let mut from_fn_stream_inner = TokenStream::new();

    for attribute in &attributes {
        from_fn_stream_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));
        from_fn_stream_inner.extend(quote!(:));
        from_fn_stream_inner.extend(quote!(Some));

        let mut from_fn_stream_inner_inner = TokenStream::new();

        from_fn_stream_inner_inner.extend(quote!(attrs.));
        from_fn_stream_inner_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));

        from_fn_stream_inner.extend(Some(TokenTree::Group(Group::new(Delimiter::Parenthesis, from_fn_stream_inner_inner))));
    }

    from_fn_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, from_fn_stream_inner))));

    from_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, from_fn_stream))));
    output.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, from_stream))));

    // impl From<Attributes> for ReactiveAttributes

    output.extend(quote!(impl From<));
    output.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    output.extend(quote!(> for));
    output.extend(Some(TokenTree::Ident(reactive_attributes_ident.clone())));

    let mut from_stream = TokenStream::new();

    from_stream.extend(quote!(fn from));

    let mut from_args = TokenStream::new();
    from_args.extend(quote!(attrs:));
    from_args.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    from_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Parenthesis, from_args))));
    from_stream.extend(quote!(-> Self));

    let mut from_fn_stream = quote!(Self);

    let mut from_fn_stream_inner = TokenStream::new();

    for attribute in &attributes {
        let name = attribute.name.clone();
        from_fn_stream_inner.extend(quote!(
            #name : std::sync::Arc::new(rusalka::store::Writable::new(attrs.#name))
        ));
    }

    from_fn_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, from_fn_stream_inner))));

    from_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, from_fn_stream))));
    output.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, from_stream))));

    // Component impl

    output.extend(quote!(impl rusalka::component::Component for #name_ident));

    let mut component_impl_stream = TokenStream::new();

    component_impl_stream.extend(quote!(
        type ComponentAttrs = #attributes_ident;
        type PartialComponentAttrs = #partial_attributes_ident;
        type ReactiveComponentAttrs = #reactive_attributes_ident;
    ));

    // fn new

    component_impl_stream.extend(quote!(
        fn new(attrs: Self::ComponentAttrs, selfref: rusalka::WeakSharedComponent<Self>) -> Self
    ));

    let mut new_stream = TokenStream::new();

    new_stream.extend(quote!(
        let attrs: Self::ReactiveComponentAttrs = attrs.into();
    ));

    for attribute in &attributes {
        let name = attribute.name.clone();
        new_stream.extend(quote!(
            let #name = attrs.#name.clone();
        ));
    }

    for variable in &reactive_variables {
        let name = variable.name.clone();
        let type_ = variable.type_.clone();
        let mut invalidator_inner = TokenStream::new();
        if let Some(def) = &variable.default {
            invalidator_inner.extend(replace_variables(def.clone()).1);
        } else {
            invalidator_inner.extend(quote!(Default::default()));
        }
        new_stream.extend(quote!(
            let #name : std::sync::Arc<rusalka::store::Writable< #type_ >> =
            std::sync::Arc::new(rusalka::store::Writable::new( #invalidator_inner ));
        ));
    }

    new_stream.extend(main_logic);

    new_stream.extend(quote!(let this = Self));

    let mut new_returnvalue_stream = TokenStream::new();

    for (i, component) in components_used.iter().enumerate() {
        let ident = Ident::new(&format!("comp{}", i), component.name.span());
        let component_name = component.name.clone();

        new_returnvalue_stream.extend(Some(TokenTree::Ident(ident)));
        new_returnvalue_stream.extend(quote!(:));

        let mut component_stream = TokenStream::new();

        match component.component_type {
            ComponentType::RealComponent => {
                component_stream.extend(Some(TokenTree::Ident(component_name.clone())));

                component_stream.extend(quote!(::new));

                let mut component_new_stream = TokenStream::new();
        
                // The following would allow not importing ComponentAttributes, but rust doesn't support it outside of nightly just yet
                // component_new_stream.extend(Some(TokenTree::Punct(Punct::new('<', Spacing::Alone))));
                // component_new_stream.extend(Some(TokenTree::Ident(component_name.clone())));
                // component_new_stream.extend(TokenStream::from(quote!(as Component>::ComponentAttrs)));
        
                component_new_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("{}Attributes", component_name), component_name.span()))));

                let (_reactive_variables, subcomponent_stream) = replace_variables(component.contents.clone());
                let components_attributes_group = Group::new(Delimiter::Brace, subcomponent_stream);

                component_new_stream.extend(Some(TokenTree::Group(components_attributes_group)));

                component_new_stream.extend(quote!(, cselfref.clone()));
        
                let component_new_group = Group::new(Delimiter::Parenthesis, component_new_stream);
        
                component_stream.extend(Some(TokenTree::Group(component_new_group)));
        
                new_returnvalue_stream.extend(wrap_in_arcmutex_cyclic(component_stream));
            },
            ComponentType::Node => {
                component_stream.extend(Some(TokenTree::Ident(component_name.clone())));

                let (_reactive_variables, subcomponent_stream) = replace_variables(component.contents.clone());
                let node_group = Group::new(Delimiter::Brace, subcomponent_stream);
                component_stream.extend(Some(TokenTree::Group(node_group)));
                new_returnvalue_stream.extend(wrap_in_arcrwlock(component_stream));
            },
            ComponentType::SlotDefinition => {
                component_stream.extend(quote!(Arc::new(Mutex::new(None))));
            }
        }
        new_returnvalue_stream.extend(quote!(,));
    }

    new_returnvalue_stream.extend(quote!(attrs, selfref, sub: vec!));

    let mut sub_stream = TokenStream::new();
    let mut i = 0u32;
    for reactive_block in &reactive_blocks {
        let content = &reactive_block.contents;
        let variables = &reactive_block.variables;
        let vecvariables = reactive_block.variables.iter().map(|v| format_ident!("vec{}", v));
        let vecvariables2 = reactive_block.variables.iter().map(|v| format_ident!("vec{}", v));
        sub_stream.extend(quote!(
            {
                #(let #variables = #variables.clone();)*
                #(let #vecvariables = #variables.clone();)*
                let vec = [#(#vecvariables2),*];
                let res = vec.subscribe(Box::new(move || {
                    #(let #variables = #variables.clone();)*
                    #content
                }));
                drop(vec);
                res
            },
        ));
        i+=1;
    }

    new_returnvalue_stream.extend(quote!([ #sub_stream ],));

    for variable in &reactive_variables {
        new_returnvalue_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
        new_returnvalue_stream.extend(quote!(,));
    }

    let new_returnvalue_group = Group::new(Delimiter::Brace, new_returnvalue_stream);
    new_stream.extend(Some(TokenTree::Group(new_returnvalue_group)));

    new_stream.extend(quote!(;));

    i = 0;
    for component in &components_used {
        match component.component_type {
            ComponentType::RealComponent => continue,
            // currently just ignore those, in the future it should probably be an error.
            ComponentType::SlotDefinition => continue,
            ComponentType::Node => {}
        };
        for event_listener in &component.event_listeners {
            new_stream.extend(quote!(let selfref = this.selfref.clone();));
            new_stream.extend(quote!(this.));
            new_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("comp{}", i), component.name.span()))));

            // Change the following line according to how realcomponents want it - this is for nodes only
            new_stream.extend(quote!(.write().unwrap().events.add_handler));

            let mut box_stream = TokenStream::new();

            box_stream.extend(quote!(Box::new));

            let mut callback_stream = TokenStream::new();

            callback_stream.extend(quote!(move |));
            callback_stream.extend(Some(TokenTree::Ident(event_listener.identifier.clone())));
            callback_stream.extend(quote!(|));

            let mut inner_callback_stream = TokenStream::new();

            inner_callback_stream.extend(quote!(
                let selfref = selfref.upgrade().unwrap();
                let mut this = selfref.lock().unwrap();
                let attrs = &this.attrs;
            ));

            for variable in &reactive_variables {
                inner_callback_stream.extend(quote!(let));
                inner_callback_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
                inner_callback_stream.extend(quote!(= &this.));
                inner_callback_stream.extend(Some(TokenTree::Ident(variable.name.clone())));
                inner_callback_stream.extend(quote!(;));
            }

            inner_callback_stream.extend(replace_variables(event_listener.callback.clone().stream()).1);

            let callback_group = Group::new(Delimiter::Brace, inner_callback_stream);

            callback_stream.extend(Some(TokenTree::Group(callback_group)));

            let callback_group = Group::new(Delimiter::Parenthesis, callback_stream);

            box_stream.extend(Some(TokenTree::Group(callback_group)));

            let box_group = Group::new(Delimiter::Parenthesis, box_stream);

            new_stream.extend(Some(TokenTree::Group(box_group)));

            new_stream.extend(quote!(;));
        }
        i += 1;
    }

    new_stream.extend(quote!(this));

    let new_group = Group::new(Delimiter::Brace, new_stream);
    component_impl_stream.extend(Some(TokenTree::Group(new_group)));

    // fn set

    component_impl_stream.extend(quote!(fn set(&mut self, attrs: Self::PartialComponentAttrs)));
    let mut set_stream = TokenStream::new();

    if !attributes.is_empty() {
        for (_i, attribute) in attributes.iter().enumerate() {
            set_stream.extend(quote!(if let Some));

            let mut some_inner = TokenStream::new();
            some_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));

            set_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Parenthesis, some_inner))));

            set_stream.extend(quote!(= attrs.));
            set_stream.extend(Some(TokenTree::Ident(attribute.name.clone())));

            let mut set_stream_inner = TokenStream::new();

            set_stream_inner.extend(quote!(self.attrs.));
            set_stream_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));
            set_stream_inner.extend(quote!(.set));

            let mut set_stream_inner_inner = TokenStream::new();

            set_stream_inner_inner.extend(Some(TokenTree::Ident(attribute.name.clone())));

            set_stream_inner.extend(Some(TokenTree::Group(Group::new(Delimiter::Parenthesis, set_stream_inner_inner))));
            set_stream_inner.extend(quote!(;));

            set_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, set_stream_inner))));
        }
    }


    component_impl_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, set_stream))));

    // fn get

    component_impl_stream.extend(quote!(fn get(&self) -> &Self::ReactiveComponentAttrs { &self.attrs }));

    // fn mount

    component_impl_stream.extend(quote!(fn mount(&self, parent: &mangui::SharedNode, before: Option<&mangui::SharedNode>)));

    let mut mount_stream = TokenStream::new();

    for i in 0..components_used.len() {
        let component = &components_used.get(i).unwrap();
        let ident = Ident::new(&format!("comp{}", i), component.name.span());

        match component.component_type {
            ComponentType::SlotDefinition => continue,
            ComponentType::RealComponent => {
                // mount
                mount_stream.extend(quote!(self.));
                mount_stream.extend(Some(TokenTree::Ident(ident)));
                mount_stream.extend(quote!(.lock().unwrap().mount));

                let mut component_mount_stream = TokenStream::new();

                match component.parent {
                    Some(parent) => {
                        let parent_ident = Ident::new(&format!("comp{}", parent), Span::call_site());

                        let mut node_insert_self_stream = TokenStream::new();

                        node_insert_self_stream.extend(quote!(self.));
                        node_insert_self_stream.extend(Some(TokenTree::Ident(parent_ident)));
                        node_insert_self_stream.extend(quote!(.clone()));

                        let node_insert_self_group = Group::new(Delimiter::Brace, node_insert_self_stream);
                        component_mount_stream.extend(quote!(&));
                        component_mount_stream.extend(Some(TokenTree::Group(node_insert_self_group)));
                    },
                    None => {
                        component_mount_stream.extend(quote!(parent));
                    }
                }
                component_mount_stream.extend(quote!(,));
                component_mount_stream.extend(quote!(before));
        
                let component_mount_group = Group::new(Delimiter::Parenthesis, component_mount_stream);
        
                mount_stream.extend(Some(TokenTree::Group(component_mount_group)));

                mount_stream.extend(quote!(;));
            },
            ComponentType::Node => {
                let mut node_stream = TokenStream::new();

                node_stream.extend(quote!(rusalka::nodes::insert));

                let mut node_insert_stream = TokenStream::new();

                match component.parent {
                    Some(parent) => {
                        let parent_ident = Ident::new(&format!("comp{}", parent), Span::call_site());

                        let mut node_insert_self_stream = TokenStream::new();

                        node_insert_self_stream.extend(quote!(self.));
                        node_insert_self_stream.extend(Some(TokenTree::Ident(parent_ident)));
                        node_insert_self_stream.extend(quote!(.clone()));

                        let node_insert_self_group = Group::new(Delimiter::Brace, node_insert_self_stream);
                        node_insert_stream.extend(quote!(&));
                        node_insert_stream.extend(Some(TokenTree::Group(node_insert_self_group)));
                    },
                    None => {
                        node_insert_stream.extend(quote!(parent));
                    }
                }

                node_insert_stream.extend(quote!(,));
                node_insert_stream.extend(quote!(&));

                let mut node_insert_self_stream = TokenStream::new();

                node_insert_self_stream.extend(quote!(self.));
                node_insert_self_stream.extend(Some(TokenTree::Ident(ident)));
                node_insert_self_stream.extend(quote!(.clone()));

                let node_insert_self_group = Group::new(Delimiter::Brace, node_insert_self_stream);
                node_insert_stream.extend(Some(TokenTree::Group(node_insert_self_group)));

                node_insert_stream.extend(quote!(,));
                node_insert_stream.extend(quote!(before));

                node_stream.extend(Some(TokenTree::Group(Group::new(Delimiter::Parenthesis, node_insert_stream))));

                mount_stream.extend(node_stream);
                mount_stream.extend(quote!(;));
            }
        }

    }

    let mount_group = Group::new(Delimiter::Brace, mount_stream);
    component_impl_stream.extend(Some(TokenTree::Group(mount_group)));

    // fn unmount

    component_impl_stream.extend(quote!(fn unmount(&self)));

    let mut unmount_stream = TokenStream::new();

    for i in 0..components_used.len() {
        let component = &components_used.get(i).unwrap();
        let ident = Ident::new(&format!("comp{}", i), component.name.span());

        match component.component_type {
            ComponentType::SlotDefinition => continue,
            ComponentType::RealComponent => {
                let mut component_stream = TokenStream::new();
        
                component_stream.extend(quote!(self.));
                component_stream.extend(Some(TokenTree::Ident(ident)));
                component_stream.extend(quote!(.lock().unwrap().unmount()));
        
                unmount_stream.extend(component_stream);
                unmount_stream.extend(quote!(;));
            },
            ComponentType::Node => {
                unmount_stream.extend(quote!(rusalka::nodes::detach));

                let mut node_detach_stream = TokenStream::new();
                node_detach_stream.extend(quote!(&));

                let mut node_detach_self_stream = TokenStream::new();

                node_detach_self_stream.extend(quote!(self.));
                node_detach_self_stream.extend(Some(TokenTree::Ident(ident)));
                node_detach_self_stream.extend(quote!(.clone()));

                let node_detach_self_group = Group::new(Delimiter::Brace, node_detach_self_stream);
                node_detach_stream.extend(Some(TokenTree::Group(node_detach_self_group)));

                let node_detach_group = Group::new(Delimiter::Parenthesis, node_detach_stream);
                unmount_stream.extend(Some(TokenTree::Group(node_detach_group)));
                unmount_stream.extend(quote!(;));
            }
        }
    }

    let unmount_group = Group::new(Delimiter::Brace, unmount_stream);
    component_impl_stream.extend(Some(TokenTree::Group(unmount_group)));

    output.extend(Some(TokenTree::Group(Group::new(Delimiter::Brace, component_impl_stream))));

    println!("{}", output);

    println!();

    output.into()
}

/// Replaces $variable with **variable.lock().unwrap().guard()
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
                output.extend(quote!(**));
                output.extend(Some(TokenTree::Ident(ident.clone())));
                output.extend(quote!(.guard()));
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

    idents.sort_by_key(|a| a.to_string());
    idents.dedup_by(|a, b| *b == a.to_string());

    (idents, output)
}

fn wrap_in_arcmutex_cyclic(stream: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(quote!(std::sync::Arc::new_cyclic));

    let mutex_group = Group::new(Delimiter::Parenthesis, stream);

    let mut mutex_stream = TokenStream::new();

    mutex_stream.extend(quote!(|cselfref|));

    mutex_stream.extend(quote!(std::sync::Mutex::new));

    mutex_stream.extend(Some(TokenTree::Group(mutex_group)));

    let arc_group = Group::new(Delimiter::Parenthesis, mutex_stream);

    output.extend(Some(TokenTree::Group(arc_group)));

    output
}

fn wrap_in_arcrwlock(stream: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(quote!(std::sync::Arc::new));

    let mutex_group = Group::new(Delimiter::Parenthesis, stream);

    let mut rwlock_stream = TokenStream::new();

    rwlock_stream.extend(quote!(std::sync::RwLock::new));

    rwlock_stream.extend(Some(TokenTree::Group(mutex_group)));

    let arc_group = Group::new(Delimiter::Parenthesis, rwlock_stream);

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

                let components = parse_components(ident, group, next + components_found.len(), Some(next));
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
                            for token in group.by_ref() {
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
                            if !reactive_variables.is_empty() {
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
                for token in group.by_ref() {
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