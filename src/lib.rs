pub mod binary_search;

#[cfg(test)]
mod data_structures {
    use super::binary_search;

    #[test]
    fn avl_tree() {
        let mut tree = binary_search::Tree::default();

        // insert values
        for n in 0..1000 {
            tree.insert(n);
        }

        // remove a value
        tree.remove(511);

        // check if a value is in the tree, returns its index
        let value_index = tree.contains(732).unwrap();

        // get a reference to the value
        assert_eq!(tree.get(value_index).unwrap(), &732);
        assert_eq!(tree.contains(511), None);
    }
}

