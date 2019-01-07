//! FBX DOM document.

use indextree::Arena;
use string_interner::StringInterner;

use crate::dom::v7400::{Node, NodeData, NodeId, StrSym};

pub use self::loader::Loader;

mod loader;

/// FBX DOM document.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// FBX node names.
    strings: StringInterner<StrSym>,
    /// FBX nodes.
    nodes: Arena<NodeData>,
    /// (Implicit) root node.
    root: NodeId,
}

impl Document {
    /// Create a new `Document`.
    pub(crate) fn new(
        strings: StringInterner<StrSym>,
        nodes: Arena<NodeData>,
        root: NodeId,
    ) -> Self {
        Self {
            strings,
            nodes,
            root,
        }
    }

    /// Resolves the given interned string symbol into the corresponding string.
    ///
    /// Returns `None` if the given symbol is registered to the document.
    pub(crate) fn string(&self, sym: StrSym) -> Option<&str> {
        self.strings.resolve(sym)
    }

    /// Returns the node from the node ID.
    ///
    /// Returns `None` if the node with the given ID is not registered to the
    /// document.
    pub(crate) fn node(&self, id: NodeId) -> Option<Node<'_>> {
        self.nodes.get(id.raw()).map(Node::new)
    }

    /// Returns the root node ID.
    pub fn root(&self) -> NodeId {
        self.root
    }
}
