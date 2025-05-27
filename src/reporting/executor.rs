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

use tokio::sync::RwLock;

use super::{
	calculator::ReportingGraphDependencies,
	types::{ReportingContext, ReportingProducts, ReportingStep},
};

#[derive(Debug)]
pub enum ReportingExecutionError {
	DependencyNotAvailable { message: String },
}

pub async fn execute_steps(
	steps: Vec<Box<dyn ReportingStep>>,
	dependencies: ReportingGraphDependencies,
	context: &ReportingContext,
) -> Result<ReportingProducts, ReportingExecutionError> {
	let products = RwLock::new(ReportingProducts::new());

	for step in steps.iter() {
		// Execute the step
		// TODO: Do this in parallel
		let mut new_products = step
			.execute(context, &steps, &dependencies, &products)
			.await?;

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

	Ok(products.into_inner())
}
