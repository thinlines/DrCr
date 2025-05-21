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

use std::fmt::Display;

use crate::util::sofy_from_eofy;

use super::{
	calculator::ReportingGraphDependencies, DateArgs, DateStartDateEndArgs, ReportingContext,
	ReportingProductId, ReportingProductKind, ReportingStep, ReportingStepArgs, ReportingStepId,
	VoidArgs,
};

pub fn register_lookup_fns(context: &mut ReportingContext) {
	context.register_lookup_fn(
		"AllTransactionsExceptRetainedEarnings",
		&[ReportingProductKind::BalancesAt],
		AllTransactionsExceptRetainedEarnings::takes_args,
		|a| {
			AllTransactionsExceptRetainedEarnings::from_args(&[ReportingProductKind::BalancesAt], a)
		},
	);

	context.register_lookup_fn(
		"AllTransactionsExceptRetainedEarnings",
		&[ReportingProductKind::BalancesBetween],
		AllTransactionsExceptRetainedEarnings::takes_args,
		|a| {
			AllTransactionsExceptRetainedEarnings::from_args(
				&[ReportingProductKind::BalancesBetween],
				a,
			)
		},
	);

	context.register_lookup_fn(
		"AllTransactionsIncludingRetainedEarnings",
		&[ReportingProductKind::BalancesAt],
		AllTransactionsIncludingRetainedEarnings::takes_args,
		|a| {
			AllTransactionsIncludingRetainedEarnings::from_args(
				&[ReportingProductKind::BalancesAt],
				a,
			)
		},
	);

	context.register_lookup_fn(
		"AllTransactionsIncludingRetainedEarnings",
		&[ReportingProductKind::BalancesBetween],
		AllTransactionsIncludingRetainedEarnings::takes_args,
		|a| {
			AllTransactionsIncludingRetainedEarnings::from_args(
				&[ReportingProductKind::BalancesBetween],
				a,
			)
		},
	);

	context.register_lookup_fn(
		"CalculateIncomeTax",
		&[ReportingProductKind::Transactions],
		CalculateIncomeTax::takes_args,
		CalculateIncomeTax::from_args,
	);

	context.register_lookup_fn(
		"CombineOrdinaryTransactions",
		&[ReportingProductKind::BalancesAt],
		CombineOrdinaryTransactions::takes_args,
		CombineOrdinaryTransactions::from_args,
	);

	context.register_lookup_fn(
		"DBBalances",
		&[ReportingProductKind::BalancesAt],
		DBBalances::takes_args,
		DBBalances::from_args,
	);

	context.register_lookup_fn(
		"PostUnreconciledStatementLines",
		&[ReportingProductKind::Transactions],
		PostUnreconciledStatementLines::takes_args,
		PostUnreconciledStatementLines::from_args,
	);
}

#[derive(Debug)]
pub struct AllTransactionsExceptRetainedEarnings {
	pub product_kinds: &'static [ReportingProductKind], // Must have single member
	pub args: Box<dyn ReportingStepArgs>,
}

impl AllTransactionsExceptRetainedEarnings {
	fn takes_args(_args: &Box<dyn ReportingStepArgs>) -> bool {
		true
	}

	fn from_args(
		product_kinds: &'static [ReportingProductKind],
		args: Box<dyn ReportingStepArgs>,
	) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsExceptRetainedEarnings {
			product_kinds,
			args,
		})
	}
}

impl Display for AllTransactionsExceptRetainedEarnings {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for AllTransactionsExceptRetainedEarnings {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsExceptRetainedEarnings",
			product_kinds: self.product_kinds,
			args: self.args.clone(),
		}
	}
}

#[derive(Debug)]
pub struct AllTransactionsIncludingRetainedEarnings {
	pub product_kinds: &'static [ReportingProductKind], // Must have single member
	pub args: Box<dyn ReportingStepArgs>,
}

impl AllTransactionsIncludingRetainedEarnings {
	fn takes_args(_args: &Box<dyn ReportingStepArgs>) -> bool {
		true
	}

	fn from_args(
		product_kinds: &'static [ReportingProductKind],
		args: Box<dyn ReportingStepArgs>,
	) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsIncludingRetainedEarnings {
			product_kinds,
			args,
		})
	}
}

impl Display for AllTransactionsIncludingRetainedEarnings {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for AllTransactionsIncludingRetainedEarnings {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsIncludingRetainedEarnings",
			product_kinds: self.product_kinds,
			args: self.args.clone(),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		vec![ReportingProductId {
			name: "AllTransactionsExceptRetainedEarnings",
			kind: self.product_kinds[0],
			args: self.args.clone(),
		}]
	}
}

#[derive(Debug)]
pub struct CalculateIncomeTax {}

impl CalculateIncomeTax {
	fn takes_args(_args: &Box<dyn ReportingStepArgs>) -> bool {
		true
	}

	fn from_args(_args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CalculateIncomeTax {})
	}
}

impl Display for CalculateIncomeTax {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for CalculateIncomeTax {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CalculateIncomeTax",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(VoidArgs {}),
		}
	}

	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		// CalculateIncomeTax depends on CombineOrdinaryTransactions
		vec![ReportingProductId {
			name: "CombineOrdinaryTransactions",
			kind: ReportingProductKind::BalancesBetween,
			args: Box::new(DateStartDateEndArgs {
				date_start: sofy_from_eofy(context.eofy_date),
				date_end: context.eofy_date.clone(),
			}),
		}]
	}

	fn after_init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
		_context: &ReportingContext,
	) {
		for other in steps {
			if let Some(other) = other.downcast_ref::<AllTransactionsExceptRetainedEarnings>() {
				// AllTransactionsExceptRetainedEarnings depends on CalculateIncomeTax
				dependencies.add_dependency(
					other.id(),
					ReportingProductId {
						name: self.id().name,
						kind: other.product_kinds[0],
						args: other.id().args,
					},
				);
			}
		}
	}
}

#[derive(Debug)]
pub struct CombineOrdinaryTransactions {
	pub args: DateArgs,
}

impl CombineOrdinaryTransactions {
	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CombineOrdinaryTransactions {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for CombineOrdinaryTransactions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
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

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		vec![
			// CombineOrdinaryTransactions depends on DBBalances
			ReportingProductId {
				name: "DBBalances",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			// CombineOrdinaryTransactions depends on PostUnreconciledStatementLines
			ReportingProductId {
				name: "PostUnreconciledStatementLines",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
		]
	}
}

#[derive(Debug)]
pub struct DBBalances {
	pub args: DateArgs,
}

impl DBBalances {
	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(DBBalances {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for DBBalances {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
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

#[derive(Debug)]
pub struct PostUnreconciledStatementLines {
	pub args: DateArgs,
}

impl PostUnreconciledStatementLines {
	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(PostUnreconciledStatementLines {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for PostUnreconciledStatementLines {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for PostUnreconciledStatementLines {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "PostUnreconciledStatementLines",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}
}
