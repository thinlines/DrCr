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
	ReportingContext, ReportingProductId, ReportingProductKind, ReportingStep,
	ReportingStepDynamicBuilder, ReportingStepId, ReportingStepLookupFn,
};

#[derive(Debug)]
pub struct ReportingGraphDependencies {
	vec: Vec<Dependency>,
}

impl ReportingGraphDependencies {
	pub fn vec(&self) -> &Vec<Dependency> {
		&self.vec
	}

	pub fn add_dependency(&mut self, step: ReportingStepId, dependency: ReportingProductId) {
		if !self
			.vec
			.iter()
			.any(|d| d.step == step && d.dependency == dependency)
		{
			self.vec.push(Dependency { step, dependency });
		}
	}

	pub fn add_target_dependency(&mut self, target: ReportingStepId, dependency: ReportingStepId) {
		for kind in target.product_kinds {
			match kind {
				ReportingProductKind::Transactions | ReportingProductKind::BalancesBetween => {
					self.add_dependency(
						target.clone(),
						ReportingProductId {
							name: dependency.name,
							kind: *kind,
							args: target.args.clone(),
						},
					);
				}
				ReportingProductKind::BalancesAt => todo!(),
				ReportingProductKind::Generic => todo!(),
			}
		}
	}

	pub fn dependencies_for_step(&self, step: &ReportingStepId) -> Vec<&Dependency> {
		return self.vec.iter().filter(|d| d.step == *step).collect();
	}
}

#[derive(Debug)]
pub struct Dependency {
	pub step: ReportingStepId,
	pub dependency: ReportingProductId,
}

#[derive(Debug)]
pub enum ReportingCalculationError {
	UnknownStep { message: String },
	NoStepForProduct { message: String },
	CircularDependencies,
}

