use std::collections::BinaryHeap;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub score: u32,
    pub decay_rate: u32,
    pub recovery_rate: u32,
}

impl Node {
    pub fn new() -> Self {
        Node {
            score: 0,
            decay_rate: 0,
            recovery_rate: 0
        }
    }
}

#[derive(Debug)]
struct PathfindingState {
    pub score: u32,
    pub timesteps_remaining: u32,
    pub node: (usize, usize)
}

#[derive(Debug, Clone, Copy)]
pub struct PathfindingStep {
    pub node: (usize, usize),
    pub score: u32
}

#[derive(Debug, Clone)]
pub struct PathfindingResult {
    pub path: Vec<PathfindingStep>
}

impl PathfindingResult {
    pub fn empty() -> PathfindingResult {
        PathfindingResult { path: Vec::new() }
    }

    pub fn occurs_at(&self, node: (usize, usize)) -> Option<usize> {
        self.path.iter().position(|&n| n.node == node)
    }

    pub fn score(&self) -> u32 {
        self.path.iter().map(|x| x.score).sum()
    }
}

impl Ord for PathfindingState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.partial_cmp(&self.score).unwrap().reverse()
    }
}

impl PartialOrd for PathfindingState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PathfindingState {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for PathfindingState {}


#[derive(Clone)]
pub struct Graph {
    nodes: Vec<Vec<Node>>,
    size: usize
}

impl Graph {
    pub fn new(size: usize) -> Self {
        Graph {
            nodes: vec![vec![Node::new(); size]; size],
            size: size
        }
    }

    pub fn add_node(&mut self, u: (usize, usize), node: Node) {
        self.nodes[u.0][u.1] = node;
    }

    pub fn get_node_at(&self, u: (usize, usize)) -> &Node {
        &self.nodes[u.0][u.1]
    }

    pub fn reset_score(&self, u: (usize, usize)) -> Graph {
        let mut graph = self.clone();
        graph.nodes[u.0][u.1].score = 0;
        graph
    }

    pub fn from_file(path: &Path) -> Self {
        let contents = read_to_string(path).expect("Unable to load file");
        let lines = contents.lines().enumerate();
        let grid_size = lines.clone().count();
        let mut graph = Graph::new(grid_size);
        for (i, line) in lines {
            for (j, c) in line.split(" ").enumerate() {

                let node = Node {
                    score: c.parse().expect("Unable to parse integer"),
                    decay_rate: 0,
                    recovery_rate: 0
                };
                graph.add_node((i, j), node);
            }
        }
        graph
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let contents = String::from_utf8(bytes).expect("Unable to convert bytes to string");
        let lines = contents.lines().enumerate();
        let grid_size = lines.clone().count();
        let mut graph = Graph::new(grid_size);
        for (i, line) in lines {
            for (j, c) in line.split(" ").enumerate() {

                let node = Node {
                    score: c.parse().expect("Unable to parse integer"),
                    decay_rate: 0,
                    recovery_rate: 0
                };
                graph.add_node((i, j), node);
            }
        }
        graph
    }

    pub fn get_neighbors(&self, u: (usize, usize)) -> Vec<(usize, usize)> {
        let (i, j) = u;
        let size = self.size;
        let mut neighbors = Vec::new();

        for (di, dj) in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)].iter() {
            let ni = i as isize + di;
            let nj = j as isize + dj;

            if ni >= 0 && ni < size as isize && nj >= 0 && nj < size as isize {
                neighbors.push((ni as usize, nj as usize));
            }
        }

        neighbors
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn recover_for(&self, recovery_rate: u32, except: (usize, usize)) -> Graph {
        let mut graph = self.clone();

        for i in 0..self.size() {
            for j in 0..self.size() {
                if (i, j) != except {
                    graph.nodes[i][j].score += recovery_rate;
                }
            }
        }
        graph
    }

