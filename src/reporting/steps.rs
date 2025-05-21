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

use crate::util::sofy_from_eofy;

use super::{
	calculator::ReportingGraphDependencies, DateArgs, DateEofyArgs, DateStartDateEndArgs,
	ReportingContext, ReportingProductId, ReportingProductKind, ReportingStep, ReportingStepArgs,
	ReportingStepId,
};

pub fn register_lookup_fns(context: &mut ReportingContext) {
	context.register_lookup_fn(
		"AllTransactionsExceptRetainedEarnings",
		&[ReportingProductKind::BalancesBetween],
		AllTransactionsExceptRetainedEarnings::from_args,
	);

	context.register_lookup_fn(
		"CalculateIncomeTax",
		&[ReportingProductKind::Transactions],
		CalculateIncomeTax::from_args,
	);

	context.register_lookup_fn(
		"CombineOrdinaryTransactions",
		&[ReportingProductKind::BalancesAt],
		CombineOrdinaryTransactions::from_args,
	);

	context.register_lookup_fn(
		"DBBalances",
		&[ReportingProductKind::BalancesAt],
		DBBalances::from_args,
	);
}

#[derive(Debug)]
pub struct AllTransactionsExceptRetainedEarnings {
	pub args: DateStartDateEndArgs,
}

impl AllTransactionsExceptRetainedEarnings {
	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsExceptRetainedEarnings {
			args: *args.downcast().unwrap(),
		})
	}
}

impl ReportingStep for AllTransactionsExceptRetainedEarnings {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsExceptRetainedEarnings",
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: Box::new(self.args.clone()),
		}
	}
}

#[derive(Debug)]
pub struct CalculateIncomeTax {
	pub args: DateEofyArgs,
}

impl CalculateIncomeTax {
	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CalculateIncomeTax {
			args: *args.downcast().unwrap(),
		})
	}
}

impl ReportingStep for CalculateIncomeTax {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CalculateIncomeTax",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}

	fn init_graph(
		&self,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
	) {
		// CalculateIncomeTax depends on CombineOrdinaryTransactions
		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: "CombineOrdinaryTransactions",
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(DateStartDateEndArgs {
					date_start: sofy_from_eofy(self.args.date_eofy),
					date_end: self.args.date_eofy.clone(),
				}),
			},
		);
	}

	fn after_init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
	) {
		for other in steps {
			if let Some(other) = other.downcast_ref::<AllTransactionsExceptRetainedEarnings>() {
				if other.args.date_start <= self.args.date_eofy
					&& other.args.date_end >= self.args.date_eofy
				{
					// AllTransactionsExceptRetainedEarnings (in applicable periods) depends on CalculateIncomeTax
					dependencies.add_target_dependency(other.id(), self.id());
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct CombineOrdinaryTransactions {
	pub args: DateArgs,
}

impl CombineOrdinaryTransactions {
	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CombineOrdinaryTransactions {
			args: *args.downcast().unwrap(),
		})
	}
}

impl ReportingStep for CombineOrdinaryTransactions {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CombineOrdinaryTransactions",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	fn init_graph(
		&self,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
	) {
		// CombineOrdinaryTransactions depends on DBBalances
		dependencies.add_dependency(
			self.id(),
			ReportingProductId {
				name: "DBBalances",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
		);
	}
}

#[derive(Debug)]
pub struct DBBalances {
	pub args: DateArgs,
}

impl DBBalances {
	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(DBBalances {
			args: *args.downcast().unwrap(),
		})
	}
}

impl ReportingStep for DBBalances {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "DBBalances",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}
}
