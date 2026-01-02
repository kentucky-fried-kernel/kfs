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
    /// * `node` must point to a valid, properly aligned, initialized `T`.
    /// * `node` must remain valid for the whole lifetime of this list.
    pub unsafe fn add_front(&mut self, node: &mut NonNull<T>) {
        if self.head.is_none() {
            // SAFETY:
            // Assuming above Safety guidelines were followed, we can safely dereference `node`.
            unsafe { *node.as_mut().next_ptr_mut() = None };
            self.head = Some(*node);
            return;
        }

        // SAFETY:
        // Assuming above Safety guidelines were followed, we can safely dereference `node`.
        unsafe { *node.as_mut().next_ptr_mut() = self.head };

        self.head = Some(*node);
    }

    /// Removes and returns the head of the list.
    ///
    /// This is considered safe because it doesn't dereference any pointers that weren't
    /// already verified when added to the list.
    pub fn take_head(&mut self) -> Option<NonNull<T>> {
        let head = self.head.take()?;

        // SAFETY:
        // Assuming this list was only accessed via its documented API (`add_front`), it should be
        // safe to dereference `head`.
        let new_head = unsafe { head.as_ref() }.next_ptr();

        self.head = new_head;

        Some(head)
    }

    pub fn pop_at(&mut self, node: &NonNull<T>) -> Option<NonNull<T>> {
        let mut last = self.head()?;

        if last == *node {
            self.set_head(None);
            return Some(last);
        }

        while let Some(curr) = unsafe { last.as_ref() }.next_ptr() {
            if curr == *node {
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
        // Assuming the `List`'s guidelines were followed, it should always be safe to access the next
        // pointer until its stored value is `None`.
        self.current = unsafe { current.as_ref() }.next_ptr();
        Some(current)
    }
}
