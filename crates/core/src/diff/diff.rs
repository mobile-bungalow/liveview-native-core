use std::collections::VecDeque;

use fxhash::{FxHashMap, FxHashSet};

use crate::dom::*;

use super::patch::Patch;
use super::traversal::MoveTo;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Cursor {
    depth: u32,
    child: u32,
    node: NodeRef,
}
impl Cursor {
    #[inline]
    fn pos(&self) -> (u32, u32) {
        (self.depth, self.child)
    }

    #[inline(always)]
    fn node<'a>(&self, doc: &'a Document) -> &'a Node {
        doc.get(self.node)
    }
}

pub fn diff(old_document: &Document, new_document: &Document) -> VecDeque<Patch> {
    let mut patches = VecDeque::new();
    let mut old_current = Cursor {
        depth: 0,
        child: 0,
        node: old_document.root(),
    };
    let mut old_next = VecDeque::from([old_current]);
    let mut new_next = VecDeque::from([Cursor {
        depth: 0,
        child: 0,
        node: new_document.root(),
    }]);
    loop {
        match dbg!((old_next.pop_front(), new_next.pop_front())) {
            // We're at the same position in both trees, so examine the details of the node under the cursor
            (Some(old_cursor), Some(new_cursor)) if old_cursor.pos() == new_cursor.pos() => {
                old_current = old_cursor;
                match dbg!((old_cursor.node(old_document), new_cursor.node(new_document))) {
                    // This was the root node, so move the cursor down a level and start walking the children
                    (Node::Root, Node::Root) => {
                        let depth = old_cursor.depth + 1;
                        let old_children = old_document.children(old_cursor.node);
                        let new_children = new_document.children(new_cursor.node);
                        old_next.extend(old_children.iter().copied().enumerate().map(
                            |(i, node)| Cursor {
                                depth,
                                child: i as u32,
                                node,
                            },
                        ));
                        new_next.extend(new_children.iter().copied().enumerate().map(
                            |(i, node)| Cursor {
                                depth,
                                child: i as u32,
                                node,
                            },
                        ));
                        continue;
                    }
                    // Both nodes are leaf nodes, compare their content for equality
                    (Node::Leaf(ref ol), Node::Leaf(ref nl)) => {
                        // If both nodes are identical, keep moving
                        // Otherwise, emit a patch to update the text of this node
                        if ol == nl {
                            continue;
                        } else {
                            patches.push_back(Patch::Replace {
                                node: old_cursor.node,
                                replacement: Node::Leaf(nl.clone()),
                            });
                        }
                    }
                    // Both nodes are elements, compare their tag and attributes for equality, then append their children to the stack
                    (Node::Element(ref old), Node::Element(ref new)) => {
                        // If the names are different, replace the old element with the new
                        if old.namespace != new.namespace || old.tag != new.tag {
                            patches.push_back(Patch::Replace {
                                node: old_cursor.node,
                                replacement: Node::Element(Element {
                                    namespace: new.namespace,
                                    tag: new.tag,
                                    attributes: old.attributes,
                                }),
                            });
                        }
                        // Check for changes to the attributes
                        let mut attr_changes =
                            diff_attributes(old_cursor.node, old, new, old_document, new_document);
                        patches.append(&mut attr_changes);
                        // Add the children of both nodes to the worklist
                        let depth = old_cursor.depth + 1;
                        let old_children = old_document.children(old_cursor.node);
                        let new_children = new_document.children(new_cursor.node);
                        old_next.extend(old_children.iter().copied().enumerate().map(
                            |(i, node)| Cursor {
                                depth,
                                child: i as u32,
                                node,
                            },
                        ));
                        new_next.extend(new_children.iter().copied().enumerate().map(
                            |(i, node)| Cursor {
                                depth,
                                child: i as u32,
                                node,
                            },
                        ));
                    }
                    // The old node was an element and the new node is a leaf; determine if this is a simple swap, addition, or removal
                    // by looking forward in the stack to future cursors which are at the same depth.
                    (Node::Element(_), Node::Leaf(ref new)) => {
                        patches.push_back(Patch::Replace {
                            node: old_cursor.node,
                            replacement: Node::Leaf(new.clone()),
                        });
                        let depth = old_cursor.depth + 1;
                        let old_children = old_document.children(old_cursor.node);
                        old_next.extend(old_children.iter().copied().enumerate().map(
                            |(i, node)| Cursor {
                                depth,
                                child: i as u32,
                                node,
                            },
                        ));
                    }
                    // The old node was a leaf and the new node is an element; determine if this is a simple swap, addition, or removal
                    // by looking forward in the stack to future cursors which are at the same depth.
                    (Node::Leaf(_), Node::Element(ref new)) => {
                        let replacement = Element {
                            namespace: new.namespace,
                            tag: new.tag,
                            attributes: AttributeList::new(),
                        };
                        let mut attr_changes = diff_attributes(
                            old_cursor.node,
                            &replacement,
                            new,
                            old_document,
                            new_document,
                        );
                        patches.push_back(Patch::Replace {
                            node: old_cursor.node,
                            replacement: Node::Element(replacement),
                        });
                        patches.append(&mut attr_changes);
                        let depth = old_cursor.depth + 1;
                        let new_children = new_document.children(new_cursor.node);
                        new_next.extend(new_children.iter().copied().enumerate().map(
                            |(i, node)| Cursor {
                                depth,
                                child: i as u32,
                                node,
                            },
                        ));
                    }
                    _ => unreachable!(),
                }
            }
            // We descended a level in the new tree, but not the old; this means that there
            // were removals in the new tree at that depth. Issue removals until the old cursor
            // catches up with the new
            (Some(old_cursor), Some(new_cursor)) if old_cursor.depth < new_cursor.depth => {
                old_current = old_cursor;
                patches.push_back(Patch::Remove {
                    node: old_cursor.node,
                });
                while let Some(old_cursor) = old_next.front() {
                    if old_cursor.pos() == new_cursor.pos() {
                        // We've caught up to the new tree
                        break;
                    }
                    let old_cursor = old_next.pop_front().unwrap();
                    old_current = old_cursor;
                    patches.push_back(Patch::Remove {
                        node: old_cursor.node,
                    });
                }
                // We either caught up to the new cursor, or ran out of old tree to visit, either way, we need to revisit the new cursor
                new_next.push_front(new_cursor);
            }
            // We descended a level in the old tree, but not the new; this means that there
            // were additions in the new tree at that depth. Issue additions until the new cursor
            // catches up with the old
            (Some(old_cursor), Some(new_cursor)) => {
                assert!(
                    old_cursor.depth > new_cursor.depth,
                    "expected {} > {}",
                    old_cursor.depth,
                    new_cursor.depth
                );
                // We need to go back up a level on the old tree, then grab the parent of _that_ node,
                // so that we can append more children. If this returns None, it means that we're inserting
                // multiple siblings at the root
                let parent = old_document
                    .parent(old_document.parent(old_cursor.node).unwrap())
                    .unwrap();
                // Handle the node currently under the cursor
                patches.push_back(Patch::Move(MoveTo::Node(parent)));
                patches.push_back(Patch::Push(parent));
                let mut subtree = recursively_append(new_cursor.node, new_document);
                patches.append(&mut subtree);
                // Handle the rest
                while let Some(new_cursor) = new_next.front() {
                    if old_cursor.pos() == new_cursor.pos() {
                        // We've caught up to the old tree
                        break;
                    }
                    let new_cursor = new_next.pop_front().unwrap();
                    patches.push_back(Patch::Move(MoveTo::Node(parent)));
                    patches.push_back(Patch::Push(parent));
                    let mut subtree = recursively_append(new_cursor.node, new_document);
                    patches.append(&mut subtree);
                }
                // We either caught up to the old cursor, or ran out of new tree to visit, either way, we need to revisit the old cursor
                old_next.push_front(old_cursor);
            }
            // We've reached the end of the worklist for the new tree, but not the old; this means
            // that all remaining nodes in the old tree were removed, since we can't have visited them yet,
            // so issue removals for all remaining old tree nodes
            (Some(old_cursor), None) => {
                old_current = old_cursor;
                patches.push_back(Patch::Remove {
                    node: old_cursor.node,
                });
                while let Some(old_cursor) = old_next.pop_front() {
                    old_current = old_cursor;
                    patches.push_back(Patch::Remove {
                        node: old_cursor.node,
                    });
                }
            }
            // We've reached the end of the worklist for the old tree, but not the new; this means
            // that all remaining nodes in the new tree were added, since we can't have visited them yet,
            // so issue additions for all remaining new tree nodes. These additions should be appended
            // to the parent of the last node of the old tree that we had a cursor to, since that necessarily
            // will be the parent of these new nodes, as the cursors on both trees are in lock-step all the
            // way down
            (None, Some(new_cursor)) => {
                patches.push_back(Patch::Push(old_document.parent(old_current.node).unwrap()));
                let mut subtree = recursively_append(new_cursor.node, new_document);
                patches.append(&mut subtree);
                while let Some(new_cursor) = new_next.pop_front() {
                    patches.push_back(Patch::Push(old_document.parent(old_current.node).unwrap()));
                    let mut subtree = recursively_append(new_cursor.node, new_document);
                    patches.append(&mut subtree);
                }
            }
            // We've reached the end of the worklist, at the same time, we're done
            (None, None) => break,
        }
    }

    patches
}

