use anyhow::{anyhow, Context, Result};
use std::{error::Error, fs, io, path::Path};

use crate::command_line_args::CommandLineArgs;
use crate::config::NUM_DIRECTIONS;
use crate::direction::{get_equally_distributed_directions_on_sphere, Direction};
use crate::grid::Grid;
use crate::run_data::RunData;
use crate::{
    cell::Cell, domain_decomposition::do_domain_decomposition, sweep::Sweep,
    util::get_shell_command_output, vector_3d::Vector3D,
};

pub fn run(args: &CommandLineArgs) -> Result<(), Box<dyn Error>> {
    let directions = get_equally_distributed_directions_on_sphere(NUM_DIRECTIONS);
    let grids: Result<Vec<_>> = args
        .grid_files
        .iter()
        .map(|file| convert_to_grid(&file))
        .collect();
    let run_data_list: Vec<_> = match args.domain_decomposition {
        None => grids?
            .into_iter()
            .map(|mut grid| run_sweep_on_processors(&mut grid, &directions))
            .collect(),
        Some(num) => grids?
            .into_iter()
            .map(|mut grid| {
                run_sweep_and_domain_decomposition_on_processors(&mut grid, &directions, num)
            })
            .collect(),
    };
    let reference = &run_data_list[0];
    for run_data in run_data_list.iter() {
        if !args.quiet {
            println!(
                "{:>4} {:.3} (speedup: {:>6.2}, efficiency {:>6.2}), comm: {:.3}, idle: {:.3}",
                run_data.num_processors,
                run_data.time,
                run_data.get_speedup(&reference),
                run_data.get_efficiency(&reference),
                run_data.time_spent_communicating / run_data.time,
                run_data.time_spent_waiting / run_data.time,
            );
        }
    }
    Ok(())
}

fn convert_to_grid(file: &Path) -> Result<Grid> {
    if let Some(extension) = file.extension() {
        let ext_str = extension.to_str().unwrap();
        if ext_str == "hdf5" {
            return read_hdf5_file(file);
        } else if ext_str == "dat" {
            return read_grid_file(file).context("While reading file as grid file");
        }
    }
    Err(anyhow!("Unknown file ending"))
}

fn run_sweep_and_domain_decomposition_on_processors(
    mut grid: &mut Grid,
    directions: &[Direction],
    num_processors: usize,
) -> RunData {
    do_domain_decomposition(&mut grid, num_processors);
    let mut sweep = Sweep::new(&grid, &directions, num_processors);
    sweep.run()
}

fn run_sweep_on_processors(grid: &mut Grid, directions: &[Direction]) -> RunData {
    let num_processors = grid.iter().map(|cell| cell.processor_num).max().unwrap() + 1;
    let mut sweep = Sweep::new(&grid, &directions, num_processors);
    sweep.run()
}

fn read_grid_file(grid_file: &Path) -> io::Result<Grid> {
    let contents = fs::read_to_string(grid_file)?;
    let mut cells = vec![];
    let mut edges = vec![];
    for line in contents.lines() {
        let mut split = line.split_ascii_whitespace();
        let label = split.next().unwrap().parse::<usize>().unwrap();
        let processor_num = split.next().unwrap().parse::<usize>().unwrap();
        let x = split.next().unwrap().parse::<f64>().unwrap();
        let y = split.next().unwrap().parse::<f64>().unwrap();
        let z = split.next().unwrap().parse::<f64>().unwrap();
        let neighbours = split.map(|num| num.parse::<usize>().unwrap());
        let center = Vector3D::new(x, y, z);
        cells.push(Cell {
            index: label,
            center,
            processor_num,
        });
        for neighbour in neighbours {
            edges.push((label, neighbour));
        }
    }
    Ok(Grid::from_cell_pairs(cells, &edges))
}

fn read_hdf5_file(hdf5_file: &Path) -> Result<Grid> {
    let filename = hdf5_file.to_str().unwrap();
    let out = get_shell_command_output(
        &"python3",
        &[
            "/home/toni/projects/swedule/getVoronoiNeighbours/getNeighbours.py",
            filename,
        ],
        None,
        false,
    );
    let grid_file = match out.success {
        true => Ok(hdf5_file.with_extension("dat")),
        false => Err(anyhow!(
            "Failed to convert snapshot to grid file: {}",
            &out.stderr
        )),
    };
    grid_file.and_then(|file| read_grid_file(&file).context("While reading grid file"))
}