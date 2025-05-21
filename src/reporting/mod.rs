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

use std::fmt::Debug;
use std::{collections::HashMap, fmt::Display};

use calculator::ReportingGraphDependencies;
use chrono::NaiveDate;
use downcast_rs::Downcast;
use dyn_clone::DynClone;
use dyn_eq::DynEq;

pub mod builders;
pub mod calculator;
pub mod steps;

pub struct ReportingContext {
	_eofy_date: NaiveDate,
	step_lookup_fn: HashMap<(&'static str, &'static [ReportingProductKind]), ReportingStepLookupFn>,
	step_dynamic_builders: Vec<ReportingStepDynamicBuilder>,
}

impl ReportingContext {
	pub fn new(eofy_date: NaiveDate) -> Self {
		Self {
			_eofy_date: eofy_date,
			step_lookup_fn: HashMap::new(),
			step_dynamic_builders: Vec::new(),
		}
	}

	fn register_lookup_fn(
		&mut self,
		name: &'static str,
		product_kinds: &'static [ReportingProductKind],
		builder: ReportingStepLookupFn,
	) {
		self.step_lookup_fn.insert((name, product_kinds), builder);
	}

	fn register_dynamic_builder(&mut self, builder: ReportingStepDynamicBuilder) {
		if !self
			.step_dynamic_builders
			.iter()
			.any(|b| b.name == builder.name)
		{
			self.step_dynamic_builders.push(builder);
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct ReportingProductId {
	name: &'static str,
	kind: ReportingProductKind,
	args: Box<dyn ReportingStepArgs>,
}

impl Display for ReportingProductId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}.{:?}({})", self.name, self.kind, self.args))
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReportingProductKind {
	Transactions,
	BalancesAt,
	BalancesBetween,
	Generic,
}

//enum ReportingProduct {
//	Transactions(Transactions),
//	BalancesAt(BalancesAt),
//	BalancesBetween(BalancesBetween),
//	Generic(Box<dyn GenericReportingProduct>),
//}

//struct Transactions {}
//struct BalancesAt {}
//struct BalancesBetween {}

//trait GenericReportingProduct {}

//type ReportingProducts = HashMap<ReportingProductId, ReportingProduct>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportingStepId {
	pub name: &'static str,
	pub product_kinds: &'static [ReportingProductKind],
	pub args: Box<dyn ReportingStepArgs>,
}

impl Display for ReportingStepId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"{}{:?}({})",
			self.name, self.product_kinds, self.args
		))
	}
}

pub trait ReportingStep: Debug + Downcast {
	// Info
	fn id(&self) -> ReportingStepId;

	// Methods
	fn init_graph(
		&self,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &mut ReportingGraphDependencies,
	) {
	}
	fn after_init_graph(
		&self,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &mut ReportingGraphDependencies,
	) {
	}
	//fn execute(&self, _context: &ReportingContext, _products: &mut ReportingProducts) {
	//	todo!();
	//}
}

downcast_rs::impl_downcast!(ReportingStep);

pub trait ReportingStepArgs: Debug + Display + Downcast + DynClone + DynEq {}

downcast_rs::impl_downcast!(ReportingStepArgs);
dyn_clone::clone_trait_object!(ReportingStepArgs);
dyn_eq::eq_trait_object!(ReportingStepArgs);

pub type ReportingStepLookupFn = fn(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateArgs {
	pub date: NaiveDate,
}

impl ReportingStepArgs for DateArgs {}

impl Display for DateArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.date))
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateEofyArgs {
	pub date_eofy: NaiveDate,
}

impl ReportingStepArgs for DateEofyArgs {}

impl Display for DateEofyArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.date_eofy))
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateStartDateEndArgs {
	pub date_start: NaiveDate,
	pub date_end: NaiveDate,
}

impl ReportingStepArgs for DateStartDateEndArgs {}

impl Display for DateStartDateEndArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}, {}", self.date_start, self.date_end))
	}
}

pub struct ReportingStepDynamicBuilder {
	name: &'static str,
	can_build: fn(
		name: &'static str,
		kind: ReportingProductKind,
		args: &Box<dyn ReportingStepArgs>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		context: &ReportingContext,
	) -> bool,
	build: fn(
		name: &'static str,
		kind: ReportingProductKind,
		args: Box<dyn ReportingStepArgs>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		context: &ReportingContext,
	) -> Box<dyn ReportingStep>,
}
