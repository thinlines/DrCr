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

use calculator::{steps_for_targets, ReportingCalculationError};
use executor::{execute_steps, ReportingExecutionError};
use types::{ReportingContext, ReportingProducts, ReportingStep};

pub mod builders;
pub mod calculator;
pub mod executor;
pub mod steps;
pub mod types;

#[derive(Debug)]
pub enum ReportingError {
	ReportingCalculationError(ReportingCalculationError),
	ReportingExecutionError(ReportingExecutionError),
}

impl From<ReportingCalculationError> for ReportingError {
	fn from(err: ReportingCalculationError) -> Self {
		ReportingError::ReportingCalculationError(err)
	}
}

impl From<ReportingExecutionError> for ReportingError {
	fn from(err: ReportingExecutionError) -> Self {
		ReportingError::ReportingExecutionError(err)
	}
}

pub fn generate_report(
	targets: Vec<Box<dyn ReportingStep>>,
	context: &ReportingContext,
) -> Result<ReportingProducts, ReportingError> {
	// Solve dependencies
	let (sorted_steps, dependencies) = steps_for_targets(targets, context)?;

	// Execute steps
	let products = execute_steps(sorted_steps, dependencies, context)?;

	Ok(products)
}
