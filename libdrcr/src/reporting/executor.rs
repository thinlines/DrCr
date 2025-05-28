/*
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022-2025  Lee Yingtong Li (RunasSudo)

	This program is free software: you can redistribute it and/or modify
	it under the terms of the GNU Affero General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU Affero General Public License for more details.

	You should have received a copy of the GNU Affero General Public License
	along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::sync::Arc;

use tokio::{sync::RwLock, task::JoinSet};

use super::{
	calculator::{would_be_ready_to_execute, ReportingGraphDependencies},
	types::{ReportingContext, ReportingProducts, ReportingStep},
};

#[derive(Debug)]
pub enum ReportingExecutionError {
	DependencyNotAvailable { message: String },
}

async fn execute_step(
	step_idx: usize,
	steps: Arc<Vec<Box<dyn ReportingStep>>>,
	dependencies: Arc<ReportingGraphDependencies>,
	context: Arc<ReportingContext>,
	products: Arc<RwLock<ReportingProducts>>,
) -> (usize, Result<ReportingProducts, ReportingExecutionError>) {
	let step = &steps[step_idx];
	let result = step
		.execute(&*context, &*steps, &*dependencies, &*products)
		.await;

	(step_idx, result)
}

pub async fn execute_steps(
	steps: Vec<Box<dyn ReportingStep>>,
	dependencies: ReportingGraphDependencies,
	context: Arc<ReportingContext>,
) -> Result<ReportingProducts, ReportingExecutionError> {
	let products = Arc::new(RwLock::new(ReportingProducts::new()));

	// Prepare for async
	let steps = Arc::new(steps);
	let dependencies = Arc::new(dependencies);

	// Execute steps asynchronously
	let mut handles = JoinSet::new();
	let mut steps_done = Vec::new();
	let mut steps_remaining = (0..steps.len()).collect::<Vec<_>>();

	while steps_done.len() != steps.len() {
		// Execute each step which is ready to run
		for step_idx in steps_remaining.iter().copied().collect::<Vec<_>>() {
			// Check if ready to run
			if would_be_ready_to_execute(&steps[step_idx], &steps, &dependencies, &steps_done) {
				// Spawn new task
				// Unfortunately the compiler cannot guarantee lifetimes are correct, so we must pass Arc across thread boundaries
				handles.spawn(execute_step(
					step_idx,
					Arc::clone(&steps),
					Arc::clone(&dependencies),
					Arc::clone(&context),
					Arc::clone(&products),
				));
				steps_remaining
					.remove(steps_remaining.iter().position(|i| *i == step_idx).unwrap());
			}
		}

		// Join next result
		let (step_idx, result) = handles.join_next().await.unwrap().unwrap();
		let step = &steps[step_idx];
		steps_done.push(step_idx);

		let mut new_products = result?;

		// Sanity check the new products
		for (product_id, _product) in new_products.map().iter() {
			if product_id.name != step.id().name {
				panic!(
					"Unexpected product name {} from step {}",
					product_id,
					step.id()
				);
			}
			if !step.id().product_kinds.contains(&product_id.kind) {
				panic!(
					"Unexpected product kind {} from step {}",
					product_id,
					step.id()
				);
			}
			if product_id.args != step.id().args {
				panic!(
					"Unexpected product args {} from step {}",
					product_id,
					step.id()
				);
			}
		}

		// Insert the new products
		products.write().await.append(&mut new_products);
	}

	Ok(Arc::into_inner(products).unwrap().into_inner())
}