    pub fn path_planning_bfs(&self, start: (usize, usize), max_timesteps: u32, recovery_rate: u32) -> PathfindingResult {
        let mut pq = BinaryHeap::new();
        let mut path = Vec::new();
        let mut graph = self.clone();
        let mut score = 0;

        pq.push(PathfindingState {
            score: graph.get_node_at(start).score,
            timesteps_remaining: max_timesteps,
            node: start
        });

        while let Some(state) = pq.pop() {

            if state.timesteps_remaining == 0 {
                break;
            }

            let node = graph.get_node_at(state.node);

            score += node.score;

            path.push(PathfindingStep { node: state.node, score: node.score });

            graph = graph
                .reset_score(state.node)
                .recover_for(recovery_rate, state.node);


            pq.clear();

            for neighbor in graph.get_neighbors(state.node) {
                let neighbor_node = graph.get_node_at(neighbor);

                pq.push(PathfindingState {
                    node: neighbor,
                    score: neighbor_node.score,
                    timesteps_remaining: state.timesteps_remaining - 1,
                });
            }
        }

        PathfindingResult { path }
    }

    pub fn path_planning_dfs(&self, start: (usize, usize), max_timesteps: u32, recovery_rate: u32) -> PathfindingResult {

        #[derive(Debug, PartialEq)]
        struct State {
            node: (usize, usize),
            score: u32,
            hops_remaining: u32,
            path: Vec<(usize, usize)>,
        }

        impl Ord for State {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other.score.partial_cmp(&self.score).unwrap() // Max heap based on score
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Eq for State {}

        let mut pq = BinaryHeap::new();
        let mut best_path = Vec::new();
        let mut graph = self.clone();

        pq.push(State { node: start, score: graph.get_node_at(start).score, hops_remaining: max_timesteps, path: vec![start] });

        while let Some(state) = pq.pop() {
            if state.hops_remaining == 0 {
                if state.score > graph.get_node_at(*best_path.last().unwrap_or(&start)).score {
                    best_path = state.path.clone();
                }
                continue;
            }

            // let node = graph.nodes.get_mut(&state.node).unwrap();
            // node.score *= node.decay_rate;

            let mut candidates = Vec::new();
            for neighbor in graph.get_neighbors(state.node) {
                let neighbor_node = &graph.get_node_at(neighbor);
                let new_score = neighbor_node.score * recovery_rate;
                let mut new_path = state.path.clone();
                new_path.push(neighbor);

                candidates.push(State {
                    node: neighbor,
                    score: new_score,
                    hops_remaining: state.hops_remaining - 1,
                    path: new_path,
                });
            }

            candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

            for candidate in candidates.into_iter().take(10) {
                pq.push(candidate);
            }
        }

        PathfindingResult { path: best_path.into_iter().map(|node| PathfindingStep { node, score: graph.get_node_at(node).score }).collect() }
    }



}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_top_left() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((0, 0)), vec![(0, 1), (1, 0), (1, 1)]);
    }

    #[test]
    fn test_top_right() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((0, 2)), vec![(0, 1), (1, 1), (1, 2)]);
    }

    #[test]
    fn test_top_middle() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((0, 1)), vec![(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)]);
    }

    #[test]
    fn test_middle_left() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((1, 0)), vec![(0, 0), (0, 1), (1, 1), (2, 0), (2, 1)]);
    }

    #[test]
    fn test_middle_right() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((1, 2)), vec![(0, 1), (0, 2), (1, 1), (2, 1), (2, 2)]);
    }

    #[test]
    fn test_middle_center() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((1, 1)), vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1), (2, 2)]);
    }

    #[test]
    fn test_bottom_left() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((2, 0)), vec![(1, 0), (1, 1), (2, 1)]);
    }

    #[test]
    fn test_bottom_right() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((2, 2)), vec![(1, 1), (1, 2), (2, 1)]);
    }

    #[test]
    fn test_bottom_middle() {
        let graph = Graph::new(3);
        assert_eq!(graph.get_neighbors((2, 1)), vec![(1, 0), (1, 1), (1, 2), (2, 0), (2, 2)]);
    }
}