slotmap::new_key_type!{ ///Node Identifier!
                        pub struct NodeKey; }

///Node - contains the item, its relative transform, its absolute transform and a list of its
///children.
pub struct Node<I, T> 
where T : Default
{
    item: I,
    relative_transform: T,
    absolute_transform: T,
    children: Vec<NodeKey>,
}

///Node Tree - generic over I, the item type, and T, the transform type.
///Stores nodes in a dense slot map from slotmap, and needs a root identifier.
pub struct NodeTree<I, T> 
where T : Default + Clone
{
    pub nodes: slotmap::DenseSlotMap<NodeKey, Node<I, T>>,
    pub root: NodeKey,
}
impl<I, T> NodeTree<I, T> 
where T : Default + Clone
{
    pub fn new(root_item: I) -> Self {
        let mut nodes = slotmap::DenseSlotMap::with_key();
        let root = nodes.insert(Node { item: root_item, relative_transform: T::default(), absolute_transform: T::default(), children: vec![] });
        Self { nodes, root }
    }
    
    pub fn insert(&mut self, item: I, relative_transform: T, parent: NodeKey) -> NodeKey {
        let key = self.nodes.insert(Node { item, relative_transform: relative_transform.clone(), absolute_transform: relative_transform, children: vec![] });
        self.nodes.get_mut(parent).expect("Tried to insert node as a child of a non-existent node!").children.push(key); 
        key
    }

    pub fn get(&self, key: NodeKey) -> Option<&I> {
        match self.nodes.get(key) {
            Some(node) => Some(&node.item),
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: NodeKey) -> Option<&mut I> {
        match self.nodes.get_mut(key) {
            Some(node) => Some(&mut node.item),
            None => None,
        }
    }

    pub fn remove(&mut self, key: NodeKey) -> Option<Node<I, T>> {
        self.nodes.remove(key)
    }

    pub fn get_pos(&self, key: NodeKey) -> Option<&T> {
        match self.nodes.get(key) {
            Some(node) => Some(&node.relative_transform),
            None => None,
        }
    }
    
    pub fn get_absolute_pos(&self, key: NodeKey) -> Option<&T> {
        match self.nodes.get(key) {
            Some(node) => Some(&node.absolute_transform),
            None => None,
        }
    }

    pub fn get_pos_mut(&mut self, key: NodeKey) -> Option<&mut T> {
        match self.nodes.get_mut(key) {
            Some(node) => Some(&mut node.relative_transform),
            None => None,
        }
    }

    pub fn iter<'r>(&'r self) -> NodeIter<'r, I, T> {
        NodeIter {
            nodes: &self.nodes,
            stack: vec![self.root],
            counted: vec![],
        }
    }
    
    pub fn iter_from<'r>(&'r self, node: NodeKey) -> NodeIter<'r, I, T> {
        NodeIter {
            nodes: &self.nodes,
            stack: vec![node],
            counted: vec![],
        }
    }

    pub fn keys(&self) -> Vec<NodeKey> {
        self.iter().collect::<Vec<_>>()
    }
}
impl<I, T> NodeTree<I, T>
where T : Default + std::ops::Add<T, Output=T> + Clone {
    pub fn update_clone(&mut self) {
        for k in self.keys() {
            let obj = self.nodes.get(k).unwrap();
            let abs = obj.absolute_transform.clone();
            let children = obj.children.clone();
            for child in children {
                let mut_obj = self.nodes.get_mut(child).unwrap();
                mut_obj.absolute_transform = abs.clone() + mut_obj.relative_transform.clone();
            }
        }
    }
}

impl<I, T> NodeTree<I, T>
where T : Default + std::ops::Add<T, Output=T> + Copy + Clone {
    pub fn update(&mut self) {
        for k in self.keys() {
            let obj = self.nodes.get(k).unwrap();
            let abs = obj.absolute_transform;
            let children = obj.children.clone();
            for child in children {
                let mut_obj = self.nodes.get_mut(child).unwrap();
                mut_obj.absolute_transform = abs + mut_obj.relative_transform;
            }
        }
    }
    pub fn move_pos(&mut self, node: NodeKey, transform: T) {
        if let Some(node) = self.nodes.get_mut(node) {
            node.relative_transform = node.relative_transform + transform;
        }
    }
    pub fn set_pos(&mut self, node: NodeKey, transform: T) {
        if let Some(node) = self.nodes.get_mut(node) {
            node.relative_transform = transform;
        }
    }
}

impl<I, T> NodeTree<I, T>
where T : Default + std::ops::Add<T, Output=T> + std::ops::Neg<Output=T> + Clone 
{
    pub fn set_pos_absolute(&mut self, node: NodeKey, transform: T) {
        let mut parent_pos = None;
        for key in self.iter() {
            let current_node = self.nodes.get(key).unwrap();
            if current_node.children.contains(&node) {
                parent_pos = Some(current_node.absolute_transform.clone());
            }
        }
        if let (Some(node), Some(parent)) = (self.nodes.get_mut(node), parent_pos) {
            node.relative_transform = transform.clone() + -parent;
            node.absolute_transform = transform;
        }
    }
}

pub struct NodeIter<'r, I, T>
where T : Default + Clone {
    nodes: &'r slotmap::DenseSlotMap<NodeKey, Node<I, T>>,
    stack: Vec<NodeKey>,
    counted: Vec<NodeKey>,
}

impl<'r, I, T> Iterator for NodeIter<'r, I, T>
where T : Default + Clone{
    type Item = NodeKey;
    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            Some(current) => {
                match self.counted.contains(&current) {
                    true => { log::error!("Node tree is cyclic."); panic!("Node tree is cyclic."); },
                    false => {
                        self.counted.push(current);
                        match self.nodes.get(current) {
                            Some(node) => {
                                for child in &node.children {
                                    self.stack.push(*child);
                                }
                                Some(current)
                            },
                            None => { log::error!("Key pointing to non-existent node."); panic!("Key pointing to non-existent node."); },
                        }
                    }
                }
            },
            None => None,
        }
    }
}


