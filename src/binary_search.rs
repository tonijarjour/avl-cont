use std::cmp;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::mem;

#[derive(Clone)]
struct Node<T> {
    value: T,
    left: Option<usize>,
    right: Option<usize>,
    height: i8,
}

impl<T> Node<T> {
    const fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
            height: 0,
        }
    }
}

#[derive(Clone)]
pub struct Tree<T> {
    data: Vec<Option<Node<T>>>,
    free: Vec<usize>,
    root: usize,
    size: usize,
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tree<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            free: Vec::new(),
            root: 0,
            size: 0,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.size
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns a reference to the value at INDEX if it exists.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data[index].as_ref().map(|n| &n.value)
    }

    // Tries to use a free'd index, otherwise pushes.
    // Returns the index that was used.
    fn insert_helper(&mut self, value: T) -> usize {
        if let Some(n) = self.free.pop() {
            self.data[n] = Some(Node::new(value));
            n
        } else {
            self.data.push(Some(Node::new(value)));
            self.len()
        }
    }

    // Removes None values from the end of the data Vec.
    // Removes corresponding indices from the free Vec.
    fn clean_tail(&mut self) -> Option<()> {
        let mut popped = Vec::new();
        while self.data.last()?.is_none() {
            self.data.pop();
            popped.push(self.data.len());
        }
        self.free.retain(|i| {
            popped.iter().position(|p| i == p).map_or(true, |n| {
                popped.swap_remove(n);
                false
            })
        });

        Some(())
    }

    fn update_and_balance(&mut self, mut visited_indices: Vec<usize>) -> Option<()> {
        // This is guarenteed to run at least once.
        // The last run is always on the root index.
        while let Some(index) = visited_indices.pop() {
            let balance_factor = self.update_height(index)?;
            let new_parent = self.balance_node(index, balance_factor)?;

            // Set the grandfather to point to the subtree's new root.
            match visited_indices.last() {
                Some(n) => {
                    let grandfather_data = self.data[*n].as_ref()?;
                    let child_is_left = grandfather_data.left.map_or(false, |left| index == left);

                    if child_is_left {
                        self.data[*n].as_mut()?.left = Some(new_parent);
                    } else {
                        self.data[*n].as_mut()?.right = Some(new_parent);
                    }
                }
                None => self.root = new_parent,
            }
        }

        Some(())
    }

    // Update a node's height to be 1 + max_height between its children.
    // If a node has no children, its height is calculated as 1 + -1 = 0.
    // Returns the balance factor of the node. This is calcuated as the
    // difference between its children's heights. If the right child has
    // a height of 3 while the left has 1, the balance factor would be
    // calculated as 2. This value or its inverse would require the tree
    // to be rebalanced.
    fn update_height(&mut self, index: usize) -> Option<i8> {
        let node_data = self.data[index].as_ref()?;

        let left_height: i8 = match node_data.left {
            Some(n) => self.data[n].as_ref()?.height,
            None => -1,
        };
        let right_height: i8 = match node_data.right {
            Some(n) => self.data[n].as_ref()?.height,
            None => -1,
        };

        let node_data = self.data[index].as_mut()?;
        node_data.height = 1 + cmp::max(left_height, right_height);

        Some(right_height - left_height)
    }

    // Balance a node if one of its sides is two nodes taller than the other.
    // In a sequence where the nodes connect A -> B -> C, a rotation is done
    // such that node B points to its parent: A <- B -> C. If node B does not
    // have a child in the same direction: A -> B and C <- B, node A is set
    // to point to node C and C to B: A -> C -> B, then node C is set to
    // point to node A: A <- C -> B. Returns the new parent's index.
    fn balance_node(&mut self, index: usize, balance_factor: i8) -> Option<usize> {
        let node_data = self.data[index].as_ref()?;
        match balance_factor {
            -2 => {
                let left_node = self.data[node_data.left?].as_ref()?;
                match left_node.left {
                    Some(_) => self.rotate_right(index),
                    None => self.rotate_left_right(index),
                }
            }
            2 => {
                let right_node = self.data[node_data.right?].as_ref()?;
                match right_node.right {
                    Some(_) => self.rotate_left(index),
                    None => self.rotate_right_left(index),
                }
            }
            _ => Some(index),
        }
    }

    fn rotate_right(&mut self, index: usize) -> Option<usize> {
        let left_index = self.data[index].as_ref()?.left?;
        let left_right_index = self.data[left_index].as_ref()?.right;

        let node_data = self.data[index].as_mut()?;
        node_data.left = left_right_index;

        let left_data = self.data[left_index].as_mut()?;
        left_data.right = Some(index);

        self.update_height(index);
        self.update_height(left_index);

        Some(left_index)
    }

    fn rotate_left(&mut self, index: usize) -> Option<usize> {
        let right_index = self.data[index].as_ref()?.right?;
        let right_left_index = self.data[right_index].as_ref()?.left;

        let node_data = self.data[index].as_mut()?;
        node_data.right = right_left_index;

        let right_data = self.data[right_index].as_mut()?;
        right_data.left = Some(index);

        self.update_height(index);
        self.update_height(right_index);

        Some(right_index)
    }

    fn rotate_left_right(&mut self, index: usize) -> Option<usize> {
        let left_index = self.data[index].as_ref()?.left?;

        self.data[index].as_mut()?.left = self.rotate_left(left_index);
        self.rotate_right(index)
    }

    fn rotate_right_left(&mut self, index: usize) -> Option<usize> {
        let right_index = self.data[index].as_ref()?.right?;

        self.data[index].as_mut()?.right = self.rotate_right(right_index);
        self.rotate_left(index)
    }
}

