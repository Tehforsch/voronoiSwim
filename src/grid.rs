use crate::{
    cell::Cell, direction::Direction, face::Face, graph::Graph, task::Task, vector_3d::Vector3D,
};
use ordered_float::OrderedFloat;

pub struct Grid {
    data: Graph<Cell, Face>,
}

impl Grid {
    fn get_dependency_graph<'b>(&self, direction: &'b Direction) -> Graph<Task<'_, 'b>, ()> {
        let tasks: Vec<Task> = self.data.iter().map(|cell| {
            Task {
                cell,
                direction
            }
        }).collect();
        let mut dependency_data = vec![];
        for (upwind_cell, downwind_cell, face) in self.data.iter_edges() {
            if Grid::is_downwind(face, &direction) {
                dependency_data.push((upwind_cell.label, downwind_cell.label, ()));
            }
        }
        Graph::from_nodes_and_edge_list(tasks, dependency_data)
    }

    fn is_downwind(face: &Face, direction: &Direction) -> bool {
        let scalar_product = face.normal.dot(&direction.vector);
        scalar_product < OrderedFloat(0.0)
    }

    pub fn from_cell_pairs(cells: Vec<Cell>, pairs: &[(usize, usize)]) -> Grid {
        let edge_list: Vec<(usize, usize, Face)> = pairs
            .into_iter()
            .map(|(i0, i1)| {
                (
                    *i0,
                    *i1,
                    Grid::face_between(&cells[*i0], &cells[*i1]),
                )
            })
            .collect();
        Grid {
            data: Graph::from_nodes_and_edge_list(cells, edge_list),
        }
    }

    fn face_between(cell_0: &Cell, cell_1: &Cell) -> Face {
        Face {
            normal: (cell_0.center.sub(&cell_1.center)),
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use super::*;
    #[test]
    fn dependency_graph() {
        let direction = Direction {
            index: 0,
            vector: Vector3D::new(1.0, 0.0, 0.0),
        };
        let cells = vec![
            Cell {
                label: 0,
                center: Vector3D::new(0., 0., 0.),
            },
            Cell {
                label: 1,
                center: Vector3D::new(1., 0., 0.),
            },
        ];
        let first_cell = cells[0].clone();
        let grid = Grid::from_cell_pairs(cells, &[(0, 1)]);
        let graph = grid.get_dependency_graph(&direction);
        let nodes = graph.traverse_depth_first(&Task {
            cell: &first_cell,
            direction: &direction,
        });
        let labels: Vec<Task> = nodes.iter().map(|node| node.label.clone()).collect();
        assert_tasks_equal(&labels, &[(0, 0), (1, 0)]);
    }

    fn assert_tasks_equal(tasks: &[Task], indices: &[(usize, usize)]) {
        for task_info in tasks.iter().zip_longest(indices.iter()) {
            match task_info {
                itertools::EitherOrBoth::Both(task, (cell_index, dir_index)) => {
                    assert_eq!(task.cell.label, *cell_index);
                    assert_eq!(task.direction.index, *dir_index);
                }
                _ => {
                    assert!(false);
                }
            };
        }
    }

}