pub enum HasStepOrCanBuild<'a, 'b> {
	HasStep(&'a Box<dyn ReportingStep>),
	CanLookup(ReportingStepLookupFn),
	CanBuild(&'b ReportingStepDynamicBuilder),
	None,
}

pub fn has_step_or_can_build<'a, 'b>(
	product: &ReportingProductId,
	steps: &'a Vec<Box<dyn ReportingStep>>,
	dependencies: &ReportingGraphDependencies,
	context: &'b ReportingContext,
) -> HasStepOrCanBuild<'a, 'b> {
	if let Some(step) = steps.iter().find(|s| {
		s.id().name == product.name
			&& s.id().args == product.args
			&& s.id().product_kinds.contains(&product.kind)
	}) {
		return HasStepOrCanBuild::HasStep(step);
	}

	// Try lookup function
	if let Some(lookup_key) = context
		.step_lookup_fn
		.keys()
		.find(|(name, kinds)| *name == product.name && kinds.contains(&product.kind))
	{
		return HasStepOrCanBuild::CanLookup(*context.step_lookup_fn.get(lookup_key).unwrap());
	}

	// No explicit step for product - try builders
	for builder in context.step_dynamic_builders.iter() {
		if (builder.can_build)(
			product.name,
			product.kind,
			&product.args,
			steps,
			dependencies,
			context,
		) {
			return HasStepOrCanBuild::CanBuild(builder);
		}
	}

	return HasStepOrCanBuild::None;
}

fn would_be_ready_to_execute(
	step: &Box<dyn ReportingStep>,
	steps: &Vec<Box<dyn ReportingStep>>,
	dependencies: &ReportingGraphDependencies,
	previous_steps: &Vec<usize>,
) -> bool {
	//println!(
	//	"- would_be_ready_to_execute: {}, {:?}",
	//	step.id(),
	//	previous_steps
	//);

	// Check whether the step would be ready to execute, if the previous steps have already completed
	'check_each_dependency: for dependency in dependencies.vec.iter() {
		if dependency.step == step.id() {
			//println!("-- {}", dependency.dependency);

			// Check if the dependency has been produced by a previous step
			for previous_step in previous_steps {
				if steps[*previous_step].id().name == dependency.dependency.name
					&& steps[*previous_step].id().args == dependency.dependency.args
					&& steps[*previous_step]
						.id()
						.product_kinds
						.contains(&dependency.dependency.kind)
				{
					continue 'check_each_dependency;
				}
			}

			// Dependency is not met
			return false;
		}
	}
	true
}

pub fn solve_for(
	targets: Vec<Box<dyn ReportingStep>>,
	context: ReportingContext,
) -> Result<Vec<Box<dyn ReportingStep>>, ReportingCalculationError> {
	let mut steps: Vec<Box<dyn ReportingStep>> = Vec::new();
	let mut dependencies = ReportingGraphDependencies { vec: Vec::new() };

	// Initialise targets
	for target in targets {
		steps.push(target);
		let target = steps.last().unwrap();
		target.as_ref().init_graph(&steps, &mut dependencies);
	}

	// Call after_init_graph on targets
	for step in steps.iter() {
		step.as_ref().after_init_graph(&steps, &mut dependencies);
	}

	// Process dependencies
	loop {
		let mut new_steps = Vec::new();

		for dependency in dependencies.vec.iter() {
			if !steps.iter().any(|s| s.id() == dependency.step) {
				// FIXME: Call the lookup function
				todo!();
			}
			if !steps.iter().any(|s| {
				s.id().name == dependency.dependency.name
					&& s.id().args == dependency.dependency.args
					&& s.id().product_kinds.contains(&dependency.dependency.kind)
			}) {
				// Try lookup function
				if let Some(lookup_key) = context.step_lookup_fn.keys().find(|(name, kinds)| {
					*name == dependency.dependency.name
						&& kinds.contains(&dependency.dependency.kind)
				}) {
					let lookup_fn = context.step_lookup_fn.get(lookup_key).unwrap();
					let new_step = lookup_fn(dependency.dependency.args.clone());

					// Check new step meets the dependency
					if new_step.id().name != dependency.dependency.name {
						panic!("Unexpected step returned from lookup function (expected name {}, got {})", dependency.dependency.name, new_step.id().name);
					}
					if new_step.id().args != dependency.dependency.args {
						panic!("Unexpected step returned from lookup function {} (expected args {:?}, got {:?})", dependency.dependency.name, dependency.dependency.args, new_step.id().args);
					}
					if !new_step
						.id()
						.product_kinds
						.contains(&dependency.dependency.kind)
					{
						panic!("Unexpected step returned from lookup function {} (expected kind {:?}, got {:?})", dependency.dependency.name, dependency.dependency.kind, new_step.id().product_kinds);
					}

					new_steps.push(new_step);
				} else {
					// No explicit step for product - try builders
					for builder in context.step_dynamic_builders.iter() {
						if (builder.can_build)(
							dependency.dependency.name,
							dependency.dependency.kind,
							&dependency.dependency.args,
							&steps,
							&dependencies,
							&context,
						) {
							new_steps.push((builder.build)(
								dependency.dependency.name,
								dependency.dependency.kind,
								dependency.dependency.args.clone(),
								&steps,
								&dependencies,
								&context,
							));
							break;
						}
					}
				}
			}
		}

		if new_steps.len() == 0 {
			break;
		}

		// Initialise new steps
		let mut new_step_indexes = Vec::new();
		for new_step in new_steps {
			new_step_indexes.push(steps.len());
			steps.push(new_step);
			let new_step = steps.last().unwrap();
			new_step.as_ref().init_graph(&steps, &mut dependencies);
		}

		// Call after_init_graph on new steps
		for new_step_index in new_step_indexes {
			steps[new_step_index].after_init_graph(&steps, &mut dependencies);
		}
	}

	// Check all dependencies satisfied
	for dependency in dependencies.vec.iter() {
		if !steps.iter().any(|s| s.id() == dependency.step) {
			return Err(ReportingCalculationError::UnknownStep {
				message: format!(
					"No implementation for step {} which {} is a dependency of",
					dependency.step, dependency.dependency
				),
			});
		}
		if !steps.iter().any(|s| {
			s.id().name == dependency.dependency.name
				&& s.id().args == dependency.dependency.args
				&& s.id().product_kinds.contains(&dependency.dependency.kind)
		}) {
			return Err(ReportingCalculationError::NoStepForProduct {
				message: format!(
					"No step builds product {} wanted by {}",
					dependency.dependency, dependency.step
				),
			});
		}
	}

	// Sort
	let mut sorted_step_indexes = Vec::new();
	let mut steps_remaining = steps.iter().enumerate().collect::<Vec<_>>();

	'loop_until_all_sorted: while !steps_remaining.is_empty() {
		for (cur_index, (orig_index, step)) in steps_remaining.iter().enumerate() {
			if would_be_ready_to_execute(step, &steps, &dependencies, &sorted_step_indexes) {
				sorted_step_indexes.push(*orig_index);
				steps_remaining.remove(cur_index);
				continue 'loop_until_all_sorted;
			}
		}

		// No steps to execute - must be circular dependency
		return Err(ReportingCalculationError::CircularDependencies);
	}

	let mut sort_mapping = vec![0_usize; sorted_step_indexes.len()];
	for i in 0..sorted_step_indexes.len() {
		sort_mapping[sorted_step_indexes[i]] = i;
	}

	// TODO: This can be done in place
	let mut sorted_steps = steps.into_iter().zip(sort_mapping).collect::<Vec<_>>();
	sorted_steps.sort_unstable_by_key(|(_s, order)| *order);
	let sorted_steps = sorted_steps
		.into_iter()
		.map(|(s, _idx)| s)
		.collect::<Vec<_>>();

	Ok(sorted_steps)
}
