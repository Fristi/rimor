use std::collections::BinaryHeap;
use crate::{Graph, PathfindingResult, PathfindingStep, Position};

#[derive(Debug)]
struct PathfindingBestFirstSearchState {
    pub score: u32,
    pub timesteps_remaining: u32,
    pub node: Position
}

impl Ord for PathfindingBestFirstSearchState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.partial_cmp(&self.score).unwrap().reverse()
    }
}

impl PartialOrd for PathfindingBestFirstSearchState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PathfindingBestFirstSearchState {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for PathfindingBestFirstSearchState {}


impl Graph {
    /// Does a best first search for a path from the start position with a maximum number of timesteps.
    pub fn path_planning_bfs(&self, start: Position, max_timesteps: u32, recovery_rate: u32) -> PathfindingResult {
        let mut pq = BinaryHeap::new();
        let mut path = Vec::new();
        let mut graph = self.clone();
        let mut score = 0;
        let mut step = 0;

        pq.push(PathfindingBestFirstSearchState {
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

            for neighbor in graph.get_nodes_out(state.node) {
                let neighbor_score = graph.get_score_at(neighbor);

                pq.push(PathfindingBestFirstSearchState {
                    node: neighbor,
                    score: *neighbor_score,
                    timesteps_remaining: state.timesteps_remaining - 1,
                });
            }
        }

        PathfindingResult { path }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_path_planning_bfs() {
        let graph = Graph::new(5);
        let start = (0,0);
        let max_timesteps = 10;
        let recovery_rate = 1;

        let result = graph.path_planning_bfs(start, max_timesteps, recovery_rate);

        assert_eq!(result.path.len(), 10);
    }

    #[test]
    fn test_path_planning_bfs_output() {
        let mut graph = Graph::new(3);

        let scores = [
            [((0,0), 0), ((0,1), 5), ((0,2), 5)],
            [((1,0), 5), ((1,1), 7), ((1,2), 5)],
            [((2,0), 5), ((2,1), 10), ((2,2), 5)],
        ];

        for row in 0..3 {
            for col in 0..3 {
                graph.add_node((row, col), scores[row][col].1);
            }
        }

        let start = (0,0);
        let max_timesteps = 3;
        let recovery_rate = 1;

        let result = graph.path_planning_bfs(start, max_timesteps, recovery_rate);

        assert_eq!(result, PathfindingResult {
            path: vec![
                PathfindingStep { node: (0, 0), score: 0, step: 1 },
                PathfindingStep { node: (1, 1), score: 8, step: 2 },
                PathfindingStep { node: (2, 1), score: 12, step: 3 }
            ]
        });
    }
}

