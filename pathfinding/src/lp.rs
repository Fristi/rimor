use std::collections::{HashMap, HashSet};
use std::env::var;
use std::hash::{Hash, Hasher, RandomState};
use crate::{Graph, PathfindingResult, PathfindingStep, Position};
use good_lp::*;

#[derive(Eq, PartialEq, Clone, Hash)]
struct Edge {
    from: Position,
    to: Position
}

impl Edge {
    fn new(from: Position, to: Position) -> Self {
        Edge { from, to }
    }

    fn get_edges_out(graph: &Graph, pos: Position) -> Vec<Edge> {
        graph.get_nodes_out(pos).iter().map(|&n| Edge::new(pos, n)).collect()
    }

    fn get_edges_in(graph: &Graph, pos: Position) -> Vec<Edge> {
        graph.get_edges_in(pos).iter().map(|&n| Edge::new(n, pos)).collect()
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct Node {
    pos: Position
}

impl Node {

    fn neighbour(edge: Edge, graph: &Graph) -> (Node, Vec<Edge>) {
        let mut edges_out = Edge::get_edges_out(graph, edge.to);
        let mut edges_in = Edge::get_edges_in(graph, edge.from);

        let mut edges = vec![];
        edges.append(&mut edges_in);
        edges.append(&mut edges_out);

        (Node { pos: edge.to }, edges)
    }

    fn start(graph: &Graph, pos: Position) -> (Node, Vec<Edge>) {
        let edges = Edge::get_edges_out(&graph, pos);

        (Node { pos }, edges)
    }
}



impl Graph {




    pub fn lp(&self, start: Position, max_timesteps: u32) -> PathfindingResult {
        let mut vars = ProblemVariables::new();
        let graph = self.clone();
        let (start_node, initial_edges) = Node::start(&graph, start);
        let mut edges = HashSet::new();
        let mut stack = vec![];
        let mut edge_vars = HashMap::new();
        let mut objective_exprs = vec![];
        let mut seen_edges = HashSet::new();
        let mut nodes = HashSet::new();

        fn sum_var(vars: Vec<Variable>) -> Expression {
            vars.iter().fold(Expression::default(), |acc, n| acc + n)
        }

        fn sum_expr(expr: Vec<Expression>) -> Expression {
            expr.iter().fold(Expression::default(), |acc, n| acc + n)
        }

        stack.append(&mut initial_edges.clone());

        for edge in initial_edges {
            edges.insert(edge);
        }


        while let Some(edge) = stack.pop() {

            let (node, neighbor_edges) = Node::neighbour(edge, &graph);

            nodes.insert(node);

            for x in neighbor_edges.iter() {
                if seen_edges.contains(x) {
                    continue;
                }
                seen_edges.insert(x.clone());
                edges.insert(x.clone());
                stack.push(x.clone());
            }
        }

        println!("Edges length: {}", edges.len());

        for edge in edges.iter() {
            let var = vars.add(variable().binary());
            edge_vars.insert(edge, var);
            let score = *graph.get_score_at(edge.to);

            if edge.from == (1,1) || edge.from == (0,0)  {
                println!("Edge {:?} -> {:?} ({:?}) has value {}", edge.from, edge.to, var, score);
            }

            objective_exprs.push(var * score);
        }

        println!("Objective length: {}", objective_exprs.len());

        println!("---");

        let objective = sum_expr(objective_exprs);

        println!("Objective: {:?}", objective);

        println!("---");

        let mut solver = vars.maximise(objective).using(default_solver);

        for node in nodes.iter() {
            let mut edges_out: Vec<Variable> = Edge::get_edges_out(&graph, node.pos).iter().filter_map(|x| edge_vars.get(x)).copied().collect();
            let mut edges_in: Vec<Variable>  = Edge::get_edges_in(&graph, node.pos).iter().filter_map(|x| edge_vars.get(x)).copied().collect();
            let n = Node { pos: (1,1) };
            if *node == n  {
                println!("Edges in: {:?}", &edges_in);
                println!("Edges out: {:?}", &edges_out);
            }

            let lhs = sum_var(edges_in);
            let rhs = sum_var(edges_out);

            if *node == n  {
                println!("lhs: {:?}", &lhs);
                println!("rhs: {:?}", &rhs);
            }

            let constraint = if *node == start_node {
                constraint!(rhs == 1)
            } else {
                constraint!(lhs - rhs == 0)
            };



            solver.add_constraint(constraint);
        }

        println!("---");

        let variables = edge_vars.values().cloned().collect();
        let sum_of_variables = sum_var(variables);
        let total_value = max_timesteps as f64;

        println!("Sum of variables: {:?} == {:?}", sum_of_variables, total_value);

        solver.add_constraint(constraint!(sum_of_variables == total_value));

        let solution = solver.solve().expect("Could not find a solution");

        println!("---");

        for edge in edges.iter() {
            let var = edge_vars.get(edge).unwrap();
            let val = solution.value(*var);
            let te= Edge { from: (1,1), to: (2,1) };
            if val == 1.0 || *edge == te  {
                println!("Edge {:?} -> {:?} ({:?}) has value {}", edge.from, edge.to, var, val);
            }
        }

        let mut step = 0;
        let mut path = vec![];
        let mut current_pos = Some(start);

        step += 1;
        path.push(PathfindingStep { node: start, score: *graph.get_score_at(start), step });


        while let Some(from) = current_pos {
            if step == max_timesteps {
                break
            }
            for to in graph.get_nodes_out(from) {
                let edge = Edge::new(from, to);
                match edge_vars.get(&edge).map(|&x| solution.value(x)) {
                    Some(val) if val == 1.0 => {
                        step += 1;
                        path.push(PathfindingStep { node: to, score: *graph.get_score_at(to), step });
                        current_pos = Some(to);
                        break;
                    },
                    _ => {
                        current_pos = None;
                    }
                }
            }

        }

        PathfindingResult { path }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lp() {
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

        let result = graph.lp(start, max_timesteps);

        println!("Result: {:?}", result);

        assert_eq!(result, PathfindingResult {
            path: vec![
                PathfindingStep { node: (0, 0), score: 0, step: 1 },
                PathfindingStep { node: (1, 1), score: 7, step: 2 },
                PathfindingStep { node: (2, 1), score: 10, step: 3 }
            ]
        });
    }

    fn lp_mid_path_problem() {
        let mut vars = ProblemVariables::new();

        let x12 = vars.add(variable().binary());
        let x13 = vars.add(variable().binary());
        let x14 = vars.add(variable().binary());
        let x23 = vars.add(variable().binary());
        let x25 = vars.add(variable().binary());
    }

    fn lp_short_path_problem() {
        let mut vars = ProblemVariables::new();
        let x12 = vars.add(variable().binary());
        let x13 = vars.add(variable().binary());
        let x14 = vars.add(variable().binary());
        let x23 = vars.add(variable().binary());
        let x41 = vars.add(variable().binary());



        let objective = x12 * 10 + x13 * 60 + x14 * 70 + x23 * 20;
        let mut solver = vars.minimise(objective).using(default_solver);

        //node 1 - move out of the start
        solver.add_constraint(constraint!(x12 + x13 + x14 == 1));
        // node 2 - should be able to move from 12 to 23
        solver.add_constraint(constraint!(x23 == x12));
        // node 3 - end node, should be able to move from 13 or 23
        solver.add_constraint(constraint!(x23 + x13 == 1));
        // node 4 - should be able to move from 14 to 41
        solver.add_constraint(constraint!(x14 == x41));

        let solution = solver.solve().expect("Unable to solve the problem");

        println!("x12: {}", solution.value(x12));
        println!("x13: {}", solution.value(x13));
        println!("x14: {}", solution.value(x14));
        println!("x23: {}", solution.value(x23));
        println!("x41: {}", solution.value(x41));
    }

    #[test]
    fn test_small_lp_problem() {
        lp_short_path_problem()
    }
}