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

//! This module contains implementations of dynamic step builders
//!
//! See [ReportingContext::register_dynamic_builder][super::types::ReportingContext::register_dynamic_builder].

use std::collections::HashMap;
use std::fmt::Display;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::model::transaction::update_balances_from_transactions;

use super::calculator::{has_step_or_can_build, HasStepOrCanBuild, ReportingGraphDependencies};
use super::executor::ReportingExecutionError;
use super::types::{
	BalancesAt, BalancesBetween, DateArgs, DateStartDateEndArgs, ReportingContext,
	ReportingProductId, ReportingProductKind, ReportingProducts, ReportingStep, ReportingStepArgs,
	ReportingStepDynamicBuilder, ReportingStepId, Transactions, VoidArgs,
};

/// Call [ReportingContext::register_dynamic_builder] for all dynamic builders provided by this module
pub fn register_dynamic_builders(context: &mut ReportingContext) {
	GenerateBalances::register_dynamic_builder(context);
	UpdateBalancesBetween::register_dynamic_builder(context);
	UpdateBalancesAt::register_dynamic_builder(context);

	// This is the least efficient way of generating BalancesBetween so put at the end
	BalancesAtToBalancesBetween::register_dynamic_builder(context);
}

/// This dynamic builder automatically generates a [BalancesBetween] by subtracting [BalancesAt] between two dates
#[derive(Debug)]
pub struct BalancesAtToBalancesBetween {
	step_name: &'static str,
	args: DateStartDateEndArgs,
}

impl BalancesAtToBalancesBetween {
	// Implements BalancesAt, BalancesAt -> BalancesBetween

	fn register_dynamic_builder(context: &mut ReportingContext) {
		context.register_dynamic_builder(ReportingStepDynamicBuilder {
			name: "BalancesAtToBalancesBetween",
			can_build: Self::can_build,
			build: Self::build,
		});
	}

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
			if !args.is::<DateStartDateEndArgs>() {
				return false;
			}

			let args = args.downcast_ref::<DateStartDateEndArgs>().unwrap();

			match has_step_or_can_build(
				&ReportingProductId {
					name,
					kind: ReportingProductKind::BalancesAt,
					args: Box::new(DateArgs {
						date: args.date_start.clone(),
					}),
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

impl Display for BalancesAtToBalancesBetween {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"{} {{BalancesAtToBalancesBetween}}",
			self.id()
		))
	}
}

#[async_trait]
impl ReportingStep for BalancesAtToBalancesBetween {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.step_name,
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		// BalancesAtToBalancesBetween depends on BalancesAt at both time points
		vec![
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: self.args.date_start.pred_opt().unwrap(), // Opening balance is the closing balance of the preceding day
				}),
			},
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: self.args.date_end,
				}),
			},
		]
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get balances at dates
		let balances_start = &products
			.get_or_err(&ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: self.args.date_start.pred_opt().unwrap(), // Opening balance is the closing balance of the preceding day
				}),
			})?
			.downcast_ref::<BalancesAt>()
			.unwrap()
			.balances;

		let balances_end = &products
			.get_or_err(&ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: self.args.date_end,
				}),
			})?
			.downcast_ref::<BalancesAt>()
			.unwrap()
			.balances;

		// Compute balances_end - balances_start
		let mut balances = BalancesBetween {
			balances: balances_end.clone(),
		};

		for (account, balance) in balances_start.iter() {
			let running_balance = balances.balances.get(account).unwrap_or(&0) - balance;
			balances.balances.insert(account.clone(), running_balance);
		}

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);
		Ok(result)
	}
}

/// This dynamic builder automatically generates a [BalancesAt] from a step which has no dependencies and generates [Transactions] (e.g. [PostUnreconciledStatementLines][super::steps::PostUnreconciledStatementLines])
#[derive(Debug)]
pub struct GenerateBalances {
	step_name: &'static str,
	args: DateArgs,
}

impl GenerateBalances {
	fn register_dynamic_builder(context: &mut ReportingContext) {
		context.register_dynamic_builder(ReportingStepDynamicBuilder {
			name: "GenerateBalances",
			can_build: Self::can_build,
			build: Self::build,
		});
	}

