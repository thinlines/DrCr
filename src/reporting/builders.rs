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

use chrono::NaiveDate;

use super::{
	calculator::{has_step_or_can_build, HasStepOrCanBuild, ReportingGraphDependencies},
	ReportingContext, ReportingProductId, ReportingProductKind, ReportingStep,
	ReportingStepDynamicBuilder, ReportingStepId,
};

pub fn register_dynamic_builders(context: &mut ReportingContext) {
	context.register_dynamic_builder(ReportingStepDynamicBuilder {
		name: "BalancesAtToBalancesBetween",
		can_build: BalancesAtToBalancesBetween::can_build,
		build: BalancesAtToBalancesBetween::build,
	});

	context.register_dynamic_builder(ReportingStepDynamicBuilder {
		name: "UpdateBalancesBetween",
		can_build: UpdateBalancesBetween::can_build,
		build: UpdateBalancesBetween::build,
	});
}

#[derive(Debug)]
pub struct BalancesAtToBalancesBetween {
	step_name: &'static str,
	date_start: NaiveDate,
	date_end: NaiveDate,
}

impl BalancesAtToBalancesBetween {
	// Implements BalancesAt, BalancesAt -> BalancesBetween

	fn can_build(
		name: &'static str,
		kind: ReportingProductKind,
		args: Vec<String>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		context: &ReportingContext,
	) -> bool {
		// Check for BalancesAt, BalancesAt -> BalancesBetween
		if kind == ReportingProductKind::BalancesBetween {
			match has_step_or_can_build(
				&ReportingProductId {
					name,
					kind: ReportingProductKind::BalancesAt,
					args: vec![args[1].clone()],
				},
				steps,
				dependencies,
				context,
			) {
				HasStepOrCanBuild::HasStep(_)
				| HasStepOrCanBuild::CanLookup(_)
				| HasStepOrCanBuild::CanBuild(_) => {
					return true;
				}
				HasStepOrCanBuild::None => {}
			}
		}
		return false;
	}

	fn build(
		name: &'static str,
		_kind: ReportingProductKind,
		args: Vec<String>,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		_context: &ReportingContext,
	) -> Box<dyn ReportingStep> {
		Box::new(BalancesAtToBalancesBetween {
			step_name: name,
			date_start: NaiveDate::parse_from_str(&args[0], "%Y-%m-%d").unwrap(),
			date_end: NaiveDate::parse_from_str(&args[1], "%Y-%m-%d").unwrap(),
		})
	}
}

impl ReportingStep for BalancesAtToBalancesBetween {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.step_name,
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: vec![
				self.date_start.format("%Y-%m-%d").to_string(),
				self.date_end.format("%Y-%m-%d").to_string(),
			],
		}
	}

	fn init_graph(
		&self,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
	) {
		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: vec![self.date_start.format("%Y-%m-%d").to_string()],
			},
		);
		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: vec![self.date_end.format("%Y-%m-%d").to_string()],
			},
		);
	}
}

#[derive(Debug)]
pub struct UpdateBalancesBetween {
	step_name: &'static str,
	date_start: NaiveDate,
	date_end: NaiveDate,
}

impl UpdateBalancesBetween {
	// Implements (BalancesBetween -> Transactions) -> BalancesBetween

	fn can_build(
		name: &'static str,
		kind: ReportingProductKind,
		_args: Vec<String>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		_context: &ReportingContext,
	) -> bool {
		// Check for Transactions -> BalancesBetween
		if kind == ReportingProductKind::BalancesBetween {
			// Initially no need to check args
			if let Some(step) = steps.iter().find(|s| {
				s.id().name == name
					&& s.id()
						.product_kinds
						.contains(&ReportingProductKind::Transactions)
			}) {
				// Check for BalancesBetween -> Transactions
				let dependencies_for_step = dependencies.dependencies_for_step(&step.id());
				if dependencies_for_step.len() == 1
					&& dependencies_for_step[0].dependency.kind
						== ReportingProductKind::BalancesBetween
				{
					return true;
				}
			}

			// Check lookup or builder - with args
			/*match has_step_or_can_build(
				&ReportingProductId {
					name,
					kind: ReportingProductKind::Transactions,
					args: args.clone(),
				},
				steps,
				dependencies,
				context,
			) {
				HasStepOrCanBuild::HasStep(step) => unreachable!(),
				HasStepOrCanBuild::CanLookup(_)
				| HasStepOrCanBuild::CanBuild(_)
				| HasStepOrCanBuild::None => {}
			}*/
		}
		return false;
	}

	fn build(
		name: &'static str,
		_kind: ReportingProductKind,
		args: Vec<String>,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		_context: &ReportingContext,
	) -> Box<dyn ReportingStep> {
		Box::new(UpdateBalancesBetween {
			step_name: name,
			date_start: NaiveDate::parse_from_str(&args[0], "%Y-%m-%d").unwrap(),
			date_end: NaiveDate::parse_from_str(&args[1], "%Y-%m-%d").unwrap(),
		})
	}
}

impl ReportingStep for UpdateBalancesBetween {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.step_name,
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: vec![
				self.date_start.format("%Y-%m-%d").to_string(),
				self.date_end.format("%Y-%m-%d").to_string(),
			],
		}
	}

	fn init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
	) {
		// Add a dependency on the Transactions result
		// Look up that step, so we can extract the appropriate args
		let parent_step = steps
			.iter()
			.find(|s| {
				s.id().name == self.step_name
					&& s.id()
						.product_kinds
						.contains(&ReportingProductKind::Transactions)
			})
			.unwrap(); // Existence is checked in can_build

		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::Transactions,
				args: parent_step.id().args.clone(),
			},
		);
	}
}
