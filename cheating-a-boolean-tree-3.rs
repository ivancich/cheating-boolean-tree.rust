#[feature(struct_variant)];
#[feature(managed_boxes)];

use std::os;
use std::io::fs::File;

use std::option::Option;

mod scanner;

enum Gate {
    And,
    Or
}

pub enum Node {
    Leaf { value: bool },
    Interior { gate : Gate,
               changeable : bool,
               left : ~Node,
               right : ~Node,
               cached_value : Option<bool> },
    TempPlaceHolder,
}

impl Node {
    pub fn value(&self) -> bool {
        match self {
            &Leaf{ value } => value,
            &Interior{ cached_value: Some(v),
                       gate: _, left: _, right: _, changeable: _} => v,
            _ => {
                unreachable!();
            }
        }
    }

    pub fn min_changes(&self, desired : bool) -> Option<uint> {
        if self.value() == desired {
            return Some(0);
        }

        match self {
            &Leaf{ value : _ } => None,
            &Interior{ left: ref left,
                       right: ref right,
                       changeable, gate,
                       cached_value: _ } => {
                match (desired, gate, changeable) {

                    // not changeable

                    (false, Or, false) =>
                        add_options(&left.min_changes(desired),
                                    &right.min_changes(desired)),
                    (true, Or, false) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),
                    (true, And, false) =>
                        add_options(&left.min_changes(desired),
                                    &right.min_changes(desired)),
                    (false, And, false) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),

                    // changeable

                    (true, Or, true) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),
                    (false, Or, true) =>
                        inc_option(&min_option(&left.min_changes(desired),
                                               &right.min_changes(desired))),
                    (false, And, true) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),
                    (true, And, true) =>
                        inc_option(&min_option(&left.min_changes(desired),
                                               &right.min_changes(desired))),
                }
            },

            _ => unreachable!()
        }
    }
}


fn min_option(v1 : &Option<uint>, v2 : &Option<uint>) -> Option<uint> {
    match (v1, v2) {
        (&None, &None) => None,
        (&Some(v), &None) => Some(v),
        (&None, &Some(v)) => Some(v),
        (&Some(v1), &Some(v2)) if v1 < v2 => Some(v1),
        (&Some(_), &Some(v2)) => Some(v2),
    }
}

/*
fn min_option<T: Ord, Clone>(v1 : &Option<T>, v2 : &Option<T>) -> Option<T> {
    match (v1, v2) {
        (&None, &None) => None,
        (&Some(ref v), &None) => Some(*v.clone()),
        (&None, &Some(ref v)) => Some(*v.clone()),
        (&Some(ref v1), &Some(ref v2)) if v1.le(v2) => Some(*v1.clone()),
        (&Some(_), &Some(ref v2)) => Some(*v2.clone())
    }
}
*/

fn inc_option<T: Add<uint,T>>(v : &Option<T>) -> Option<T> {
    match v {
        &None => None,
        &Some(ref v) => Some(*v + 1)
    }
}


fn add_options<T: Add<T,T>>(v1 : &Option<T>, v2 : &Option<T>) -> Option<T> {
    match (v1, v2) {
        (&None, _) => None,
        (_, &None) => None,
        (&Some(ref a), &Some(ref b)) => Some(*a + *b),
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
        let mut nodes : ~[~Node] = ~[];

        // read in the interior nodes
        for _ in range(0, node_count / 2) {
            let gate = match scanner.next_uint().unwrap() {
                1 => And,
                _ => Or
            };
            let changeable = 1 == scanner.next_uint().unwrap();
            nodes.push(~Interior{gate: gate,
                                 changeable: changeable,
                                 cached_value: None,
                                 left: ~TempPlaceHolder,
                                 right: ~TempPlaceHolder});
        }

        // read in the leaf nodes
        for _ in range(0, 1 + node_count / 2) {
            let value = scanner.next_uint().unwrap();
            nodes.push(~Leaf{value: 1 == value});
        }

        // Move backwards through the vector connecting a node to its
        // parent (either left or right reference). When we've filled
        // out an interior node's left child, we know that the right
        // child has already been filled out, so we can compute the
        // cached value.
        for index in range(1, node_count).invert() {
            let node : ~Node = nodes.pop();
            let parent_index = (index - 1) / 2;
            let parent = &mut nodes[parent_index];
            match parent {
                // connect as parent's left child (index is odd)
                &~Interior{left: ref mut left,
                           right: ref right,
                           cached_value: ref mut cached_value,
                           gate,
                           changeable:_} if 1 == index %2 => {
                    *left = node;

                    // compute cached value
                    match gate {
                        And => {
                            *cached_value =
                                Some(left.value() && right.value());
                        },
                        Or => {
                            *cached_value =
                                Some(left.value() || right.value());
                        }
                    }
                }

                // connect as parent's right child
                &~Interior{right: ref mut right,
                           left:_, cached_value:_, gate:_, changeable:_} => {
                    *right = node;
                }

                _ => unreachable!()
            }
        }

        let root = nodes.pop();
        let min = root.min_changes(desired_value);

        print!("Case \\#{}: ", test);
        match min {
            None => { println!("IMPOSSIBLE"); }
            Some(v) => { println!("{}", v); }
        }
    } // test loop
} // main fn

