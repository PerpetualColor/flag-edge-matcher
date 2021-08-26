pub mod shared {
    use serde::{Serialize, Deserialize};
    use std::collections::HashMap;
    use std::fmt;

    pub type EdgeInfo = Vec<(String, u32)>;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FlagEdges {
        pub id: String,
        pub top: EdgeInfo,
        pub right: EdgeInfo,
        pub bottom: EdgeInfo,
        pub left: EdgeInfo,
    }
    
    #[derive(Debug, Copy, Clone)]
    pub enum Sides {
        TOP,
        RIGHT,
        BOTTOM,
        LEFT
    }

    impl Sides {
        pub fn opposite(&self) -> Self {
            use Sides::*;
            match self {
                TOP => BOTTOM,
                RIGHT => LEFT,
                BOTTOM => TOP,
                LEFT => RIGHT
            }
        }

        pub fn offset(&self) -> (i32, i32) {
            use Sides::*;
            match self {
                TOP => (0, 1),
                RIGHT => (1, 0),
                BOTTOM => (0, -1),
                LEFT => (-1, 0)
            }
        }
    }

    
    #[derive(Clone, Debug)]
    pub struct FlagGraph {
        pub graph: HashMap<(i32, i32), String>,
        pub remaining_flags: HashMap<String, u32>,
        pub idx: u32,
    }
    
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct FlagGraphSerialize {
        graph: Vec<((i32, i32), String)>,
        remaining_flags: HashMap<String, u32>,
        idx: u32,
    }

    impl FlagGraphSerialize {
        pub fn new(flag_graph: &FlagGraph) -> FlagGraphSerialize {
            FlagGraphSerialize {
                graph: flag_graph.graph.iter().map(|v| ((v.0.0, v.0.1), v.1.to_string())).collect(),
                remaining_flags: flag_graph.remaining_flags.clone(),
                idx: flag_graph.idx,
            }
        }

        pub fn to_flag_graph(&self) -> FlagGraph {
            let mut graph_map = HashMap::new();
            for cell in &self.graph {
                graph_map.insert(cell.0, cell.1.clone());
            }
            FlagGraph {
                graph: graph_map,
                remaining_flags: self.remaining_flags.clone(),
                idx: self.idx
            }
        }
    }

    
    #[derive(Clone, Serialize, Deserialize)]
    pub struct MultiFlag {
        pub id: String,
        pub top: String,
        pub right: String,
        pub bottom: String,
        pub left: String,
    }

    impl MultiFlag {
        pub fn side(&self, side: Sides) -> &str {
            use Sides::*;
            match side {
                TOP => &self.top,
                RIGHT => &self.right,
                BOTTOM => &self.bottom,
                LEFT => &self.left,
            }
        }
    }

    impl fmt::Display for MultiFlag {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.id)
        }
    }

    impl fmt::Debug for MultiFlag {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.id)
        }
    }

}