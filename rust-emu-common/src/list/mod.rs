pub struct Node<T: PartialOrd> {
  data: T,
  next: Option<Box<Node<T>>>,
}

impl<'a, T: PartialOrd> Node<T> {
  pub fn new(data: T) -> Node<T> {
    Node {
      data: data,
      next: None,
    }
  }
  pub fn new_head(data: T, next: Option<Box<Node<T>>>) -> Node<T> {
    Node {
      data: data,
      next: next,
    }
  }
}
pub struct List<T: PartialOrd> {
  pub head: Option<Box<Node<T>>>,
}

impl<'a, T: PartialOrd> List<T> {
  pub fn new() -> List< T> {
    List { head: None }
  }

  pub fn insert(&mut self, data: T) {
    if self.head.is_none() {
      self.head = Some(Box::new(Node::new(data)))
    } else {
      let mut current = self.head.as_mut().unwrap();
      while current.next.is_some() {
        current = current.next.as_mut().unwrap();
      }
      current.next = Some(Box::new(Node::new(data)));
    }
  }

  pub fn insert_by_key(&mut self, data: T) {
    if self.head.is_none() {
      self.head = Some(Box::new(Node::new(data)));
      return;
    }
    if self.head.as_ref().unwrap().data > data {
      self.head = Some(Box::new(Node::new_head(data, self.head.take())));
      return;
    }
    let mut current = self.head.as_mut().unwrap();
    while current.next.is_some() && current.next.as_ref().unwrap().data < data {
      current = current.next.as_mut().unwrap();
    }
    let node = Node::new_head(data, current.next.take());
    current.next = Some(Box::new(node));
  }

  pub fn pop(&mut self) -> Option< T> {
    if self.head.is_none() {
      return None;
    }
    let data = self.head.take().map(|node| {
      self.head = node.next;
      node.data
    });
    data
  }

}

pub struct IntoIter<T: PartialOrd>(List<T>);

impl< T: PartialOrd> Iterator for IntoIter< T> {
  type Item = T;
  fn next(&mut self) -> Option<Self::Item> {
    self.0.pop()
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn test_insert() {
    let mut list = super::List::new();
    list.insert("a");
    list.insert("b");
    list.insert("c");
    list.insert("d");
    list.insert("e");
    list.insert("f");

    assert_eq!(list.pop(), Some("a"));
    assert_eq!(list.pop(), Some("b"));
    assert_eq!(list.pop(), Some("c"));
    assert_eq!(list.pop(), Some("d"));
    assert_eq!(list.pop(), Some("e"));
    assert_eq!(list.pop(), Some("f"));
    assert_eq!(list.pop(), None);
  }

  #[test]
  fn test_insert_with_key() {

    let mut list = super::List::new();
    list.insert_by_key(&'d');
    list.insert_by_key(&'c');
    list.insert_by_key(&'e');
    list.insert_by_key(&'b');
    list.insert_by_key(&'a');
    list.insert_by_key(&'f');

    assert_eq!(list.pop(), Some(&'a'));
    assert_eq!(list.pop(), Some(&'b'));
    assert_eq!(list.pop(), Some(&'c'));
    assert_eq!(list.pop(), Some(&'d'));
    assert_eq!(list.pop(), Some(&'e'));
    assert_eq!(list.pop(), Some(&'f'));
    assert_eq!(list.pop(), None);
  }
}
