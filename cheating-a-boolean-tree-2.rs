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
    Interior { gate : Gate,
               changeable : bool,
               left : ~Node,
               right : ~Node,
               cached_value : Option<bool> },
    TempPlaceHolder,
}

impl Node {
    pub fn eval(&mut self) -> bool {
        match self {
            &Leaf{ value } => value,
            &Interior{ cached_value: Some(v),
                       gate: _, left: _, right: _, changeable: _} => v,
            &Interior{ gate,
                       left: ref mut left,
                       right: ref mut right,
                       cached_value: ref mut cached,
                       changeable: _} => {
                let my_val = match gate {
                    And => left.eval() && right.eval(),
                    Or => left.eval() || right.eval(),
                };
                *cached = Some(my_val);
                my_val
            }
            _ => {
                unreachable!();
            }
        }
    }

    pub fn min_changes(&mut self, desired : bool) -> Option<uint> {
        if self.eval() == desired {
            return Some(0);
        }

        match self {
            &Leaf{ value : _ } => None,
            &Interior{ changeable : false,
                       left: ref mut left,
                       right: ref mut right,
                       gate,
                       cached_value: _ } => {
                match (desired, gate, left.eval(), right.eval()) {
                    (false, Or, true, true) =>
                        add_options(&left.min_changes(desired),
                                    &right.min_changes(desired)),
                    (false, Or, false, true) =>
                        right.min_changes(desired),
                    (false, Or, true, false) =>
                        left.min_changes(desired),
                    (true, Or, false, false) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),

                    (true, And, false, false) =>
                        add_options(&left.min_changes(desired),
                                    &right.min_changes(desired)),
                    (true, And, true, false) =>
                        right.min_changes(desired),
                    (true, And, false, true) =>
                        left.min_changes(desired),
                    (false, And, true, true) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),

                    _ => unreachable!()
                }
            },

            &Interior{ changeable : true,
                       left: ref mut left,
                       right: ref mut right,
                       gate,
                       cached_value: _} => {
                match (desired, gate, left.eval(), right.eval()) {
                    (false, Or, false, _) => Some(1),
                    (false, Or, _, false) => Some(1),
                    (true, Or, false, false) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),
                    (false, Or, true, true) =>
                        inc_option(
                        &min_option(&left.min_changes(desired),
                                    &right.min_changes(desired))),
                    (true, And, true, _) => Some(1),
                    (true, And, _, true) => Some(1),
                    (false, And, true, true) =>
                        min_option(&left.min_changes(desired),
                                   &right.min_changes(desired)),
                    (true, And, false, false) =>
                        inc_option(
                        &min_option(&left.min_changes(desired),
                                    &right.min_changes(desired))),
                    _ => unreachable!()
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
        (&Some(v1), &Some(v2)) if v1 <= v2 => Some(v1),
        (&Some(_), &Some(v2)) => Some(v2)
    }
}

fn inc_option(v : &Option<uint>) -> Option<uint> {
    match v {
        &None => None,
        &Some(v) => Some(1 + v)
    }
}

fn add_options(v1 : &Option<uint>, v2 : &Option<uint>) -> Option<uint> {
    match (v1, v2) {
        (&None, _) => None,
        (_, &None) => None,
        (&Some(a), &Some(b)) => Some(a + b),
    }
}


fn main() {
    let args = os::args();
    assert_eq!(2, args.len());

    let path = Path::new(args[1]);
    let mut scanner = scanner::Scanner::new_from_reader(File::open(&path));

    let test_count = scanner.next_uint().unwrap();
    let mut nodes : ~[~Node] = ~[];

    for test in range(1, 1 + test_count) {
        let node_count = scanner.next_uint().unwrap();
        let desired_value = 1 == scanner.next_uint().unwrap();
        let mut interior_queue : ~[Node] = ~[];

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

        for _ in range(0, 1 + node_count / 2) {
            let value = scanner.next_uint().unwrap();
            nodes.push(~Leaf{value: 1 == value});
        }

        for index in range(1, node_count).invert() {
            let node : ~Node = nodes.pop();
            let parent_index = (index - 1) / 2;
            let parent = &mut nodes[parent_index];
            match parent {
                &~Interior{left: ref mut left,
                           right:_, cached_value:_, gate:_, changeable:_}
                if 1 == index %2 => {
                    *left = node;
                }
                &~Interior{right: ref mut right,
                           left:_, cached_value:_, gate:_, changeable:_} => {
                    *right = node;
                }
                _ => {
                    unreachable!();
                }
            }
        }

        let mut root = nodes.pop();
        let value = root.eval();

        let min = root.min_changes(desired_value);

        print!("Case \\#{}: ", test);
        match min {
            None => { println!("IMPOSSIBLE"); }
            Some(v) => { println!("{}", v); }
        }
    } // test loop
} // main fn

