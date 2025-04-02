use std::collections::{BinaryHeap, HashMap};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use good_lp::*;

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

#[derive(Debug, Copy, Clone)]
pub struct PathfindingStep {
    pub node: (usize, usize),
    pub score: u32
}

pub struct PathfindingResult {
    pub score: u32,
    pub path: Vec<PathfindingStep>
}

impl PathfindingResult {
    pub fn empty() -> PathfindingResult {
        PathfindingResult { score: 0, path: Vec::new() }
    }

    pub fn occurs_at(&self, node: (usize, usize)) -> Option<usize> {
        self.path.iter().position(|&n| n.node == node)
    }
}

impl Ord for PathfindingState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.partial_cmp(&self.score).unwrap().reverse() // Max heap based on score
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
    edges: HashMap<(usize, usize), Vec<(usize, usize)>>
}

impl Graph {
    pub fn new(size: usize) -> Self {
        let mut graph = Graph {
            nodes: vec![vec![Node::new(); size]; size],
            edges: HashMap::new()
        };

        for i in 0..size {
            for j in 0..size {
                for di in -1..=1 {
                    for dj in -1..=1 {
                        if di == 0 && dj == 0 {
                            continue; // Skip self
                        }

                        let ni = i as isize + di;
                        let nj = j as isize + dj;

                        if ni >= 0 && ni < size as isize && nj >= 0 && nj < size as isize {
                            graph.add_edge((i, j), (ni as usize, nj as usize));
                        }
                    }
                }
            }
        }

        graph
    }

    pub fn add_edge(&mut self, u: (usize, usize), v: (usize, usize)) {
        self.edges.entry(u).or_insert(Vec::new()).push(v);
    }

    pub fn add_node(&mut self, u: (usize, usize), node: Node) {
        self.nodes[u.0][u.1] = node;
    }

    pub fn get_node_at(&self, u: (usize, usize)) -> &Node {
        &self.nodes[u.0][u.1]
    }

