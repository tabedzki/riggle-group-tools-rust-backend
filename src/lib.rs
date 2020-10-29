use itertools;
use pyo3::prelude::*;
use rayon::prelude::*;
use serde; // 1.0.117
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, ErrorKind};

#[repr(C)]
#[pyclass]
#[derive(serde::Deserialize, Debug, Clone)]
pub(crate) struct Particle {
    #[pyo3(get)]
    item: u32,
    #[pyo3(get)]
    part_type: u32,
    #[pyo3(get)]
    mol: u32,
    #[pyo3(get)]
    x: f32,
    #[pyo3(get)]
    y: f32,
    #[pyo3(get)]
    z: f32,
    #[pyo3(get)]
    mass: Option<f32>,
    #[pyo3(get)]
    vx: Option<f32>,
    #[pyo3(get)]
    vy: Option<f32>,
    #[pyo3(get)]
    vz: Option<f32>,
    #[pyo3(get)]
    xs: Option<f32>,
    #[pyo3(get)]
    ys: Option<f32>,
    #[pyo3(get)]
    zs: Option<f32>,
    #[pyo3(get)]
    xsu: Option<f32>,
    #[pyo3(get)]
    ysu: Option<f32>,
    #[pyo3(get)]
    zsu: Option<f32>,
    #[pyo3(get)]
    fx: Option<f32>,
    #[pyo3(get)]
    fy: Option<f32>,
    #[pyo3(get)]
    fz: Option<f32>,
    #[pyo3(get)]
    mux: Option<f32>,
    #[pyo3(get)]
    muy: Option<f32>,
    #[pyo3(get)]
    muz: Option<f32>,
    #[pyo3(get)]
    omegax: Option<f32>,
    #[pyo3(get)]
    omegay: Option<f32>,
    #[pyo3(get)]
    omegaz: Option<f32>,
    #[pyo3(get)]
    angmomx: Option<f32>,
    #[pyo3(get)]
    angmomy: Option<f32>,
    #[pyo3(get)]
    angmomz: Option<f32>,
}

#[repr(C)]
#[pyclass]
#[derive(Debug, Clone)]
pub(crate) struct SimHolder {
    #[pyo3(get, set)]
    pub simulations: Vec<Simulation>,
}

#[repr(C)]
#[pyclass]
#[derive(Debug, Clone)]
pub(crate) struct Simulation {
    #[pyo3(get)]
    pub frames: Vec<SimulationFrame>,
}

#[repr(C)]
#[pyclass]
#[derive(Debug, Clone)]
pub(crate) struct SimulationFrame {
    #[pyo3(get)]
    time: u64,
    #[pyo3(get)]
    atoms: Vec<Particle>,
    #[pyo3(get)]
    box_size: Vec<(f32, f32)>,
}

#[pymethods]
impl SimHolder {
    #[new]
    fn new() -> Self {
        Self {
            simulations: Vec::with_capacity(0),
        }
    }

    pub fn add_simulations(&mut self, sims: Vec<Simulation>) -> PyResult<()> {
        self.simulations.extend(sims);
        Ok(())
    }
}

#[pymethods]
impl Simulation {
    #[new]
    fn new() -> Self {
        Self {
            frames: Vec::with_capacity(0),
        }
    }

    pub fn add_frames(&mut self, frames: Vec<SimulationFrame>) -> PyResult<()> {
        self.frames.extend(frames);
        Ok(())
    }

    pub fn calc_msd(&mut self, idx_begin: usize, idx_end: usize) -> PyResult<Vec<Vec<f32>>> {
        Ok(calculate_msd(self, idx_begin, idx_end)?)
    }
}

#[pymethods]
impl SimulationFrame {
    #[new]
    fn new() -> Self {
        Self {
            time: 0,
            atoms: Vec::with_capacity(0),
            box_size: Vec::with_capacity(3),
        }
    }

    pub fn add_atoms(&mut self, atoms: Vec<Particle>) -> PyResult<()> {
        self.atoms.extend(atoms);
        Ok(())
    }
}

#[pymodule]
fn rust2py_alt(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "read_file")]
    fn read_file_py(_py: Python, file_path: &str) -> PyResult<SimHolder> {
        let out = read_file(file_path)?;
        Ok(out)
    }

    m.add_class::<Particle>()?;
    m.add_class::<Simulation>()?;
    m.add_class::<SimHolder>()?;
    m.add_class::<SimulationFrame>()?;

    Ok(())
}

