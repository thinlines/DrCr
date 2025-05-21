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

use super::{
	calculator::{has_step_or_can_build, HasStepOrCanBuild, ReportingGraphDependencies},
	DateArgs, DateStartDateEndArgs, ReportingContext, ReportingProductId, ReportingProductKind,
	ReportingStep, ReportingStepArgs, ReportingStepDynamicBuilder, ReportingStepId,
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
	args: DateStartDateEndArgs,
}

impl BalancesAtToBalancesBetween {
	// Implements BalancesAt, BalancesAt -> BalancesBetween

	fn can_build(
		name: &'static str,
		kind: ReportingProductKind,
		args: &Box<dyn ReportingStepArgs>,
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
					args: args.clone(),
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
		args: Box<dyn ReportingStepArgs>,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		_context: &ReportingContext,
	) -> Box<dyn ReportingStep> {
		Box::new(BalancesAtToBalancesBetween {
			step_name: name,
			args: *args.downcast().unwrap(),
		})
	}
}

impl ReportingStep for BalancesAtToBalancesBetween {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.step_name,
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: Box::new(self.args.clone()),
		}
	}

	fn init_graph(
		&self,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
	) {
		// BalancesAtToBalancesBetween depends on BalancesAt at both time points
		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: self.args.date_start.clone(),
				}),
			},
		);
		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: self.args.date_end.clone(),
				}),
			},
		);
	}
}

#[derive(Debug)]
pub struct UpdateBalancesBetween {
	step_name: &'static str,
	args: DateStartDateEndArgs,
}

impl UpdateBalancesBetween {
	// Implements (BalancesBetween -> Transactions) -> BalancesBetween

	fn can_build(
		name: &'static str,
		kind: ReportingProductKind,
		_args: &Box<dyn ReportingStepArgs>,
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
		args: Box<dyn ReportingStepArgs>,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		_context: &ReportingContext,
	) -> Box<dyn ReportingStep> {
		Box::new(UpdateBalancesBetween {
			step_name: name,
			args: *args.downcast().unwrap(),
		})
	}
}

impl ReportingStep for UpdateBalancesBetween {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.step_name,
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: Box::new(self.args.clone()),
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