    pub fn get_node_at_mut(&mut self, u: (usize, usize)) -> &mut Node {
        &mut self.nodes[u.0][u.1]
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

    pub fn get_neighbors(&self, u: (usize, usize)) -> Option<&Vec<(usize, usize)>> {
        self.edges.get(&u)
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn recover_for(&mut self, recovery_rate: u32, except: (usize, usize)) {
        for i in 0..self.size() {
            for j in 0..self.size() {
                if (i, j) != except {
                    self.nodes[i][j].score += recovery_rate;
                }
            }
        }
    }

    pub fn path_planning_bfs(&mut self, start: (usize, usize), max_timesteps: u32, recovery_rate: u32) -> PathfindingResult {
        let mut pq = BinaryHeap::new();
        let mut path = Vec::new();
        let mut score = 0;

        pq.push(PathfindingState {
            score: self.get_node_at(start).score,
            timesteps_remaining: max_timesteps,
            node: start
        });

        while let Some(state) = pq.pop() {

            if state.timesteps_remaining == 0 {
                break;
            }

            let node = self.get_node_at_mut(state.node);

            score += node.score;

            path.push(PathfindingStep { node: state.node, score: node.score });

            node.score *= 0;

            self.recover_for(recovery_rate, state.node);

            // Add neighbors to queue
            if let Some(neighbors) = self.get_neighbors(state.node) {

                // clear the queue, such that old nodes are not considered
                pq.clear();

                for &neighbor in neighbors {
                    let neighbor_node = self.get_node_at(neighbor);

                    pq.push(PathfindingState {
                        node: neighbor,
                        score: neighbor_node.score,
                        timesteps_remaining: state.timesteps_remaining - 1,
                    });
                }
            }
        }

        PathfindingResult { path, score }
    }

    pub fn path_planning_lp(&self, start: (usize, usize), max_timesteps: u32, recovery_rate: f64) -> PathfindingResult {

        fn add_cube(vars: &mut ProblemVariables, var_def: VariableDefinition, x_len: usize, y_len: usize, z_len: usize) -> Vec<Vec<Vec<Variable>>> {
            let mut z =  vec![];

            for i in 0..x_len {
                let mut y = vec![];
                for j in 0..y_len {
                    let mut x = vec![];
                    for k in 0..z_len {
                        x.push(vars.add(var_def.clone().name(format!("x_{}_{}_{}", i, j, k))));
                    }
                    y.push(x);
                }
                z.push(y);
            }

            z
        }

        let mut vars = ProblemVariables::new();
        let nr_timesteps = max_timesteps as usize;

        // Binary variable indicating if you move from node at time t
        let x = add_cube(&mut vars, variable().binary(), self.size(), self.size(), nr_timesteps);
        // Binary variable indicating if you visit node at time t
        let v = add_cube(&mut vars, variable().binary(), self.size(), self.size(), nr_timesteps);
        // Integer variable indicating the score of node at time t
        let s = add_cube(&mut vars, variable(), self.size(), self.size(), nr_timesteps);
        // Integer variable indicating the score of node at time t
        let z = add_cube(&mut vars, variable(), self.size(), self.size(), nr_timesteps);

        let objective =
            z.iter().fold(Expression::default(), |acc, rows| {
                rows.iter().fold(acc, |acc_row, cols| {
                    acc_row + cols.iter().fold(Expression::default(), |acc_col, var| {
                        acc_col + var
                    })
                })
            });

        let mut solver = vars.maximise(objective).using(default_solver);

        solver.add_constraint(constraint!(x[start.0][start.1][0] == 1));

        for x in 0 .. self.size() {
            for y in 0 .. self.size() {
                // Set the initial scores
                solver.add_constraint(constraint!(s[x][y][0] == self.get_node_at((x, y)).score));
            }
        }

        for t in 1 .. nr_timesteps {

            for i in 0 .. self.size() {
                for j in 0 .. self.size() {
                    // If visited, reset; if not, recover
                    solver.add_constraint(constraint!(s[i][j][t - 1] + recovery_rate * (1 - v[i][j][t]) == s[i][j][t]));
                    // Cannot exceed max score
                    solver.add_constraint(constraint!(s[i][j][t - 1] <= self.get_node_at((i, j)).score));

                    match self.get_neighbors((i, j)) {
                        Some(neighbors) => {
                            let expr = neighbors.iter().fold(Expression::with_capacity(neighbors.len()), |acc, neighbor| {
                                acc + x[neighbor.0][neighbor.1][t - 1]
                            });

                            // Only visit nodes you arrive at
                            solver.add_constraint(constraint!(v[i][j][t] == expr));
                        },
                        None => ()
                    }

                    // Upper bound
                    solver.add_constraint(constraint!(z[i][j][t] <= s[i][j][t]));
                    // If not visiting, z = 0
                    solver.add_constraint(constraint!(z[i][j][t] <= v[i][j][t] * self.get_node_at((i, j)).score));
                    // If visiting, z = s
                    solver.add_constraint(constraint!(z[i][j][t] <= s[i][j][t] - (1 - v[i][j][t]) * self.get_node_at((i, j)).score));
                    // Non negative
                    solver.add_constraint(constraint!(z[i][j][t] >= 0));
                }
            }

        }

        let solution = solver.solve().unwrap();

        for i in 0 .. self.size() {
            for j in 0 .. self.size() {
                for t in 0 .. nr_timesteps {
                    if(solution.value(v[i][j][t]) == 1.0_f64) {
                        println!("x[{}][{}][{}]", i, j, t);
                    }
                }
            }
        }

        PathfindingResult::empty()
    }



}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_edges() {
        let mut graph = Graph::new(3);
        let edges = [
            // top left
            ((0, 0), vec![(0, 1), (1, 0), (1, 1)]),
            // top right
            ((0, 2), vec![(0, 1), (1, 2), (1, 1)]),
            // first row - in between nodes
            ((0, 1), vec![(0, 0), (0, 2), (1, 1), (1, 0), (1, 2)]),
            // second row - left
            ((1, 0), vec![(0, 0), (0, 1), (1, 1), (2, 1), (2, 0)]),
            // second row - right
            ((1, 2), vec![(0, 2), (0, 1), (1, 1), (2, 1), (2, 2)]),
            // second row - in between nodes
            ((1, 1), vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1), (2, 2)]),
            // bottom left
            ((2, 0), vec![(2, 1), (1, 1), (1, 0)]),
            // bottom right
            ((2, 2), vec![(2, 1), (1, 2), (1, 1)]),
            // bottom in between
            ((2, 1), vec![(2, 0), (2, 2), (1, 1), (1, 0), (1, 2)]),
        ];

        assert_eq!(graph.edges, edges.iter().cloned().collect());
    }
}