impl<T: Ord> Tree<T> {
    /// Returns the index of VALUE if it is found.
    pub fn contains(&self, value: &T) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        let parent_index = match self.contains_helper(value) {
            Some((true, Some(n))) => *n.last()?,
            Some((true, None)) => return Some(self.root),
            _ => return None,
        };
        let parent_data = self.data[parent_index].as_ref()?;

        match value.cmp(&parent_data.value) {
            Ordering::Less => parent_data.left,
            Ordering::Greater => parent_data.right,
            Ordering::Equal => unreachable!(),
        }
    }

    // Returns a bool and all visited indices up to
    // and including the (prospective) parent index.
    fn contains_helper(&self, value: &T) -> Option<(bool, Option<Vec<usize>>)> {
        let mut current_index = self.root;
        let mut current_data = self.data[current_index].as_ref()?;

        if value == &current_data.value {
            return Some((true, None));
        }

        let mut visited_indices = vec![current_index];

        // Returns false if the value must be a child of a node
        // that lacks a child on the correct side.
        while current_data.left.is_some() || current_data.right.is_some() {
            current_index = match value.cmp(&current_data.value) {
                Ordering::Less => match current_data.left {
                    Some(n) => n,
                    None => return Some((false, Some(visited_indices))),
                },
                Ordering::Greater => match current_data.right {
                    Some(n) => n,
                    None => return Some((false, Some(visited_indices))),
                },
                Ordering::Equal => current_index,
            };

            current_data = self.data[current_index].as_ref()?;
            if value == &current_data.value {
                return Some((true, Some(visited_indices)));
            }

            visited_indices.push(current_index);
        }

        Some((false, Some(visited_indices)))
    }

    /// Insert VALUE into the tree. Must be unique. Returns the index that was
    /// used, or None if it wasn't inserted.
    pub fn insert(&mut self, value: T) -> Option<usize> {
        if self.is_empty() {
            self.data.push(Some(Node::new(value)));
            self.size = 1;
            return Some(0);
        }

        let Some((false, Some(visited_indices))) = self.contains_helper(&value) else {
            return None;
        };
        let parent_index = *visited_indices.last()?;
        let insert_index;

        match &value.cmp(&self.data[parent_index].as_ref()?.value) {
            Ordering::Less => {
                insert_index = self.insert_helper(value);
                self.data[parent_index].as_mut()?.left = Some(insert_index);
            }
            Ordering::Greater => {
                insert_index = self.insert_helper(value);
                self.data[parent_index].as_mut()?.right = Some(insert_index);
            }
            Ordering::Equal => unreachable!(),
        }

        self.update_and_balance(visited_indices);
        self.size += 1;
        Some(insert_index)
    }

    /// Remove VALUE from the tree.
    pub fn remove(&mut self, value: &T) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let is_root = match self.remove_root_helper(value) {
            Some((b, None)) => b,
            Some((true, Some(return_val))) => return Some(return_val),
            _ => unreachable!(),
        };

        let mut visited_indices = match self.contains_helper(value) {
            Some((true, None)) => vec![self.root],
            Some((true, Some(n))) => n,
            _ => return None,
        };
        let parent_index = *visited_indices.last()?;
        let parent_data = self.data[parent_index].as_ref()?;
        let child_is_left = value < &parent_data.value;

        let val_index = match (is_root, child_is_left) {
            (true, _) => self.root,
            (false, true) => parent_data.left?,
            (false, false) => parent_data.right?,
        };
        let val_data = self.data[val_index].as_ref()?;
        let return_val;

        if let (Some(val_left), Some(val_right)) = (val_data.left, val_data.right) {
            let mut left_data = self.data[val_left].as_ref()?;
            let mut right_data = self.data[val_right].as_ref()?;

            let mut current_index;

            // The node being removed will have its child tree modified.
            // So it must be added to the visited indices to be updated
            // and balanced. This is also done for every node visited
            // up to the replacement. The root index was already added.
            if !is_root {
                visited_indices.push(val_index);
            }

            // Climbs to the replacement node on whichever side
            // has the greater height. If it doesn't have a child,
            // its parent's pointer is set to None.
            let replace_index = if left_data.height > right_data.height {
                current_index = val_left;
                while let Some(n) = left_data.right {
                    visited_indices.push(current_index);
                    current_index = n;
                    left_data = self.data[current_index].as_ref()?;
                }

                if let Some(n) = left_data.left {
                    self.data.swap(current_index, n);
                    n
                } else {
                    self.data[*visited_indices.last()?].as_mut()?.right = None;
                    current_index
                }
            } else {
                current_index = val_right;
                while let Some(n) = right_data.left {
                    visited_indices.push(current_index);
                    current_index = n;
                    right_data = self.data[current_index].as_ref()?;
                }

                if let Some(n) = right_data.right {
                    self.data.swap(current_index, n);
                    n
                } else {
                    self.data[*visited_indices.last()?].as_mut()?.left = None;
                    current_index
                }
            };

            self.free.push(replace_index);
            let replace = Some(Node {
                value: mem::replace(&mut self.data[replace_index], None)?.value,
                // If the value at the index is None, set pointer to None.
                left: self.data[val_left].as_ref().map(|_| val_left),
                right: self.data[val_right].as_ref().map(|_| val_right),
                // The height of this new Node doesn't matter
                // since it will be updated.
                height: 0,
            });

            return_val = mem::replace(&mut self.data[val_index], replace)?.value;
        } else {
            if let Some(child_index) = val_data.left.xor(val_data.right) {
                if child_is_left {
                    self.data[parent_index].as_mut()?.left = Some(child_index);
                } else {
                    self.data[parent_index].as_mut()?.right = Some(child_index);
                }
            } else if child_is_left {
                self.data[parent_index].as_mut()?.left = None;
            } else {
                self.data[parent_index].as_mut()?.right = None;
            }

            self.free.push(val_index);
            return_val = mem::replace(&mut self.data[val_index], None)?.value;
        }

        self.update_and_balance(visited_indices);
        self.clean_tail();
        self.size -= 1;
        Some(return_val)
    }

    // Handling trivial cases for removing a value at root.
    fn remove_root_helper(&mut self, value: &T) -> Option<(bool, Option<T>)> {
        let return_val;
        let root_data = self.data[self.root].as_ref()?;
        if value == &root_data.value {
            if self.size == 1 {
                return_val = self.data.pop()??.value;
                self.free.clear();
                self.data.clear();
                self.size = 0;
                self.root = 0;

                return Some((true, Some(return_val)));
            }

            if let Some(new_root) = root_data.left.xor(root_data.right) {
                return_val = mem::replace(&mut self.data[self.root], None)?.value;
                self.size -= 1;
                self.free.push(self.root);
                self.root = new_root;
                self.clean_tail();

                return Some((true, Some(return_val)));
            }

            return Some((true, None));
        }

        Some((false, None))
    }
}

pub struct Iter<T> {
    data: Vec<Option<Node<T>>>,
    queue: VecDeque<usize>,
}

impl<T> IntoIterator for Tree<T> {
    type Item = T;
    type IntoIter = Iter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            data: self.data,
            queue: VecDeque::from([self.root]),
        }
    }
}

impl<T> Iterator for Iter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.queue.pop_front() {
            let current = mem::replace(&mut self.data[current], None)?;

            if let Some(n) = current.left {
                self.queue.push_back(n);
            }
            if let Some(n) = current.right {
                self.queue.push_back(n);
            }

            Some(current.value)
        } else {
            None
        }
    }
}
