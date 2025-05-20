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

use crate::util::sofy_from_eofy;

use super::{
	calculator::ReportingGraphDependencies, ReportingContext, ReportingProductId,
	ReportingProductKind, ReportingStep, ReportingStepId,
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
	pub date_start: NaiveDate,
	pub date_end: NaiveDate,
}

impl AllTransactionsExceptRetainedEarnings {
	fn from_args(args: Vec<String>) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsExceptRetainedEarnings {
			date_start: NaiveDate::parse_from_str(&args[0], "%Y-%m-%d").unwrap(),
			date_end: NaiveDate::parse_from_str(&args[1], "%Y-%m-%d").unwrap(),
		})
	}
}

impl ReportingStep for AllTransactionsExceptRetainedEarnings {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsExceptRetainedEarnings",
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: vec![
				self.date_start.format("%Y-%m-%d").to_string(),
				self.date_end.format("%Y-%m-%d").to_string(),
			],
		}
	}
}

#[derive(Debug)]
pub struct CalculateIncomeTax {
	pub date_eofy: NaiveDate,
}

impl CalculateIncomeTax {
	fn from_args(args: Vec<String>) -> Box<dyn ReportingStep> {
		Box::new(CalculateIncomeTax {
			date_eofy: NaiveDate::parse_from_str(&args[0], "%Y-%m-%d").unwrap(),
		})
	}
}

impl ReportingStep for CalculateIncomeTax {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CalculateIncomeTax",
			product_kinds: &[ReportingProductKind::Transactions],
			args: vec![self.date_eofy.format("%Y-%m-%d").to_string()],
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
				name: "CombineOrdinaryTransactions",
				kind: ReportingProductKind::BalancesBetween,
				args: vec![
					sofy_from_eofy(self.date_eofy)
						.format("%Y-%m-%d")
						.to_string(),
					self.date_eofy.format("%Y-%m-%d").to_string(),
				],
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
				if other.date_start <= self.date_eofy && other.date_end >= self.date_eofy {
					dependencies.add_target_dependency(other.id(), self.id());
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct CombineOrdinaryTransactions {
	pub date: NaiveDate,
}

impl CombineOrdinaryTransactions {
	fn from_args(args: Vec<String>) -> Box<dyn ReportingStep> {
		Box::new(CombineOrdinaryTransactions {
			date: NaiveDate::parse_from_str(&args[0], "%Y-%m-%d").unwrap(),
		})
	}
}

impl ReportingStep for CombineOrdinaryTransactions {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CombineOrdinaryTransactions",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: vec![self.date.format("%Y-%m-%d").to_string()],
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
				name: "DBBalances",
				kind: ReportingProductKind::BalancesAt,
				args: vec![self.date.format("%Y-%m-%d").to_string()],
			},
		);
	}
}

#[derive(Debug)]
pub struct DBBalances {
	pub date: NaiveDate,
}

impl DBBalances {
	fn from_args(args: Vec<String>) -> Box<dyn ReportingStep> {
		Box::new(DBBalances {
			date: NaiveDate::parse_from_str(&args[0], "%Y-%m-%d").unwrap(),
		})
	}
}

impl ReportingStep for DBBalances {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "DBBalances",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: vec![self.date.format("%Y-%m-%d").to_string()],
		}
	}
}
