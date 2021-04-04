//////////////////////////////////////////////////////////////////////////////
//  File: neil/solver.rs
//////////////////////////////////////////////////////////////////////////////
//  Copyright 2016 Samuel Sleight
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//////////////////////////////////////////////////////////////////////////////

use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;

use super::problem::Problem;

/**
 * A solver will take a problem and use simulated annealing
 * to try and find an optimal state.
 */
#[derive(Debug, Clone)]
pub struct Solver {
    /**
     * The maximum number of iterations to run the algorithm
     * for.
     */
    pub iterations: u64,

    /**
     * The initial temperature of the process.
     */
    pub initial_temperature: f64,

    /**
     * The factor to multiply the temperature by each time it
     * is lowered - this should be a number between 0.0 and 1.0.
     */
    pub temperature_reduction: f64,

    /**
     * The maximimum number of attempts to find a new state
     * before lowering the temperature.
     */
    pub max_attempts: u64,

    /**
     * The maximum number of accepted new states before lowering
     * the temperature.
     */
    pub max_accepts: u64,

    /**
     * The maximum number of rejected states before terminating the
     * process.
     */
    pub max_rejects: u64,
}

impl Solver {
    /**
     * Construct the new default solver.
     */
    pub fn new() -> Solver {
        Default::default()
    }

    /**
     * Construct a new solver with a given builder function.
     */
    pub fn build_new<F>(builder: F) -> Solver
    where
        F: FnOnce(&mut Solver),
    {
        let mut solver = Solver::new();
        builder(&mut solver);
        solver
    }

    /**
     * Run the solver on the given problem.
     */
    pub fn solve<P>(&self, problem: &P) -> P::State
    where
        P: Problem,
    {
        let mut rng = thread_rng();
        let range = Uniform::new(0.0, 1.0);

        let mut state = problem.initial_state();
        let mut energy = problem.energy(&state);
        let mut temperature = self.initial_temperature;

        let mut attempted = 0;
        let mut accepted = 0;
        let mut rejected = 0;

        for i in 0..self.iterations {
            state = {
                let next_state = problem.new_state(&state);
                let new_energy = problem.energy(&next_state);

                attempted += 1;

                let de = new_energy - energy;
                println!("[{}] {}", i, temperature);

                // println!(
                //     "[{}] temp: {}, energy: {}, new_energy: {}",
                //     i, temperature, energy, new_energy
                // );

                if de < 0.0 || range.sample(&mut rng) <= -de / temperature {
                    accepted += 1;
                    energy = new_energy;

                    next_state
                } else {
                    state
                }
            };

            if attempted >= self.max_attempts || accepted >= self.max_accepts {
                if accepted == 0 {
                    rejected += 1;
                } else {
                    rejected = 0;
                }

                attempted = 0;
                accepted = 0;
                temperature *= self.temperature_reduction;

                if rejected >= self.max_rejects {
                    println!("stop max rejected");
                    break;
                }
            }
        }

        state
    }
}

impl Default for Solver {
    fn default() -> Solver {
        Solver {
            iterations: 10_000,
            initial_temperature: 100.0,
            temperature_reduction: 0.95,
            max_attempts: 1000,
            max_accepts: 100,
            max_rejects: 100,
        }
    }
}