fn diff_attributes(
    node: NodeRef,
    old: &Element,
    new: &Element,
    old_document: &Document,
    new_document: &Document,
) -> VecDeque<Patch> {
    use std::collections::hash_map::Entry;
    
    let mut patches = VecDeque::new();
    let mut old_attribute_names = FxHashSet::default();
    let mut new_attribute_names = FxHashSet::default();
    let mut old_attributes = FxHashMap::default();
    let mut new_attributes = FxHashMap::default();

    let old_attrs = old.attributes(&old_document.attribute_lists);
    let new_attrs = new.attributes(&new_document.attribute_lists);

    for oattr in old_attrs {
        let old_attr = &old_document.attrs[*oattr];
        old_attribute_names.insert((old_attr.namespace, old_attr.name));
        match old_attributes.entry((old_attr.namespace, old_attr.name)) {
            Entry::Vacant(entry) => {
                entry.insert(vec![(*oattr, &old_attr.value)]);
            }
            Entry::Occupied(mut entry) => {
                let values = entry.get_mut();
                if values.iter().copied().any(|(_, v)| v == &old_attr.value) {
                    continue;
                }
                values.push((*oattr, &old_attr.value));
            }
        }
    }

    for nattr in new_attrs {
        let new_attr = &new_document.attrs[*nattr];
        new_attribute_names.insert((new_attr.namespace, new_attr.name));
        match new_attributes.entry((new_attr.namespace, new_attr.name)) {
            Entry::Vacant(entry) => {
                entry.insert(vec![(*nattr, &new_attr.value)]);
            }
            Entry::Occupied(mut entry) => {
                let values = entry.get_mut();
                if values.iter().copied().any(|(_, v)| v == &new_attr.value) {
                    continue;
                }
                values.push((*nattr, &new_attr.value));
            }
        }
    }

    // Additions (in new, not in old)
    for diff in new_attribute_names.difference(&old_attribute_names) {
        // Issue patch to add this attribute to the old
        patches.extend(
            new_attributes[&diff]
                .iter()
                .copied()
                .map(|(_, value)| Patch::AddAttributeTo {
                    node,
                    attr: Attribute {
                        namespace: diff.0,
                        name: diff.1,
                        value: value.clone(),
                    },
                }),
        );
    }

    // Removals (in old, not in new)
    for diff in old_attribute_names.difference(&new_attribute_names) {
        // Issue patch to remove this attribute from the old
        patches.push_back(Patch::RemoveAttributeByName {
            node,
            namespace: diff.0,
            name: diff.1,
        });
    }

    // Modifications (in both)
    for diff in old_attribute_names.intersection(&new_attribute_names) {
        let old_values = &old_attributes[&diff];
        let new_values = &new_attributes[&diff];

        // If no values have changed, we're done
        if old_values
            .iter()
            .map(|(_, v)| v)
            .eq(new_values.iter().map(|(_, v)| v))
        {
            continue;
        }

        // Otherwise, for each value change, issue a patch to remove the old value and add the new
        todo!()
    }

    patches
}

