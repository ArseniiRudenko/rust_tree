use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Add;
use std::rc::{Rc, Weak};

trait Monad<T> where T: Debug{

    type TR<T1> where T1: Debug;

    fn map<T1>(&self,f: fn(&T)->T1) -> Rc<Self::TR<T1>> where T1: Debug;

    fn flat_map<T1>(&self,f: fn(&T) -> Rc<Self::TR<T1>>) -> Rc<Self::TR<T1>> where T1: Debug;
}


trait Tree<T>{
    fn is_root(&self)->bool;

    fn add_child(self:&mut Rc<Self>,value:T);

    fn set_child_value(self:&mut Rc<Self>, index:usize, value:T);

    fn insert_subtree_at(self:&mut Rc<Self>,index:usize,value:Rc<Self>);

    fn new(value:T)->Rc<Self>;
}


#[derive(Debug)]
struct Node<T> where T: Debug {
    content: T,
    parent: Weak<Node<T>>,
    children : RefCell<Vec<Rc<Node<T>>>>
}

impl<T> Tree<T> for Node<T> where T: Debug+Clone{

    fn is_root(&self) -> bool {
        self.parent.upgrade().is_none()
    }

    fn add_child(self: &mut Rc<Self> , value:T){
        let child=Rc::new(
            Node{
                content:value,
                parent: Rc::downgrade(self),
                children: RefCell::new(vec![])
            });
        self.children.borrow_mut().push(child);
    }

    fn set_child_value(self: &mut Rc<Node<T>>, index: usize, value: T) {
        let child=Rc::new(
            Node{
                content:value,
                parent: Rc::downgrade(self),
                children: self.children.borrow_mut()[index].children.clone()
            });
        self.children.borrow_mut()[index] = child
    }

    fn insert_subtree_at(self: &mut Rc<Node<T>>, index: usize, value: Rc<Node<T>>) {
        let child=Rc::new(
            Node{
                content: value.content.clone(),
                parent: Rc::downgrade(self),
                children: value.children.clone()
            });
        self.children.borrow_mut()[index] = child
    }

    fn new(value:T) -> Rc<Node<T>> {
        Rc::new(
            Node{
                content:value,
                parent: Weak::new(),
                children: RefCell::new(vec![])
            }
        )
    }
}

impl<T> Node<T> where T: Debug{

    fn get_child(self: & mut Rc<Self>, index:usize) -> Result<Rc<Node<T>>,String> {
        self
            .children
            .try_borrow()
            .map_err(|err|err.to_string())
            .and_then(|b|  //obtain clone of node rc at required index
                b.get(index).map(
                    |f|
                        f.clone()
                ).ok_or("index out of range".to_string())
            )
    }

    fn map_internal<T1>(&self, f: fn(&T) -> T1, parent:Weak<Node<T1>>) -> Rc<Node<T1>> where T1: Debug {
        Rc::new_cyclic(|weak_ref|{
            let mut new_root = Node{
                content:f(&self.content),
                parent,
                children: RefCell::new(vec![])
            };
            new_root.children =
                RefCell::new(
                    self.children
                    .borrow_mut().iter()
                    .map(|c|c.map_internal(f,weak_ref.clone()))
                    .collect()
                );
            return new_root
        })
    }

}

impl<T> Monad<T> for Node<T> where T: Debug{

    type TR<T1> = Node<T1> where T1: Debug;

    fn map<T1>(&self, f: fn(&T) -> T1) -> Rc<Node<T1>> where T1: Debug{
        Rc::new_cyclic(|weak_ref|{
            let mut new_root = Node{
                content:f(&self.content),
                parent: Weak::new(),
                children: RefCell::new(vec![])
            };
            new_root.children =
                RefCell::new(
                self.children
                    .borrow_mut().iter()
                    .map(|c|c.map_internal(f,weak_ref.clone()))
                    .collect()
            );
            return new_root
        })
    }

    fn flat_map<T1>(&self, f: fn(&T) -> Rc<Node<T1>>) -> Rc<Node<T1>> where T1: Debug{
        todo!()
    }

}



fn main() {
    println!("Hello, world!");
    let mut tree = Node::new("what".to_string());
    tree.add_child("is".to_string());
    println!("{:?}",tree);
    tree.get_child(0).unwrap().add_child("second order Child".to_string());

    println!("{:?}",tree);
    let mut new_tree = tree.map(|s|{s.clone().add("some")});
    println!("{:?}",new_tree);
    new_tree.set_child_value(0,"is not some".to_string());
    println!("{:?}",new_tree);
}
