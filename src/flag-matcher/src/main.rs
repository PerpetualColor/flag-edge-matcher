use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use rand::thread_rng;
use rand::seq::SliceRandom;

use shared::shared::*;

fn edge_to_id(edge: &EdgeInfo) -> String {
    let mut output = String::new();
    for segment in edge {
        output += &segment.0;
        output += &segment.1.to_string();
    }
    output
}

fn edges_into_id(flag: &FlagInfo) -> String {
    let mut output = String::new();
    output += &flag.top;
    output += ","; 
    output += &flag.right;
    output += ",";
    output += &flag.bottom;
    output += "," ;
    output += &flag.left;

    output
}

fn read_edge_data_from_file<P: AsRef<Path>>(path: P) -> Vec<FlagEdges> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let output = serde_json::from_reader(reader).unwrap();

    output
}

// no longer used
// fn idx_to_coords(idx: u32) -> (i32, i32) {
//     let idx = idx as i32;
//     let shell = (idx as f32).sqrt().ceil() as i32;
//     let off = shell * shell - idx;
//     let y = std::cmp::min(shell - 1, off);
//     let x_off = std::cmp::max(0, shell * shell - idx - (shell - 1));
//     let x = shell - 1 - x_off;
//     (x, y)
// }

#[derive(Clone)]
struct FlagInfo {
    id: String,
    top: String,
    right: String,
    bottom: String,
    left: String
}

struct EdgeData {
    top: HashMap<String, Vec<String>>,
    right: HashMap<String, Vec<String>>,
    bottom: HashMap<String, Vec<String>>,
    left: HashMap<String, Vec<String>>,
}