	fn can_build(
		name: &'static str,
		kind: ReportingProductKind,
		args: &Box<dyn ReportingStepArgs>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		context: &ReportingContext,
	) -> bool {
		// Check for Transactions -> BalancesAt
		if kind == ReportingProductKind::BalancesAt {
			// Try DateArgs
			match has_step_or_can_build(
				&ReportingProductId {
					name,
					kind: ReportingProductKind::Transactions,
					args: args.clone(),
				},
				steps,
				dependencies,
				context,
			) {
				HasStepOrCanBuild::HasStep(step) => {
					// Check for () -> Transactions
					if dependencies.dependencies_for_step(&step.id()).len() == 0 {
						return true;
					}
				}
				HasStepOrCanBuild::CanLookup(lookup_fn) => {
					// Check for () -> Transactions
					let step = lookup_fn(args.clone());
					if step.requires(context).len() == 0 {
						return true;
					}
				}
				HasStepOrCanBuild::CanBuild(_) | HasStepOrCanBuild::None => {}
			}

			// Try VoidArgs
			match has_step_or_can_build(
				&ReportingProductId {
					name,
					kind: ReportingProductKind::Transactions,
					args: Box::new(VoidArgs {}),
				},
				steps,
				dependencies,
				context,
			) {
				HasStepOrCanBuild::HasStep(step) => {
					// Check for () -> Transactions
					if dependencies.dependencies_for_step(&step.id()).len() == 0 {
						return true;
					}
				}
				HasStepOrCanBuild::CanLookup(lookup_fn) => {
					// Check for () -> Transactions
					let step = lookup_fn(args.clone());
					if step.requires(context).len() == 0 {
						return true;
					}
				}
				HasStepOrCanBuild::CanBuild(_) | HasStepOrCanBuild::None => {}
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
		Box::new(GenerateBalances {
			step_name: name,
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for GenerateBalances {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{} {{GenerateBalances}}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for GenerateBalances {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.step_name,
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	fn init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
		context: &ReportingContext,
	) {
		// Add a dependency on the Transactions result
		// Look up that step, so we can extract the appropriate args

		// Try DateArgs
		match has_step_or_can_build(
			&ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
			steps,
			dependencies,
			context,
		) {
			HasStepOrCanBuild::HasStep(_)
			| HasStepOrCanBuild::CanLookup(_)
			| HasStepOrCanBuild::CanBuild(_) => {
				dependencies.add_dependency(
					self.id(),
					ReportingProductId {
						name: self.step_name,
						kind: ReportingProductKind::Transactions,
						args: Box::new(self.args.clone()),
					},
				);
				return;
			}
			HasStepOrCanBuild::None => (),
		}

		// Must be VoidArgs (as checked in can_build)
		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
		);
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get the transactions
		let transactions_product = &dependencies.dependencies_for_step(&self.id())[0].product;
		let transactions = &products
			.get_or_err(transactions_product)?
			.downcast_ref::<Transactions>()
			.unwrap()
			.transactions;

		// Sum balances
		let mut balances = BalancesAt {
			balances: HashMap::new(),
		};
		update_balances_from_transactions(&mut balances.balances, transactions.iter());

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);
		Ok(result)
	}
}

/// This dynamic builder automatically generates a [BalancesAt] from:
/// - a step which generates [Transactions] from [BalancesAt], or
/// - a step which generates [Transactions] from [BalancesBetween], and for which a [BalancesAt] is also available
#[derive(Debug)]
pub struct UpdateBalancesAt {
	step_name: &'static str,
	args: DateArgs,
}

impl UpdateBalancesAt {
	// Implements (BalancesAt -> Transactions) -> BalancesAt

	fn register_dynamic_builder(context: &mut ReportingContext) {
		context.register_dynamic_builder(ReportingStepDynamicBuilder {
			name: "UpdateBalancesAt",
			can_build: Self::can_build,
			build: Self::build,
		});
	}

	fn can_build(
		name: &'static str,
		kind: ReportingProductKind,
		args: &Box<dyn ReportingStepArgs>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		context: &ReportingContext,
	) -> bool {
		if !args.is::<DateArgs>() {
			return false;
		}

		// Check for Transactions -> BalancesAt
		if kind == ReportingProductKind::BalancesAt {
			// Initially no need to check args
			if let Some(step) = steps.iter().find(|s| {
				s.id().name == name
					&& s.id()
						.product_kinds
						.contains(&ReportingProductKind::Transactions)
			}) {
				// Check for BalancesAt -> Transactions
				let dependencies_for_step = dependencies.dependencies_for_step(&step.id());
				if dependencies_for_step.len() == 1
					&& dependencies_for_step[0].product.kind == ReportingProductKind::BalancesAt
				{
					return true;
				}

				// Check if BalancesBetween -> Transactions and BalancesAt is available
				if dependencies_for_step.len() == 1
					&& dependencies_for_step[0].product.kind
						== ReportingProductKind::BalancesBetween
				{
					match has_step_or_can_build(
						&ReportingProductId {
							name: dependencies_for_step[0].product.name,
							kind: ReportingProductKind::BalancesAt,
							args: Box::new(DateArgs {
								date: args.downcast_ref::<DateArgs>().unwrap().date,
							}),
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
		Box::new(UpdateBalancesAt {
			step_name: name,
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for UpdateBalancesAt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{} {{UpdateBalancesAt}}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for UpdateBalancesAt {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.step_name,
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	fn init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
		_context: &ReportingContext,
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

		// Look up the BalancesAt step
		let dependencies_for_step = dependencies.dependencies_for_step(&parent_step.id());
		let dependency = &dependencies_for_step[0].product; // Existence and uniqueness checked in can_build

		if dependency.kind == ReportingProductKind::BalancesAt {
			// Directly depends on BalancesAt -> Transaction
			// Do not need to add extra dependencies
		} else {
			// As checked in can_build, must depend on BalancesBetween -> Transaction with a BalancesAt available
			dependencies.add_dependency(
				self.id(),
				ReportingProductId {
					name: dependency.name,
					kind: ReportingProductKind::BalancesAt,
					args: Box::new(DateArgs {
						date: self.args.date,
					}),
				},
			);
		}
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Look up the parent step, so we can extract the appropriate args
		let parent_step = steps
			.iter()
			.find(|s| {
				s.id().name == self.step_name
					&& s.id()
						.product_kinds
						.contains(&ReportingProductKind::Transactions)
			})
			.unwrap(); // Existence is checked in can_build

		// Get transactions
		let transactions = &products
			.get_or_err(&ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::Transactions,
				args: parent_step.id().args,
			})?
			.downcast_ref::<Transactions>()
			.unwrap()
			.transactions;

		// Look up the BalancesAt step
		let dependencies_for_step = dependencies.dependencies_for_step(&parent_step.id());
		let dependency = &dependencies_for_step[0].product; // Existence and uniqueness checked in can_build

		let opening_balances_at;

		if dependency.kind == ReportingProductKind::BalancesAt {
			// Directly depends on BalancesAt -> Transaction
			opening_balances_at = products
				.get_or_err(&dependency)?
				.downcast_ref::<BalancesAt>()
				.unwrap();
		} else {
			// As checked in can_build, must depend on BalancesBetween -> Transaction with a BalancesAt available
			opening_balances_at = products
				.get_or_err(&ReportingProductId {
					name: dependency.name,
					kind: ReportingProductKind::BalancesAt,
					args: Box::new(DateArgs {
						date: self.args.date,
					}),
				})?
				.downcast_ref()
				.unwrap();
		}

		// Sum balances
		let mut balances = BalancesAt {
			balances: opening_balances_at.balances.clone(),
		};
		update_balances_from_transactions(
			&mut balances.balances,
			transactions
				.iter()
				.filter(|t| t.transaction.dt.date() <= self.args.date),
		);

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);
		Ok(result)
	}
}

/// This dynamic builder automatically generates a [BalancesBetween] from a step which generates [Transactions] from [BalancesBetween]
#[derive(Debug)]
pub struct UpdateBalancesBetween {
	step_name: &'static str,
	args: DateStartDateEndArgs,
}

impl UpdateBalancesBetween {
	fn register_dynamic_builder(context: &mut ReportingContext) {
		context.register_dynamic_builder(ReportingStepDynamicBuilder {
			name: "UpdateBalancesBetween",
			can_build: Self::can_build,
			build: Self::build,
		});
	}

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
					&& dependencies_for_step[0].product.kind
						== ReportingProductKind::BalancesBetween
				{
					return true;
				}
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
		Box::new(UpdateBalancesBetween {
			step_name: name,
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for UpdateBalancesBetween {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{} {{UpdateBalancesBetween}}", self.id()))
	}
}

#[async_trait]
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
		_context: &ReportingContext,
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
				args: parent_step.id().args,
			},
		);

		// Look up the BalancesBetween step
		let dependencies_for_step = dependencies.dependencies_for_step(&parent_step.id());
		let balances_between_product = &dependencies_for_step[0].product; // Existence and uniqueness checked in can_build

		if *balances_between_product
			.args
			.downcast_ref::<DateStartDateEndArgs>()
			.unwrap() == self.args
		{
			// Directly depends on BalanceBetween -> Transaction with appropriate date
			// Do not need to add extra dependencies
		} else {
			// Depends on BalanceBetween with appropriate date
			dependencies.add_dependency(
				self.id(),
				ReportingProductId {
					name: balances_between_product.name,
					kind: ReportingProductKind::BalancesBetween,
					args: Box::new(self.args.clone()),
				},
			);
		}
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Look up the parent step, so we can extract the appropriate args
		let parent_step = steps
			.iter()
			.find(|s| {
				s.id().name == self.step_name
					&& s.id()
						.product_kinds
						.contains(&ReportingProductKind::Transactions)
			})
			.unwrap(); // Existence is checked in can_build

		// Get transactions
		let transactions = &products
			.get_or_err(&ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::Transactions,
				args: parent_step.id().args,
			})?
			.downcast_ref::<Transactions>()
			.unwrap()
			.transactions;

		// Look up the BalancesBetween step
		let dependencies_for_step = dependencies.dependencies_for_step(&parent_step.id());
		let balances_between_product = &dependencies_for_step[0].product; // Existence and uniqueness is checked in can_build

		// Get opening balances
		let opening_balances = &products
			.get_or_err(&ReportingProductId {
				name: balances_between_product.name,
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(self.args.clone()),
			})?
			.downcast_ref::<BalancesBetween>()
			.unwrap()
			.balances;

		// Sum balances
		let mut balances = BalancesBetween {
			balances: opening_balances.clone(),
		};
		update_balances_from_transactions(
			&mut balances.balances,
			transactions.iter().filter(|t| {
				t.transaction.dt.date() >= self.args.date_start
					&& t.transaction.dt.date() <= self.args.date_end
			}),
		);

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.step_name,
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);
		Ok(result)
	}
}
