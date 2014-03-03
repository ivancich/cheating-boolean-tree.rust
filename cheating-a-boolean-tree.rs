#[feature(struct_variant)];
#[feature(managed_boxes)];

use std::os;
use std::at_vec;
use std::io::fs::File;

use std::option::Option;

mod scanner;

enum Gate {
    And,
    Or
}

pub enum Node {
    Leaf { value: bool },
    Interior { gate : Gate, changeable : bool }
}

impl Clone for Node {
    fn clone(&self) -> Node {
        match *self {
            Leaf{value: v} =>
                Leaf{value: v},
            Interior {gate: g, changeable: c} =>
                Interior{gate: g, changeable: c}
        }
    }
}

pub struct NodeRef {
    index : uint,
    node_list : @[Node],
}

impl NodeRef {
    pub fn new(nodes : &[Node]) -> ~NodeRef {
        ~NodeRef{ index : 0,
                  node_list : ::at_vec::to_managed(nodes) }
    }

    pub fn parent(&self) -> Option<~NodeRef> {
        let new_index = (self.index - 1) / 2;
        if new_index < 0 {
            None
        } else {
            Some(~NodeRef { index : new_index, node_list : self.node_list })
        }
    }

    pub fn left_child(&self) -> Option<~NodeRef> {
        let new_index = 1 + 2 * self.index;
        if new_index < self.node_list.len() {
            Some(~NodeRef { index : new_index,
                            node_list : self.node_list })
        } else {
            None
        }
    }

    pub fn right_child(&self) -> Option<~NodeRef> {
        let new_index = 2 + 2 * self.index;
        if new_index < self.node_list.len() {
            Some(~NodeRef { index : new_index, node_list : self.node_list })
        } else {
            None
        }
    }

    pub fn eval(&self) -> bool {
        debug!("evaluating node with index {}", self.index);
        let node = &self.node_list[self.index];
        match *node {
            Leaf{value : v} => v,
            Interior{ gate : And, changeable : _ } =>
                self.left_child().unwrap().eval() &&
                self.right_child().unwrap().eval(),
            Interior{ gate : Or, changeable : _ } => 
                self.left_child().unwrap().eval() ||
                self.right_child().unwrap().eval(),
        }
    }

    pub fn min_changes(&self, desired : bool) -> Option<uint> {
        let cur_val = self.eval();
        if cur_val == desired {
            return Some(0);
        }

        let node = &self.node_list[self.index];
        match *node {
            Leaf{value : _} => None,
            Interior{ gate : op, changeable : false} => {
                let left_val = self.left_child().unwrap().eval();
                let right_val = self.right_child().unwrap().eval();
                match (desired, op, left_val, right_val) {
                    (false, Or, true, true) =>
                        NodeRef::add(
                        &self.left_child().unwrap().min_changes(desired),
                        &self.right_child().unwrap().min_changes(desired)),
                    (false, Or, false, true) =>
                        self.right_child().unwrap().min_changes(desired),
                    (false, Or, true, false) =>
                        self.left_child().unwrap().min_changes(desired),
                    (true, Or, false, false) =>
                        NodeRef::min_option(
                        &self.left_child().unwrap().min_changes(desired),
                        &self.right_child().unwrap().min_changes(desired)),

                    (true, And, false, false) =>
                        NodeRef::add(
                        &self.left_child().unwrap().min_changes(desired),
                        &self.right_child().unwrap().min_changes(desired)),
                    (true, And, true, false) =>
                        self.right_child().unwrap().min_changes(desired),
                    (true, And, false, true) =>
                        self.left_child().unwrap().min_changes(desired),
                    (false, And, true, true) =>
                        NodeRef::min_option(
                        &self.left_child().unwrap().min_changes(desired),
                        &self.right_child().unwrap().min_changes(desired)),

                    _ => fail!("Impossible")
                }
            },
            Interior{ gate : op, changeable : true} => {
                let left_val = self.left_child().unwrap().eval();
                let right_val = self.right_child().unwrap().eval();
                match (desired, op, left_val, right_val) {
                    (false, Or, false, _) => Some(1),
                    (false, Or, _, false) => Some(1),
                    (true, Or, false, false) =>
                        NodeRef::min_option(
                        &self.left_child().unwrap().min_changes(desired),
                        &self.right_child().unwrap().min_changes(desired)),
                    (false, Or, true, true) =>
                        NodeRef::add_one(
                        &NodeRef::min_option(
                            &self.left_child().unwrap().min_changes(desired),
                            &self.right_child().unwrap().min_changes(desired))),
                    (true, And, true, _) => Some(1),
                    (true, And, _, true) => Some(1),
                    (false, And, true, true) =>
                        NodeRef::min_option(
                        &self.left_child().unwrap().min_changes(desired),
                        &self.right_child().unwrap().min_changes(desired)),
                    (true, And, false, false) =>
                        NodeRef::add_one(
                        &NodeRef::min_option(
                            &self.left_child().unwrap().min_changes(desired),
                            &self.right_child().unwrap().min_changes(desired))),
                    _ => fail!("Impossible combination.")
                }
            }
        }
    }

    fn min_option(v1 : &Option<uint>, v2 : &Option<uint>) -> Option<uint> {
        match (v1, v2) {
            (&None, &None) => None,
            (&Some(v), &None) => Some(v),
            (&None, &Some(v)) => Some(v),
            (&Some(v1), &Some(v2)) if v1 <= v2 => Some(v1),
            (&Some(_), &Some(v2)) => Some(v2)
        }
    }

    fn add_one(v : &Option<uint>) -> Option<uint> {
        match v {
            &None => None,
            &Some(v) => Some(1 + v)
        }
    }

    fn add(v1 : &Option<uint>, v2 : &Option<uint>) -> Option<uint> {
        match (v1, v2) {
            (&None, _) => None,
            (_, &None) => None,
            (&Some(a), &Some(b)) => Some(a + b),
        }
    }
}

fn main() {
    let args = os::args();
    assert_eq!(2, args.len());

    let path = Path::new(args[1]);
    let mut scanner = scanner::Scanner::new_from_reader(File::open(&path));

    let test_count = scanner.next_uint().unwrap();

    for test in range(1, 1 + test_count) {
        let node_count = scanner.next_uint().unwrap();
        let desired_value = 1 == scanner.next_uint().unwrap();
        let mut nodes : ~[Node] = ~[];

        for _ in range(0, node_count / 2) {
            let gate = match scanner.next_uint().unwrap() {
                1 => And,
                _ => Or
            };
            let changeable = 1 == scanner.next_uint().unwrap();
            nodes.push(Interior{gate: gate, changeable: changeable});
        }

        for _ in range(0, 1 + node_count / 2) {
            let value = scanner.next_uint().unwrap();
            nodes.push(Leaf{value: 1 == value});
        }

        let tree = NodeRef::new(nodes);

        let min = tree.min_changes(desired_value);

        print!("Case \\#{}: ", test);
        match min {
            None => { println!("IMPOSSIBLE"); }
            Some(v) => { println!("{}", v); }
        }
    }
}
