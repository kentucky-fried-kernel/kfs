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

    pub fn add_back(&mut self, node: &mut NonNull<T>) {
        if self.head.is_none() {
            unsafe { *node.as_mut().next_ptr_mut() = None };
            self.head = Some(*node);
            return;
        }

        let mut current_link = self.head;
        let mut last_node_ptr = None;

        while let Some(current) = current_link {
            last_node_ptr = Some(current);
            current_link = unsafe { current.as_ref() }.next_ptr();
        }

        if let Some(mut last) = last_node_ptr {
            let tail = unsafe { last.as_mut() };
            *tail.next_ptr_mut() = Some(*node);
        }

        unsafe { *node.as_mut().next_ptr_mut() = None };
    }

    pub fn add_front(&mut self, node: &mut NonNull<T>) {
        if self.head.is_none() {
            unsafe { *node.as_mut().next_ptr_mut() = None };
            self.head = Some(*node);
            return;
        }

        unsafe { *node.as_mut().next_ptr_mut() = Some(*node) };
        self.head = Some(*node);
    }

    pub fn take_head(&mut self) -> Option<NonNull<T>> {
        let head = self.head.take()?;
        self.head = unsafe { head.as_ref() }.next_ptr();
        Some(head)
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

        self.current = unsafe { current.as_ref() }.next_ptr();
        Some(current)
    }
}
