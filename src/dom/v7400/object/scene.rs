//! Scene node.

use log::trace;

use crate::dom::v7400::object::{ObjectId, ObjectNodeId};
use crate::dom::v7400::{Core, Document, DowncastId, NodeId};
use crate::dom::AccessError;

/// Scene node ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneNodeId(ObjectNodeId);

impl SceneNodeId {
    /// Creates a new `SceneNodeId`.
    pub(crate) fn new(id: ObjectNodeId) -> Self {
        Self(id)
    }
}

impl From<SceneNodeId> for ObjectNodeId {
    fn from(v: SceneNodeId) -> Self {
        v.0
    }
}

impl DowncastId<SceneNodeId> for ObjectNodeId {
    fn downcast(self, doc: &Document) -> Option<SceneNodeId> {
        trace!("Trying to downcast {:?} to `SceneNodeId`", self);

        let maybe_invalid_id = SceneNodeId::new(self);
        if doc
            .parsed_node_data()
            .scenes()
            .contains_key(&maybe_invalid_id)
        {
            // Valid!
            trace!(
                "Successfully downcasted {:?} to {:?}",
                self,
                maybe_invalid_id
            );
            Some(maybe_invalid_id)
        } else {
            // Invalid.
            trace!(
                "Downcast failed: {:?} is not convertible to `SceneNodeId`",
                self
            );
            None
        }
    }
}

/// Scene node data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SceneNodeData {
    /// Root object ID.
    root_object: ObjectId,
}

impl SceneNodeData {
    /// Loads the scene node data.
    pub(crate) fn load(obj_node_id: ObjectNodeId, core: &Core) -> Result<Self, AccessError> {
        trace!("Loading scene node data from object node {:?}", obj_node_id);

        let child_root_node_id = NodeId::from(obj_node_id)
            .children_by_name(core, "RootNode")
            .next()
            .ok_or_else(|| AccessError::NodeNotFound("`RootNode`".into()))?;
        trace!("Found child node `RootNode`: node={:?}", child_root_node_id);

        let root_object = child_root_node_id
            .node(core)
            .attributes()
            .get(0)
            .ok_or(AccessError::AttributeNotFound(Some(0)))?
            .get_i64()
            .map(ObjectId::new)
            .ok_or(AccessError::UnexpectedAttributeType(Some(0)))?;
        trace!("Got root object id: obj_id={:?}", root_object);

        trace!("Successfully loaded scene node data from {:?}", obj_node_id);

        Ok(Self { root_object })
    }

    /// Returns root object ID.
    pub fn root(&self) -> ObjectId {
        self.root_object
    }
}
