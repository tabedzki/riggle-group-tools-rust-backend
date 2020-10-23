extern crate cpython;

// use cpython::{PyResult, Python, py_module_initializer, py_fn};
// use cpython;
// use anyhow;
use csv::Reader; // 1.1.3
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde; // 1.0.117
use std::fs;
use std::fs::File;
use std::io::stdin;
use regex::Regex;
use std::io::{BufReader,BufRead};
// use std::slice;

#[repr(C)]
#[derive(serde::Deserialize)]
struct Particle {
    item: u32,
    mol: u32,
    x: f32,
    y: f32,
    z: f32,
    v_x: Option<f32>,
    v_y: Option<f32>,
    v_z: Option<f32>,
}

// fn read_file<T>(file_path: &str) -> PyResult<T> {
//     let mut count = 0;
//     let file = File::open(file_path)?;
//     let mut rdr = csv::Reader::from_reader(file);
//     let mut records = rdr.records();
//     for _ in 0..3 {
//         let record = records.next()?;
//         println!("{:?}", record);
//     }
//     Ok(());
// }

fn main() {
    let stdin = stdin();
    let mut reader = Reader::from_reader(stdin);
    let records = reader.deserialize::<Particle>().collect::<Vec<_>>();
}

#[repr(C)]
struct SimHolder {
    simulations: Vec<Simulation>,
}

#[repr(C)]
struct Simulation {
    frames: Vec<SimulationFrame>,
}

#[repr(C)]
struct SimulationFrame {
    time: u32,
    pos: Vec<Vec<u32>>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

cpython::py_module_initializer!(rust2py, |py, m| {
    m.add(py, "__doc__", "This module is inplemented in Rust.")?;
    m.add(py, "get_result", cpython::py_fn!(py, get_result(val: &str)))?;
    m.add(py, "read_sim", cpython::py_fn!(py, read_sim(val: &str)))?;
    m.add(py, "echo", cpython::py_fn!(py, echo(val: Vec<f32>)))?;
    m.add(py, "avg", cpython::py_fn!(py, avg(val: Vec<f32>)))?;
    m.add(py, "test2", cpython::py_fn!(py, test2(val: Vec<Vec<f32>>)))?;
    m.add(
        py,
        "row_avg",
        cpython::py_fn!(py, row_avg(val: Vec<Vec<f32>>)),
    )?;
    Ok(())
});

fn get_result(_py: cpython::Python, val: &str) -> cpython::PyResult<String> {
    let mut x: u32 = 1;
    return Ok("Rust says: ".to_owned() + val);
}

fn read_sim(_py: cpython::Python, filename: &str) -> cpython::PyResult<String> {
    let contents = fs::read_to_string(filename).expect("Something with wrong reading the file");
    let mut f = File::open(filename).expect("File not found.");
    return Ok("Read in ".to_owned() + filename);
}

fn echo(_py: cpython::Python, nums: Vec<f32>) -> cpython::PyResult<Vec<f32>> {
    // let contents = fs::read_to_string(filename).expect("Something with wrong reading the file");
    //  let mut f = File::open(filename).expect("File not found.");
    return Ok(nums);
}

fn avg(_py: cpython::Python, nums: Vec<f32>) -> cpython::PyResult<f32> {
    // let contents = fs::read_to_string(filename).expect("Something with wrong reading the file");
    //  let mut f = File::open(filename).expect("File not found.");
    let mut total: f32 = nums.iter().sum();
    total /= nums.len() as f32;
    return Ok(total);
}

// fn test2(_py: Python, nums:  Vec<Vec<f32>>) -> PyResult<Vec<Vec<f32>>>{
fn test2(_py: cpython::Python, nums: Vec<Vec<f32>>) -> cpython::PyResult<f32> {
    let mut total: f32 = nums.iter().flatten().sum();
    // total /= nums.len() as f64;
    return Ok(total);
}

fn row_avg(_py: cpython::Python, nums: Vec<Vec<f32>>) -> cpython::PyResult<Vec<f32>> {
    // let mut total: f32 = nums.iter().flatten().sum();
    let mut totals: Vec<f32> = Vec::with_capacity(nums.len());
    for row in nums {
        let mut total: f32 = row.iter().sum();
        total /= row.len() as f32;
        totals.push(total / row.len() as f32);
    }
    // total /= nums.len() as f64;
    return Ok(totals);
}

#[pymodule]
fn rust2py_alt(py: Python, m: &PyModule) -> PyResult<()> {
    // let mut total: f32 = nums.iter().flatten().sum();
    // let mut totals: Vec<f32> = Vec::with_capacity(nums.len());
    // for row in nums{
    //     let total: f32 = row.iter().sum();
    //     total /= row.len() as f32;
    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(_py: Python, a: i64, b: i64) -> PyResult<String> {
        let out = sum_as_string(a, b);
        Ok(out)
    }

    m.add_function(wrap_pyfunction!(sum_as_string, m)?).unwrap();
    m.add_function(wrap_pyfunction!(read_file2, m)?).unwrap();
    //     totals.push(total / row.len() as f32);
    // }
    // total /= nums.len() as f64;
    // totals
    Ok(())
}

// logic implemented as a normal Rust function
#[pyfunction]
fn sum_as_string(a: i64, b: i64) -> String {
    format!("{}", a + b)
}

// fn read_frames(_py, nums) -> PyResult

#[pyfunction]
fn read_file2(file_path: &str) -> PyResult<()> {
    let re = Regex::new("TIMESTEP").unwrap();
    let resultt = re.find_iter(file_path).count();
    println!("Found TIMESTEP {} times", resultt);
    let f = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).flexible(true).delimiter(b' ').from_reader(f);
    let mut records = rdr.records();
    for _ in 0..3 {
        if let Some(Ok(record)) = records.next() {
            println!("{:?}", record);
        }
    }
    Ok(())
}

// let f = match f {
//     Ok(file) => file,
//     Err(e) => return Err(e)
// };