impl EdgeData {
    fn side(&self, side: Sides) -> &HashMap<String, Vec<String>> {
        use Sides::*;
        match side {
            TOP => &self.top,
            RIGHT => &self.right,
            BOTTOM => &self.bottom,
            LEFT => &self.left,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BoundaryCell {
    loc: (i32, i32),
    from: Sides,
}

impl PartialEq for BoundaryCell {
    fn eq(&self, other: &Self) -> bool {
        self.loc == other.loc
    }
}

impl Eq for BoundaryCell {}

impl Hash for BoundaryCell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.loc.hash(state);
    }
}

fn dist_sq(l1: (i32, i32)) -> i32 {
    l1.0.pow(2) + l1.1.pow(2)
}

fn add_next_states(state: &FlagGraph, next_states: &mut VecDeque<FlagGraph>, edge_data: &EdgeData, multi_flags: &HashMap<String, MultiFlag>) -> i32 {
    let mut states_added = 0;

    let gen_new_state = |location: (i32, i32), add_id: String| {
        let mut next_state = state.clone();
        *next_state.remaining_flags.get_mut(&add_id).unwrap() -= 1;
        if *next_state.remaining_flags.get(&add_id).unwrap() == 0 {
            next_state.remaining_flags.remove(&add_id);
        }
        next_state.idx += 1;
        next_state.graph.insert(location, add_id);

        next_state
    };

    let side_matches = |loc: (i32, i32), side: Sides, id: &String| {
        use Sides::*;
        let check_loc = match side {
            TOP => (loc.0, loc.1 + 1),
            RIGHT => (loc.0 + 1, loc.1),
            BOTTOM => (loc.0, loc.1 - 1),
            LEFT => (loc.0 - 1, loc.1),
        };

        let check_value = state.graph.get(&check_loc);
        if check_value.is_none() {
            return true;
        } else {
            let check_value = check_value.unwrap();
            match side.opposite() {
                TOP => &multi_flags.get(check_value).unwrap().top == id,
                RIGHT => &multi_flags.get(check_value).unwrap().right == id,
                BOTTOM => &multi_flags.get(check_value).unwrap().bottom == id,
                LEFT => &multi_flags.get(check_value).unwrap().left == id
            }
        }
    };

    // locate all boundary cells
    let mut boundary_cells = HashMap::new();
    
    for cell in &state.graph {
        use Sides::*;
        for side in &[TOP, RIGHT, LEFT, BOTTOM] {
            let offset = side.offset();
            let new_loc = (cell.0.0 + offset.0, cell.0.1 + offset.1);
            if !state.graph.contains_key(&new_loc) {
                if !boundary_cells.contains_key(&BoundaryCell {
                    loc: new_loc,
                    from: Sides::BOTTOM,
                }) {
                    boundary_cells.insert(BoundaryCell {
                        loc: new_loc,
                        from: side.opposite(),
                    }, 1);
                } else {
                    *boundary_cells.get_mut(&BoundaryCell {
                        loc: new_loc,
                        from: Sides::BOTTOM,
                    }).unwrap() += 1;
                }
            }
        }
    }

    let mut place_flag_at_loc = |id: &String, edge_data: &HashMap<String, Vec<String>>, next_states: &mut VecDeque<FlagGraph>, loc: (i32, i32)| {
        if edge_data.contains_key(id) {
            let mut cur_edge_data = edge_data.get(id).unwrap().clone();
            cur_edge_data.shuffle(&mut thread_rng());
            for flag_id in &cur_edge_data {
                if state.remaining_flags.contains_key(flag_id) {
                    let flag = multi_flags.get(flag_id).unwrap();
                    if side_matches(loc, Sides::TOP, &flag.top) &&
                        side_matches(loc, Sides::RIGHT, &flag.right) &&
                        side_matches(loc, Sides::BOTTOM, &flag.bottom) &&
                        side_matches(loc, Sides::LEFT, &flag.left)
                    {
                        next_states.push_back(gen_new_state(loc, String::from(flag_id)));
                        states_added += 1;
                    }
                }
            }
        }
    };

    let mut boundary_iter = boundary_cells.iter().collect::<Vec<(&BoundaryCell, &u32)>>();
    boundary_iter.shuffle(&mut thread_rng());
    // sorts by number of edges
    // boundary_iter.sort_by(|b1: &(&BoundaryCell, &u32), b2: &(&BoundaryCell, &u32)| b1.1.partial_cmp(&b2.1).unwrap());

    // sorts by distance to (0, 0)
    boundary_iter.sort_by(|b1: &(&BoundaryCell, &u32), b2: &(&BoundaryCell, &u32)| dist_sq(b2.0.loc).partial_cmp(&dist_sq(b1.0.loc)).unwrap());

    for (boundary, _) in boundary_iter {
        let from_offset = (boundary.loc.0 + boundary.from.offset().0, boundary.loc.1 + boundary.from.offset().1);

        place_flag_at_loc(
            &multi_flags.get(state.graph.get(&from_offset).unwrap()).unwrap().side(boundary.from.opposite()).to_string(),
            &edge_data.side(boundary.from), next_states, boundary.loc
        );
    }
    return states_added;
}

fn save_graph_to_file(flag_graph: &FlagGraph) {
    let mut output_file = File::create("best_graph_found_".to_string() + &flag_graph.idx.to_string() + ".json").unwrap();
    output_file.write_all(serde_json::to_string(&FlagGraphSerialize::new(&flag_graph)).unwrap().as_bytes()).unwrap();
}

fn save_multi_flags_to_file(multi_flags: &HashMap<String, HashSet<String>>) {
    let mut output_file = File::create("multi_flags.json").unwrap();
    output_file.write_all(serde_json::to_string(multi_flags).unwrap().as_bytes()).unwrap();
}

fn generate_flag_arrangement(multi_flags_count: &HashMap<String, u32>, flags_to_multi_flags: &HashMap<String, String>, edge_data: &EdgeData, multi_flags: &HashMap<String, MultiFlag>, start_id: String) -> Option<FlagGraph> {
    let mut initial_state = FlagGraph {
        graph: HashMap::new(),
        remaining_flags: multi_flags_count.clone(),
        idx: 1
    };

    let initial_flag = flags_to_multi_flags.get(&start_id).unwrap().to_string();
    *initial_state.remaining_flags.get_mut(&initial_flag).unwrap() -= 1;
    if *initial_state.remaining_flags.get(&initial_flag).unwrap() == 0 {
        initial_state.remaining_flags.remove(&initial_flag);
    }
    initial_state.graph.insert((0, 0), initial_flag);

    let mut next_states = VecDeque::new();

    add_next_states(&initial_state, &mut next_states, &edge_data, &multi_flags);

    let mut best_result: Option<FlagGraph> = None;
    let mut i = 0;
    while !next_states.is_empty() {
        let s = next_states.pop_back().unwrap();
        let new_states = add_next_states(&s, &mut next_states, &edge_data, &multi_flags);

        if new_states == 0 {
            if best_result.is_none() {
                best_result = Some(s.clone());
                save_graph_to_file(&s);
                println!("New best found: ");
                println!("{:?} ({} flags)", s.graph, s.idx);
                i -= 1000;
            } else {
                let prev_best = best_result.unwrap();
                if prev_best.idx < s.idx {
                    best_result = Some(s.clone());
                    save_graph_to_file(&s);
                    println!("New best found: ");
                    println!("{:?} ({} flags)", s.graph, s.idx);
                    i -= 1000;
                } else {
                    best_result = Some(prev_best);
                }
            }
        }

        i += 1;

        if i % 500 == 0 {
            println!("i: {}, states: {}", i, next_states.len());
        }

        if i >= 30000 {
            next_states.clear();

            let mut initial_state = FlagGraph {
                graph: HashMap::new(),
                remaining_flags: multi_flags_count.clone(),
                idx: 1
            };

            let initial_flag = flags_to_multi_flags.get(&start_id).unwrap().to_string();
            *initial_state.remaining_flags.get_mut(&initial_flag).unwrap() -= 1;
            if *initial_state.remaining_flags.get(&initial_flag).unwrap() == 0 {
                initial_state.remaining_flags.remove(&initial_flag);
            }
            initial_state.graph.insert((0, 0), initial_flag);

            add_next_states(&initial_state, &mut next_states, &edge_data, &multi_flags);
            i = 0;
        }

    }
    best_result
}

fn main() {
    let flag_data = read_edge_data_from_file("./flag_edges.json");

    // process flags into id strings

    let mut flags = HashMap::new();
    for f in flag_data {
        flags.insert(String::from(&f.id), FlagInfo {
            id: String::from(&f.id),
            top: edge_to_id(&f.top),
            right: edge_to_id(&f.right),
            bottom: edge_to_id(&f.bottom),
            left: edge_to_id(&f.left),
        });
    }

    let mut multi_flags_to_flags: HashMap<String, HashSet<String>> = HashMap::new();
    let mut multi_flags_count: HashMap<String, u32> = HashMap::new();
    let mut multi_flags = HashMap::new();
    let mut flags_to_multi_flags = HashMap::new();

    for f in &flags {
        let edge_id = edges_into_id(f.1);
        flags_to_multi_flags.insert(f.0.clone(), edge_id.clone());
        if multi_flags_to_flags.contains_key(&edge_id) {
            multi_flags_to_flags.get_mut(&edge_id).unwrap().insert(String::from(f.0));
            *multi_flags_count.get_mut(&edge_id).unwrap() += 1;
        } else {
            let mut new_set = HashSet::new();
            new_set.insert(String::from(f.0));
            multi_flags_to_flags.insert(String::from(&edge_id), new_set);
            multi_flags_count.insert(String::from(&edge_id), 1);
            multi_flags.insert(String::from(&edge_id), MultiFlag {
                id: edge_id,
                top: f.1.top.clone(),
                right: f.1.right.clone(),
                bottom: f.1.bottom.clone(),
                left: f.1.left.clone()
            });
        }
    }
    println!("{} flags -> {} multiflags", flags.len(), multi_flags.len());
    println!("{:?}", multi_flags_to_flags);
    println!("");
    
    let mut top_edges = HashMap::new();
    let mut right_edges = HashMap::new();
    let mut bottom_edges = HashMap::new();
    let mut left_edges = HashMap::new();

    let add_edge_to_map = |edges: &mut HashMap<String, Vec<String>>, edge_id: &String, id: &String| {
        if edges.contains_key(edge_id) {
            edges.get_mut(edge_id).unwrap().push(id.to_string());
        } else {
            edges.insert(edge_id.to_string(), vec![String::from(id)]);
        }
    };

    for mf in &multi_flags {
        add_edge_to_map(&mut top_edges, &mf.1.top, &mf.0);
        add_edge_to_map(&mut right_edges, &mf.1.right, &mf.0);
        add_edge_to_map(&mut bottom_edges, &mf.1.bottom, &mf.0);
        add_edge_to_map(&mut left_edges, &mf.1.left, &mf.0);
    }

    let edge_data = EdgeData {
        top: top_edges,
        right: right_edges,
        bottom: bottom_edges,
        left: left_edges,
    };

    save_multi_flags_to_file(&multi_flags_to_flags);
    generate_flag_arrangement(&multi_flags_count, &flags_to_multi_flags, &edge_data, &multi_flags, "sc".to_string());
}