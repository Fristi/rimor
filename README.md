# rimor
This is a simple undirected pathfinder implemented in Rust. 

### General requirements

The path finding algorithm assumes the following

- There are no obstacles in the grid
- The algorithm can move horizontally, vertically and diagonally
- The path has a maximum of T timesteps
- For each time step you can move to another adjacent cell
- When you visit a cell the score is reset to 0
- When you visit a cell the score for other cells is increased by the `recovery_rate` which is default 1
- The algorithm tries to find a path which yields a maximum score

Both the app/CLI allow you to load grid files which are formatted as a 2D array of integers, where each integer represents a score. Each line is a row and each line contains multiple integers separted by a space. The grid has a symmetrical height and width, so it's an NxN grid

### Visualization

You can visualize the working of the pathfinder by visiting the webpage which is hosted on [GitHub Pages](https://fristi.github.io/rimor). This is WebAssembly version which works well for 20x20 grids to demonstrate it visually. 

![visualization](visualize.png)

#### Features
- You can load a grid from a file
- You can click on a tile which will have a red border, this does indicate the **start** tile
- You can tweak parameters like recovery rate, timesteps and the algorithm on the fly
- When you press **find path** it will show the score on the right and a path which changes color to easily track where it is going.
- Once a path is set, you can track it's exact path by looking at
  - The **top left** of a tile which shows the step
  - The **bottom right** of a tile which shows the score

### CLI

Another option for larger grids is to use the CLI version. This is a Rust CLI application which can be used to find the most feasible path in a grid. It is designed to work with larger grids, up to 1000x1000 and more

```bash
cli -I /Users/{user}/Downloads/grids/1000.txt -T 100 -x 1 -y 2 --timeout 100
```

### Available algorithms

#### Best First Search

This is a greedy algorithm which tries to find the best path by exploring the most promising nodes first. For each node it visits it get the adjacent nodes and put these values in priority queue and in the next step it will explore the item which has the highest score in the queue. The time complexity is O(T) which is fast, but it will miss the honey pots which are not considered with this approach



