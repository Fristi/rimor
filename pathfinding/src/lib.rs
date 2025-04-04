use std::collections::BinaryHeap;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug)]
struct PathfindingState {
    pub score: u32,
    pub timesteps_remaining: u32,
    pub node: (usize, usize)
}

#[derive(Debug, Clone, Copy)]
pub struct PathfindingStep {
    pub node: (usize, usize),
    pub score: u32,
    pub step: u32
}

#[derive(Debug, Clone)]
pub struct PathfindingResult {
    pub path: Vec<PathfindingStep>
}

impl PathfindingResult {
    pub fn empty() -> PathfindingResult {
        PathfindingResult { path: Vec::new() }
    }

    pub fn steps_at(&self, node: (usize, usize)) -> Vec<&PathfindingStep> {
        self.path.iter().filter(|&&n| n.node == node).collect()
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
    nodes: Vec<Vec<u32>>,
    size: usize
}

impl Graph {
    pub fn new(size: usize) -> Self {
        Graph {
            nodes: vec![vec![0; size]; size],
            size: size
        }
    }

    pub fn add_node(&mut self, u: (usize, usize), score: u32) {
        self.nodes[u.0][u.1] = score;
    }

    pub fn get_score_at(&self, u: (usize, usize)) -> &u32 {
        &self.nodes[u.0][u.1]
    }

    pub fn reset_score(&self, u: (usize, usize)) -> Graph {
        let mut graph = self.clone();
        graph.nodes[u.0][u.1] = 0;
        graph
    }

    pub fn from_file(path: &Path) -> Self {
        let contents = read_to_string(path).expect("Unable to load file");
        let lines = contents.lines().enumerate();
        let grid_size = lines.clone().count();
        let mut graph = Graph::new(grid_size);
        for (i, line) in lines {
            for (j, c) in line.split(" ").enumerate() {
                graph.add_node((i, j), c.parse().expect("Unable to parse integer"));
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
                graph.add_node((i, j), c.parse().expect("Unable to parse integer"));
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
                    graph.nodes[i][j] += recovery_rate;
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
        let mut step = 0;

        pq.push(PathfindingState {
            score: graph.get_score_at(start).clone(),
            timesteps_remaining: max_timesteps,
            node: start
        });

        while let Some(state) = pq.pop() {

            if state.timesteps_remaining == 0 {
                break;
            }

            let score_ = graph.get_score_at(state.node);

            score += score_;
            step += 1;

            path.push(PathfindingStep { node: state.node, score: *score_, step: step });

            graph = graph
                .reset_score(state.node)
                .recover_for(recovery_rate, state.node);


            pq.clear();

            for neighbor in graph.get_neighbors(state.node) {
                let neighbor_score = graph.get_score_at(neighbor);

                pq.push(PathfindingState {
                    node: neighbor,
                    score: *neighbor_score,
                    timesteps_remaining: state.timesteps_remaining - 1,
                });
            }
        }

        PathfindingResult { path }
    }

    pub fn path_planning_dfs(&self, start: (usize, usize), max_timesteps: u32, recovery_rate: u32) -> PathfindingResult {

        unimplemented!()
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