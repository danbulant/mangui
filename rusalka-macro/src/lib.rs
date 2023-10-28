use proc_macro::{TokenStream, TokenTree, Ident, Group, Punct, Span};
use quote::quote;

struct Attribute {
    name: Ident,
    default: Option<TokenStream>,
    type_: TokenStream
}

#[derive(Debug)]
struct ComponentUsed {
    name: Ident,
    contents: TokenStream,
    parent: Option<usize>
}

#[proc_macro]
pub fn make_component(item: TokenStream) -> TokenStream {
    dbg!(&item);

    let mut last_identifier = None;
    let mut item = item.into_iter();
    let name = item.next().unwrap();
    item.next().unwrap();
    let str_name = name.to_string();

    dbg!(&name);

    // let mut attributes: Vec<Attribute> = Vec::new();

    // let mut struct_values = Vec::new();

    let mut main_logic = Vec::new();

    // let mut reactive_variables = Vec::new();

    let mut components_used: Vec<ComponentUsed> = Vec::new();

    for token in item {
        match token {
            TokenTree::Ident(ident) => {
                last_identifier = Some(ident.to_string());
                let ident = ident.to_string();

                if ident == "Logic" {
                    // dbg!("Logic");
                } else if ident == "Component" {
                    // dbg!("Component");
                } else {
                    panic!("Unknown identifier: {:?}", ident);
                }
            },
            TokenTree::Group(group) => {
                match &last_identifier {
                    Some(ident) => {
                        match ident.as_str() {
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

        let midstream = TokenStream::from(quote!(: rusalka::SharedComponent<));

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

    let attributes_struct_stream = TokenStream::new();

    // attributes TBD

    output.extend(TokenStream::from(quote!(pub struct)));
    output.extend(Some(TokenTree::Ident(attributes_ident.clone())));
    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, attributes_struct_stream))));

    // Component impl

    output.extend(TokenStream::from(quote!(impl rusalka::component::Component for)));
    output.extend(Some(name.clone()));

    let mut component_impl_stream = TokenStream::new();

    component_impl_stream.extend(TokenStream::from(quote!(type ComponentAttrs =)));
    component_impl_stream.extend(Some(TokenTree::Ident(attributes_ident.clone())));
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
        component_stream.extend(TokenStream::from(quote!(::new)));

        let mut component_new_stream = TokenStream::new();

        let component_attributes = Ident::new(&format!("{}Attributes", component_name.to_string()), Span::call_site());

        component_new_stream.extend(Some(TokenTree::Ident(component_attributes)));

        let component_attributes_group_stream = TokenStream::new();

        let components_attributes_group = Group::new(proc_macro::Delimiter::Brace, component_attributes_group_stream);

        component_new_stream.extend(Some(TokenTree::Group(components_attributes_group)));

        let component_new_group = Group::new(proc_macro::Delimiter::Parenthesis, component_new_stream);

        component_stream.extend(Some(TokenTree::Group(component_new_group)));

        new_returnvalue_stream.extend(wrap_in_arcmutex(component_stream));
        new_returnvalue_stream.extend(TokenStream::from(quote!(,)));

        i+=1;
    }

    new_returnvalue_stream.extend(TokenStream::from(quote!(attrs)));

    let new_returnvalue_group = Group::new(proc_macro::Delimiter::Brace, new_returnvalue_stream);
    new_stream.extend(Some(TokenTree::Group(new_returnvalue_group)));

    let new_group = Group::new(proc_macro::Delimiter::Brace, new_stream);
    component_impl_stream.extend(Some(TokenTree::Group(new_group)));

    // fn set

    component_impl_stream.extend(TokenStream::from(quote!(fn set(&mut self, attrs: Self::ComponentAttrs) { self.attrs = attrs; })));

    // fn get

    component_impl_stream.extend(TokenStream::from(quote!(fn get(&self) -> &Self::ComponentAttrs { &self.attrs })));

    // fn mount

    component_impl_stream.extend(TokenStream::from(quote!(fn mount(&self, parent: &mangui::SharedNode, before: Option<&mangui::SharedNode>))));

    let mut mount_stream = TokenStream::new();

    for i in 0..components_used.len() {
        let ident = Ident::new(&format!("comp{}", i), Span::call_site());

        let mut component_stream = TokenStream::new();

        component_stream.extend(TokenStream::from(quote!(self.)));
        component_stream.extend(Some(TokenTree::Ident(ident)));
        component_stream.extend(TokenStream::from(quote!(.lock().unwrap().mount)));

        let mut component_mount_stream = TokenStream::new();

        component_mount_stream.extend(TokenStream::from(quote!(parent)));
        component_mount_stream.extend(TokenStream::from(quote!(,)));
        component_mount_stream.extend(TokenStream::from(quote!(before)));

        let component_mount_group = Group::new(proc_macro::Delimiter::Parenthesis, component_mount_stream);

        component_stream.extend(Some(TokenTree::Group(component_mount_group)));

        mount_stream.extend(component_stream);
        mount_stream.extend(TokenStream::from(quote!(;)));
    }

    let mount_group = Group::new(proc_macro::Delimiter::Brace, mount_stream);
    component_impl_stream.extend(Some(TokenTree::Group(mount_group)));

    // fn unmount

    component_impl_stream.extend(TokenStream::from(quote!(fn unmount(&self))));

    let mut unmount_stream = TokenStream::new();

    for i in 0..components_used.len() {
        let ident = Ident::new(&format!("comp{}", i), Span::call_site());

        let mut component_stream = TokenStream::new();

        component_stream.extend(TokenStream::from(quote!(self.)));
        component_stream.extend(Some(TokenTree::Ident(ident)));
        component_stream.extend(TokenStream::from(quote!(.lock().unwrap().unmount())));

        unmount_stream.extend(component_stream);
        unmount_stream.extend(TokenStream::from(quote!(;)));
    }

    let unmount_group = Group::new(proc_macro::Delimiter::Brace, unmount_stream);
    component_impl_stream.extend(Some(TokenTree::Group(unmount_group)));

    // fn update

    component_impl_stream.extend(TokenStream::from(quote!(fn update(&self) {})));

    output.extend(Some(TokenTree::Group(Group::new(proc_macro::Delimiter::Brace, component_impl_stream))));

    dbg!(&output);

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

    let this_component = ComponentUsed {
        name,
        contents: self_stream.clone(),
        parent
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
            _ => {
                // skip until next ',', writing to self_stream

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
        }
    }

    components_found.get_mut(0).unwrap().contents = self_stream;

    components_found
}