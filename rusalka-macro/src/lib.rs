use proc_macro::{TokenStream, TokenTree, Ident, Group, Punct, Span};
use quote::quote;

#[derive(Debug)]
struct Attribute {
    name: Ident,
    default: Option<TokenStream>,
    type_: TokenStream
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
    component_type: ComponentType
}

#[proc_macro]
/// If you have syntax errors because of attributes, wrap the default value in parentheses.
pub fn make_component(item: TokenStream) -> TokenStream {
    dbg!(&item);

    let mut last_identifier = None;
    let mut item = item.into_iter();
    let name = item.next().unwrap();
    item.next().unwrap();
    let str_name = name.to_string();

    dbg!(&name);

    let mut attributes: Vec<Attribute> = Vec::new();

    // let mut struct_values = Vec::new();

    let mut main_logic = Vec::new();

    // let mut reactive_variables = Vec::new();

    let mut components_used: Vec<ComponentUsed> = Vec::new();

    for token in item {
        match token {
            TokenTree::Ident(ident) => {
                last_identifier = Some(ident.to_string());
                let ident = ident.to_string();

                match ident.as_str() {
                    "Logic" | "Component" | "Attributes" => {},
                    _ => panic!("Unknown identifier: {:?}", ident)
                }
            },
            TokenTree::Group(group) => {
                match &last_identifier {
                    Some(ident) => {
                        match ident.as_str() {
                            "Attributes" => {
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
                                    let colon = match colon {
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

                                        attributes.push(Attribute {
                                            name,
                                            default: Some(default),
                                            type_
                                        });
                                    } else {
                                        attributes.push(Attribute {
                                            name,
                                            default: None,
                                            type_
                                        });
                                    }
                                }
                            },
                            "Logic" => {
                                main_logic.push(group.stream());
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

    dbg!(&components_used);

    let mut output = TokenStream::new();

    // Component struct

    output.extend(TokenStream::from(quote!(pub struct)));
    output.extend(Some(name.clone()));

    let mut component_struct_stream = TokenStream::new();

    let mut i = 0;
    for component in &components_used {

        let ident = Ident::new(&format!("comp{}", i), Span::call_site());

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

    // Attributes struct

    let attributes_ident = Ident::new(&format!("{str_name}Attributes"), Span::call_site());

    component_struct_stream.extend(TokenStream::from(quote!(attrs: )));
    component_struct_stream.extend(Some(TokenTree::Ident(attributes_ident.clone())));

    let component_struct_group = Group::new(proc_macro::Delimiter::Brace, component_struct_stream);
    output.extend(Some(TokenTree::Group(component_struct_group)));


    // attributes TBD

    let attributes_struct_stream = TokenStream::new();
    output.extend(TokenStream::from(quote!(#[derive(Default)] pub struct)));
    output.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, attributes_struct_stream))));

    // partial attributes

    let partial_attributes_ident = Ident::new(&format!("Partial{str_name}Attributes"), Span::call_site());
    output.extend(TokenStream::from(quote!(#[derive(Default)] pub struct)));
    output.extend(Some(TokenTree::Ident(partial_attributes_ident.clone())));
    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, TokenStream::new()))));

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

    let mut from_fn_stream = TokenStream::new();
    from_fn_stream.extend(TokenStream::from(quote!(Self {})));

    from_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, from_fn_stream))));
    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, from_stream))));

    // Component impl

    output.extend(TokenStream::from(quote!(impl rusalka::component::Component for)));
    output.extend(Some(name.clone()));

    let mut component_impl_stream = TokenStream::new();

    component_impl_stream.extend(TokenStream::from(quote!(type ComponentAttrs =)));
    component_impl_stream.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    component_impl_stream.extend(TokenStream::from(quote!(;)));
    component_impl_stream.extend(TokenStream::from(quote!(type PartialComponentAttrs =)));
    component_impl_stream.extend(Some(TokenTree::Ident(partial_attributes_ident.clone())));
    component_impl_stream.extend(TokenStream::from(quote!(;)));

    // fn new

    component_impl_stream.extend(TokenStream::from(quote!(fn new(attrs: Self::ComponentAttrs) -> Self)));

    let mut new_stream = TokenStream::new();

    new_stream.extend(main_logic);

    new_stream.extend(TokenStream::from(quote!(Self)));

    let mut new_returnvalue_stream = TokenStream::new();

    i = 0;
    for component in &components_used {
        let ident = Ident::new(&format!("comp{}", i), Span::call_site());
        let component_name = component.name.clone();
        dbg!(&component_name);

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
        
                component_new_stream.extend(Some(TokenTree::Ident(Ident::new(&format!("{}Attributes", component_name), Span::call_site()))));
        
                let components_attributes_group = Group::new(proc_macro::Delimiter::Brace, component.contents.clone());
        
                component_new_stream.extend(Some(TokenTree::Group(components_attributes_group)));
        
                let component_new_group = Group::new(proc_macro::Delimiter::Parenthesis, component_new_stream);
        
                component_stream.extend(Some(TokenTree::Group(component_new_group)));
        
                new_returnvalue_stream.extend(wrap_in_arcmutex(component_stream));
            },
            ComponentType::Node => {
                let node_group = Group::new(proc_macro::Delimiter::Brace, component.contents.clone());
                component_stream.extend(Some(TokenTree::Group(node_group)));
                new_returnvalue_stream.extend(wrap_in_arcrwlock(component_stream));
            }
        }
        new_returnvalue_stream.extend(TokenStream::from(quote!(,)));

        i+=1;
    }

    new_returnvalue_stream.extend(TokenStream::from(quote!(attrs)));

    let new_returnvalue_group = Group::new(proc_macro::Delimiter::Brace, new_returnvalue_stream);
    new_stream.extend(Some(TokenTree::Group(new_returnvalue_group)));

    let new_group = Group::new(proc_macro::Delimiter::Brace, new_stream);
    component_impl_stream.extend(Some(TokenTree::Group(new_group)));

    // fn set

    component_impl_stream.extend(TokenStream::from(quote!(fn set(&mut self, attrs: Self::PartialComponentAttrs))));
    let set_stream = TokenStream::new();

    component_impl_stream.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, set_stream))));

    // fn get

    component_impl_stream.extend(TokenStream::from(quote!(fn get(&self) -> &Self::ComponentAttrs { &self.attrs })));

    // fn mount

    component_impl_stream.extend(TokenStream::from(quote!(fn mount(&self, parent: &mangui::SharedNode, before: Option<&mangui::SharedNode>))));

    let mut mount_stream = TokenStream::new();

    for i in 0..components_used.len() {
        let ident = Ident::new(&format!("comp{}", i), Span::call_site());
        let component = &components_used.get(i).unwrap();

        match component.component_type {
            ComponentType::RealComponent => {
                let mut component_stream = TokenStream::new();
        
                component_stream.extend(TokenStream::from(quote!(self.)));
                component_stream.extend(Some(TokenTree::Ident(ident)));
                component_stream.extend(TokenStream::from(quote!(.lock().unwrap().mount)));
        
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
        
                component_stream.extend(Some(TokenTree::Group(component_mount_group)));
        
                mount_stream.extend(component_stream);
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
        let ident = Ident::new(&format!("comp{}", i), Span::call_site());
        let component = &components_used.get(i).unwrap();

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

    component_impl_stream.extend(TokenStream::from(quote!(fn update(&self, bitmap: &[u32]) { self.check_update(bitmap); })));

    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, component_impl_stream))));

    // dbg!(&output);

    dbg!(attributes);

    println!("{}", output.to_string());

    println!();

    output
}

