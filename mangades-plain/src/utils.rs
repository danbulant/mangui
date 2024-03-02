use std::sync::Arc;
use mangui::SharedNode;

pub fn detach(node: &SharedNode) {
    if let Some(parent) = node.lock().unwrap().parent() {
        parent.lock().unwrap().remove_child(node).unwrap();
    }
    node.clone().lock().unwrap().set_parent(None);
}

pub fn insert(parent: &SharedNode, node: &SharedNode, before: Option<&SharedNode>) {
    if node.lock().unwrap().parent().is_some() && !Arc::ptr_eq(&node.lock().unwrap().parent().unwrap(), parent) {
        detach(node);
    }
    match before {
        Some(before) => {
            parent.lock().unwrap().add_child_before(node.clone(), before).unwrap();
            node.lock().unwrap().set_parent(Some(Arc::downgrade(parent)));
        },
        None => {
            append(parent, node);
        }
    }
}

pub fn append(parent: &SharedNode, node: &SharedNode) {
    parent.lock().unwrap().add_child(node.clone()).unwrap();
    node.lock().unwrap().set_parent(Some(Arc::downgrade(parent)));
}