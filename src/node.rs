use std::rc::{Rc, Weak};
use std::collections::HashMap;


static mut LAST_ID: u32 = 0;

#[derive(Debug, PartialEq)]
pub enum VARIANT{
    Root,
    Group,
    Package,
    Dependency,
    State,
    Item, 
}

#[derive(Debug, PartialEq, Clone)]
pub enum STATE{
    Open,
    Planed,
    Working,
    Closed,
    Unknown,
}
#[derive(Debug, PartialEq, Clone)]
pub enum PROPERTY{
    Muss,
    Soll,
    NiceToHave,
    Unknown,
}
// All nodes have the same lifetime
#[derive(Debug)]
pub struct Node  {
    pub id: u32,
    pub name: String,
    pub file: String,
    pub kind: VARIANT,
    pub state: STATE,
    pub property: PROPERTY,
    pub cost: f32,
    pub day: u32,
    pub children: Vec<Node>,  // ownership
}
// maybe we should have a list of nodes and a list of edges for the graph

impl Node{
    pub fn new(line: String, path: &std::path::Path) -> Self{
        let mut currId = 0;
        unsafe{
            LAST_ID+=1;
            currId = LAST_ID;
        }
        let (kind, state) = match line.as_str() {
          "Planned" => (VARIANT::State, STATE::Open),
          "Done" => (VARIANT::State, STATE::Closed),
          "Dependencies" => (VARIANT::Dependency, STATE::Unknown),
          _ => (VARIANT::Item, STATE::Unknown),
        };
        Self{
            id: currId,
            name: line,
            file: path.to_str().unwrap().to_string(),
            kind: kind,
            state: state,
            property: PROPERTY::Unknown,
            cost: 0.0,
            day: 0,
            children: Vec::new(),
        }
    }   
    pub fn new2(line: String, path: &std::path::Path, kind: VARIANT) -> Self{
        let mut currId = 0;
        unsafe{
            LAST_ID+=1;
            currId = LAST_ID;
        }
        Self{
            id: currId,
            name: line,
            file: path.to_str().unwrap().to_string(),
            kind: kind,
            state: STATE::Unknown,
            property: PROPERTY::Unknown,
            cost: 0.0,
            day: 0,
            children: Vec::new(),
        }
    }   
}

pub fn setProperty(node: &mut Node, prop: &str)
{
    let prop = match prop.to_ascii_lowercase().as_str() {
      "muss" =>  PROPERTY::Muss,
      "soll" => PROPERTY::Soll,
      "nicetohave" => PROPERTY::NiceToHave,
      "nice to have" => PROPERTY::NiceToHave,
      _ => PROPERTY::Unknown,
    };
    node.property = prop;
}


pub fn addChild(parent: &mut Node, mut child: Node )
{
    match (parent.state.clone(), child.state.clone()){
        (STATE::Unknown, _) => {parent.state= child.state.clone()},
        (STATE::Open, STATE::Closed) | (STATE::Closed, STATE::Open) => {
            parent.state = STATE::Working 
        },
        (_, STATE::Unknown) => {child.state= parent.state.clone()},
        (_, _) => {}
    }
    parent.cost += child.cost;
    parent.day += child.day;
    parent.children.push(child);
}

#[cfg(test)]
mod NodeTest {
    use super::*;
    #[test]
    fn new() {
        let testNode = Node::new("test".to_string(), VARIANT::Work);
        assert_eq!(testNode.name, "test");
        assert_eq!(testNode.state, STATE::Open);
        assert_eq!(testNode.kind, VARIANT::Work);
        assert_eq!(testNode.cost, 0.0);
        assert_eq!(testNode.budget, 0.0);
        assert_eq!(testNode.expense, 0.0);
        assert_eq!(testNode.dependencies.len(), 0);
        assert_eq!(testNode.children.len(), 0);
    }
}