fn wrap_in_arcmutex(stream: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(TokenStream::from(quote!(std::sync::Arc::new)));

    let mut mutex_group = Group::new(proc_macro::Delimiter::Parenthesis, stream);

    let mut mutex_stream = TokenStream::new();

    mutex_stream.extend(TokenStream::from(quote!(std::sync::Mutex::new)));

    mutex_stream.extend(Some(TokenTree::Group(mutex_group)));

    let arc_group = Group::new(proc_macro::Delimiter::Parenthesis, mutex_stream);

    output.extend(Some(TokenTree::Group(arc_group)));

    output
}

fn wrap_in_arcrwlock(stream: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(TokenStream::from(quote!(std::sync::Arc::new)));

    let mut mutex_group = Group::new(proc_macro::Delimiter::Parenthesis, stream);

    let mut mutex_stream = TokenStream::new();

    mutex_stream.extend(TokenStream::from(quote!(std::sync::RwLock::new)));

    mutex_stream.extend(Some(TokenTree::Group(mutex_group)));

    let arc_group = Group::new(proc_macro::Delimiter::Parenthesis, mutex_stream);

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
        component_type: if name_starts_lowercase { ComponentType::Node } else { ComponentType::RealComponent }
    };


    components_found.push(this_component);

    while let Some(token) = group.next() {
        match token {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '@' {
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
                } else {
                    self_stream.extend(Some(TokenTree::Punct(punct)));
                    while let Some(token) = group.next() {
                        match token {
                            TokenTree::Punct(punct) => {
                                if punct.as_char() == ',' {
                                    break;
                                } else {
                                    self_stream.extend(Some(TokenTree::Punct(punct)));
                                }
                            },
                            _ => {
                                self_stream.extend(Some(token));
                            }
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