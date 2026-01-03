use core::ptr::NonNull;

pub trait IntrusiveLink {
    fn next_ptr(&self) -> Option<NonNull<Self>>
    where
        Self: Sized;

    fn next_ptr_mut(&mut self) -> &mut Option<NonNull<Self>>
    where
        Self: Sized;
}

#[derive(Clone, Copy, Debug)]
pub struct List<T> {
    head: Option<NonNull<T>>,
}

impl<T: IntrusiveLink> const Default for List<T>
where
    T: Copy,
{
    fn default() -> Self {
        Self { head: None }
    }
}

impl<T: IntrusiveLink> List<T>
where
    T: Copy,
{
    #[must_use]
    pub fn head(&self) -> Option<NonNull<T>> {
        self.head
    }

    pub fn set_head(&mut self, head: Option<NonNull<T>>) {
        self.head = head;
    }

    /// Adds a node to the front of the `List<T>`.
    ///
    /// # Safety
    /// If any of the following conditions are violated, the result is Undefined Behavior:
    /// * `node` must point to a valid, properly aligned, initialized instance of `T`.
    /// * `node` must remain valid for the whole lifetime of this list (i.e., it must outlive this
    ///   `List<T>` instance or be removed from it before becoming invalid).
    /// * `node` must not be concurrently modified through any other pointer.
    pub unsafe fn add_front(&mut self, node: &mut NonNull<T>) {
        if self.head.is_none() {
            // SAFETY:
            // The caller guarantees `node` points to valid, initialized memory.
            unsafe { *node.as_mut().next_ptr_mut() = None };
            self.head = Some(*node);
            return;
        }

        // SAFETY:
        // The caller guarantees `node` points to valid, initialized memory.
        unsafe { *node.as_mut().next_ptr_mut() = self.head };

        self.head = Some(*node);
    }

    /// Removes and returns the head of the list. Returns `None` if the list is empty.
    ///
    /// This operation is safe because it only dereferences pointers that were verified as valid
    /// when added via `add_front`, under the assumption that the caller upheld the safety
    /// contract.
    pub fn take_head(&mut self) -> Option<NonNull<T>> {
        let head = self.head.take()?;

        // SAFETY:
        // `head` was previously added via `add_front`, whose safety contract
        // requires that the pointer remain valid. We rely on the caller having
        // upheld that contract.
        let new_head = unsafe { head.as_ref() }.next_ptr();

        self.head = new_head;

        Some(head)
    }

    /// Removes a specific node from the list if it exists.
    ///
    /// Returns `Some(NonNull<T>)` (a pointer to the removed node) if the node was found and
    /// removed, `None` otherwise.
    ///
    /// This operation is safe under the same assumptions as `take_head`: it only dereferences
    /// pointers that were verified as valid when added via `add_front`.
    pub fn pop_at(&mut self, node: &NonNull<T>) -> Option<NonNull<T>> {
        let mut last = self.head()?;

        if last == *node {
            // SAFETY:
            // `last` ist the current head of the list and was previously added via `add_front`, whose safety
            // contract requires that the pointer remain valid while in the list.
            let new_head = unsafe { last.as_ref().next_ptr() };
            self.set_head(new_head);
            return Some(last);
        }

        // SAFETY:
        // `last` is in the list, so it was added via `add_front` and the caller guarantees it remains
        // valid while in the list.
        while let Some(curr) = unsafe { last.as_ref() }.next_ptr() {
            if curr == *node {
                // SAFETY:
                // Both `last` and `curr` are in the list and were subject to the safety requirements of
                // `add_front`.
                *unsafe { last.as_mut() }.next_ptr_mut() = unsafe { curr.as_ref() }.next_ptr();
                return Some(curr);
            }
            last = curr;
        }
        None
    }
}

impl<T: IntrusiveLink> IntoIterator for List<T> {
    type IntoIter = ListIntoIterator<T>;
    type Item = NonNull<T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { current: self.head }
    }
}

pub struct ListIntoIterator<T> {
    current: Option<NonNull<T>>,
}

impl<T: IntrusiveLink> Iterator for ListIntoIterator<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;

        // SAFETY:
        // `current` was in the list, meaning it was added via `add_front`. Its safety contract requires all
        // nodes remain valid while in the list.
        self.current = unsafe { current.as_ref() }.next_ptr();
        Some(current)
    }
}
