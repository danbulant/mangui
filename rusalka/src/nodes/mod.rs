use std::sync::Arc;
use mangui::SharedNode;

pub fn detach(node: &SharedNode) {
    if let Some(parent) = node.read().unwrap().parent() {
        parent.write().unwrap().remove_child(node).unwrap();
    }
    node.clone().write().unwrap().set_parent(None);
}

pub fn insert(parent: &SharedNode, node: &SharedNode, before: Option<&SharedNode>) {
    if node.read().unwrap().parent().is_some() && !Arc::ptr_eq(&node.read().unwrap().parent().unwrap(), parent) {
        detach(node);
    }
    match before {
        Some(before) => {
            parent.write().unwrap().add_child_before(node.clone(), before).unwrap();
            node.write().unwrap().set_parent(Some(Arc::downgrade(parent)));
        },
        None => {
            append(parent, node);
        }
    }
}

pub fn append(parent: &SharedNode, node: &SharedNode) {
    parent.write().unwrap().add_child(node.clone()).unwrap();
    node.write().unwrap().set_parent(Some(Arc::downgrade(parent)));
}