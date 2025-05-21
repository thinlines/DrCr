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

use super::{calculator::ReportingGraphDependencies, types::{ReportingContext, ReportingProducts, ReportingStep}};

#[derive(Debug)]
pub enum ReportingExecutionError {
	DependencyNotAvailable { message: String }
}

pub fn execute_steps(
	steps: Vec<Box<dyn ReportingStep>>,
	dependencies: ReportingGraphDependencies,
	context: &ReportingContext,
) -> Result<ReportingProducts, ReportingExecutionError> {
	let mut products = ReportingProducts::new();

	for step in steps.iter() {
		step.execute(context, &steps, &dependencies, &mut products)?;
	}

	Ok(products)
}