fn recursively_append(src: NodeRef, src_document: &Document) -> VecDeque<Patch> {
    use petgraph::visit::{depth_first_search, DfsEvent};

    let mut patches = VecDeque::new();

    depth_first_search(src_document, Some(src), |event| {
        match event {
            DfsEvent::Discover(node, _) => {
                match src_document.get(node) {
                    Node::Leaf(ref s) => patches.push_back(Patch::CreateAndMoveTo {
                        node: Node::Leaf(s.clone()),
                    }),
                    Node::Element(ref elem) => {
                        let element = Element {
                            namespace: elem.namespace,
                            tag: elem.tag,
                            attributes: AttributeList::new(),
                        };
                        patches.push_back(Patch::CreateAndMoveTo {
                            node: Node::Element(element),
                        });
                        for attr in elem.attributes.as_slice(&src_document.attribute_lists) {
                            let attribute = &src_document.attrs[*attr];
                            patches.push_back(Patch::AddAttribute {
                                attr: attribute.clone(),
                            });
                        }
                    }
                    // Ignore the root
                    Node::Root => (),
                };
            }
            DfsEvent::TreeEdge(_from, _to) => {
                // Moving down the tree one level, ignored because Discover takes care of traversing for us
                ()
            }
            DfsEvent::Finish(_node, _) => {
                // All children of `node` have been visited, and we're returning up the tree
                patches.push_back(Patch::Attach);
            }
            // This is a tree, these types of edges aren't allowed
            DfsEvent::BackEdge(_, _) 
            | DfsEvent::CrossForwardEdge(_, _) => unreachable!(),
        }
    });

    patches
}