fn calculate_msd(
    sim: &mut Simulation,
    start_idx: usize,
    idx_end: usize,
) -> Result<Vec<Vec<f32>>, io::Error> {

    Ok(sim.frames[start_idx..idx_end]
        .par_iter()
        .map(|frame| {
            frame
                .atoms
                .iter()
                .map(|p| {
                    (p.x - frame.atoms[start_idx].x).powi(2)
                        + (p.y - frame.atoms[start_idx].y).powi(2)
                        + (p.z - frame.atoms[start_idx].z).powi(2)
                })
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>())
}

fn read_file(file_path: &str) -> Result<SimHolder, io::Error> {
    let file = BufReader::new(File::open(&file_path)?);

    let matched_lines = itertools::process_results(file.lines(), |i| {
        i.filter(|l| l.contains("TIMESTEP")).count()
    })?;

    let mut sim_holder = SimHolder {
        simulations: Vec::with_capacity(1),
    };
    sim_holder.simulations.push(Simulation {
        frames: Vec::with_capacity(matched_lines),
    });
    let f = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(b' ')
        .from_reader(f);

    let records = rdr.records();
    let mut status: Option<String> = None;
    let mut cur_frame: Option<&mut SimulationFrame> = None;
    let mut lines2read = 0;

    for record in records {
        let line = record.unwrap();
        let rr: Option<Particle> = line.deserialize(None).unwrap_or_else(|_| None);

        if let Some(particle) = rr {
            cur_frame
                .as_mut()
                .ok_or_else(|| {
                    return io::Error::new(
                        ErrorKind::Other, "Invalid Line Found: No frame initialized. Most likely improper file format.");
                })?
                .atoms
                .push(particle);
            continue;
        }

        if lines2read > 0 {
            match status.as_deref() {
                Some(x) if x.contains("TIMESTEP") => {
                    sim_holder
                        .simulations
                        .last_mut()
                        .unwrap()
                        .frames
                        .push(SimulationFrame {
                            time: line.as_slice().parse::<u64>().unwrap(),
                            atoms: Vec::with_capacity(0),
                            box_size: Vec::with_capacity(3),
                        });
                    cur_frame = Some(
                        sim_holder
                            .simulations
                            .last_mut()
                            .expect("No simulation found.")
                            .frames
                            .last_mut()
                            .expect("No simulation frame found."),
                    );
                    status = None;
                    lines2read = 0;
                }
                Some(x) if x.contains("BOX") => {
                    lines2read -= 1;
                    cur_frame
                        .as_mut()
                        .unwrap()
                        .box_size
                        .push(line.deserialize(None).expect("Expected Box Dimensions"));
                    if lines2read == 0 {
                        status = None;
                    }
                }
                Some(x) if x.contains("NUMBER") => {
                    lines2read = 0;
                    let capacity = line
                        .as_slice()
                        .parse::<usize>()
                        .expect("Expected number of ATOMS");
                    cur_frame.as_mut().unwrap().atoms.reserve_exact(capacity);
                    status = None;
                }
                Some(_) => {
                    return Err(io::Error::new(
                        ErrorKind::Other,
                        // PyValueError::new_err(
                        "Not one of the possible options.",
                    ));
                }
                None => (),
            }
        } else {
            match line {
                x if x.as_slice().contains("TIMESTEP") => {
                    status = Some(String::from("TIMESTEP"));
                    lines2read = 1;
                }
                x if x.as_slice().contains("BOX") => {
                    if cur_frame.is_none() {
                        return Err(io::Error::new(
                            ErrorKind::Other,
                            "Simulation frame is not initialized.",
                        ));
                    }
                    status = Some(String::from("BOX"));
                    lines2read = x.len() - 3;
                }
                x if x.as_slice().contains("NUMBER") => {
                    if cur_frame.is_none() {
                        return Err(io::Error::new(
                            ErrorKind::Other,
                            "Simulation frame is not initialized.",
                        ));
                    }
                    status = Some(String::from("NUMBER"));
                    lines2read = 1;
                }
                x if x.as_slice().contains("ITEM:ATOMS") => {
                    if cur_frame.is_none() {
                        return Err(io::Error::new(
                            ErrorKind::Other,
                            "Simulation frame is not initialized.",
                        ));
                    }
                    status = Some(String::from("ATOMS"));
                }
                x => {
                    return Err(io::Error::new(
                        ErrorKind::Other, // PyValueError::new_err(
                        format!("Invalid Line Found:\n{}", x.as_slice()).as_str(),
                    ));
                }
            };
        }
    }

    Ok(sim_holder)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
