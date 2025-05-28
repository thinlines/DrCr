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

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use async_trait::async_trait;
use chrono::NaiveDate;
use downcast_rs::Downcast;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::db::DbConnection;
use crate::model::transaction::TransactionWithPostings;
use crate::QuantityInt;

use super::calculator::ReportingGraphDependencies;
use super::executor::ReportingExecutionError;

// -----------------
// REPORTING CONTEXT

/// Records the context for a single reporting job
pub struct ReportingContext {
	// Configuration
	pub db_connection: DbConnection,
	pub eofy_date: NaiveDate,
	pub reporting_commodity: String,

	// State
	pub(crate) step_lookup_fn: HashMap<
		(&'static str, &'static [ReportingProductKind]),
		(ReportingStepTakesArgsFn, ReportingStepFromArgsFn),
	>,
	pub(crate) step_dynamic_builders: Vec<ReportingStepDynamicBuilder>,
}

impl ReportingContext {
	/// Initialise a new [ReportingContext]
	pub fn new(
		db_connection: DbConnection,
		eofy_date: NaiveDate,
		reporting_commodity: String,
	) -> Self {
		Self {
			db_connection,
			eofy_date,
			reporting_commodity,
			step_lookup_fn: HashMap::new(),
			step_dynamic_builders: Vec::new(),
		}
	}

	/// Register a lookup function
	///
	/// A lookup function generates concrete [ReportingStep]s from a [ReportingStepId].
	pub fn register_lookup_fn(
		&mut self,
		name: &'static str,
		product_kinds: &'static [ReportingProductKind],
		takes_args_fn: ReportingStepTakesArgsFn,
		from_args_fn: ReportingStepFromArgsFn,
	) {
		self.step_lookup_fn
			.insert((name, product_kinds), (takes_args_fn, from_args_fn));
	}

	/// Register a dynamic builder
	///
	/// Dynamic builders are called when no concrete [ReportingStep] is implemented, and can dynamically generate a [ReportingStep]. Dynamic builders are implemented in [super::builders].
	pub fn register_dynamic_builder(&mut self, builder: ReportingStepDynamicBuilder) {
		if !self
			.step_dynamic_builders
			.iter()
			.any(|b| b.name == builder.name)
		{
			self.step_dynamic_builders.push(builder);
		}
	}
}

/// Function which determines whether the [ReportingStepArgs] are valid arguments for a given [ReportingStep]
///
/// See [ReportingContext::register_lookup_fn].
pub type ReportingStepTakesArgsFn = fn(args: &Box<dyn ReportingStepArgs>) -> bool;

/// Function which builds a concrete [ReportingStep] from the given [ReportingStepArgs]
///
/// See [ReportingContext::register_lookup_fn].
pub type ReportingStepFromArgsFn = fn(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep>;

// -------------------------------
// REPORTING STEP DYNAMIC BUILDERS

/// Represents a reporting step dynamic builder
///
/// See [ReportingContext::register_dynamic_builder].
pub struct ReportingStepDynamicBuilder {
	pub name: &'static str,
	pub can_build: fn(
		name: &'static str,
		kind: ReportingProductKind,
		args: &Box<dyn ReportingStepArgs>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		context: &ReportingContext,
	) -> bool,
	pub build: fn(
		name: &'static str,
		kind: ReportingProductKind,
		args: Box<dyn ReportingStepArgs>,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		context: &ReportingContext,
	) -> Box<dyn ReportingStep>,
}

// ------------------
// REPORTING PRODUCTS

/// Identifies a [ReportingProduct]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReportingProductId {
	pub name: &'static str,
	pub kind: ReportingProductKind,
	pub args: Box<dyn ReportingStepArgs>,
}

impl Display for ReportingProductId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}.{:?}({})", self.name, self.kind, self.args))
	}
}

/// Identifies a type of [ReportingProduct]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReportingProductKind {
	Transactions,
	BalancesAt,
	BalancesBetween,
	Generic,
}

/// Represents the result of a [ReportingStep]
pub trait ReportingProduct: Debug + Downcast + DynClone + Send + Sync {}

downcast_rs::impl_downcast!(ReportingProduct);
dyn_clone::clone_trait_object!(ReportingProduct);

/// Records a list of transactions generated by a [ReportingStep]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transactions {
	pub transactions: Vec<TransactionWithPostings>,
}

impl ReportingProduct for Transactions {}

/// Records cumulative account balances at a particular point in time
#[derive(Clone, Debug)]
pub struct BalancesAt {
	pub balances: HashMap<String, QuantityInt>,
}

impl ReportingProduct for BalancesAt {}

/// Records the total value of transactions in each account between two points in time
#[derive(Clone, Debug)]
pub struct BalancesBetween {
	pub balances: HashMap<String, QuantityInt>,
}

impl ReportingProduct for BalancesBetween {}

/// Map from [ReportingProductId] to [ReportingProduct]
#[derive(Clone, Debug)]
pub struct ReportingProducts {
	// This needs to be an IndexMap not HashMap, because sometimes we query which product is more up to date
	map: IndexMap<ReportingProductId, Box<dyn ReportingProduct>>,
}

impl ReportingProducts {
	pub fn new() -> Self {
		Self {
			map: IndexMap::new(),
		}
	}

	/// Returns a reference to the underlying [IndexMap]
	pub fn map(&self) -> &IndexMap<ReportingProductId, Box<dyn ReportingProduct>> {
		&self.map
	}

	/// Insert a key-value pair in the map
	///
	/// See [IndexMap::insert].
	pub fn insert(&mut self, key: ReportingProductId, value: Box<dyn ReportingProduct>) {
		self.map.insert(key, value);
	}

	/// Moves all key-value pairs from `other` into `self`, leaving `other` empty
	///
	/// See [IndexMap::append].
	pub fn append(&mut self, other: &mut ReportingProducts) {
		self.map.append(&mut other.map);
	}

	pub fn get_or_err(
		&self,
		key: &ReportingProductId,
	) -> Result<&Box<dyn ReportingProduct>, ReportingExecutionError> {
		match self.map.get(key) {
			Some(value) => Ok(value),
			None => Err(ReportingExecutionError::DependencyNotAvailable {
				message: format!("Product {} not available when expected", key),
			}),
		}
	}

	pub fn get_owned_or_err(
		mut self,
		key: &ReportingProductId,
	) -> Result<Box<dyn ReportingProduct>, ReportingExecutionError> {
		match self.map.swap_remove(key) {
			Some(value) => Ok(value),
			None => Err(ReportingExecutionError::DependencyNotAvailable {
				message: format!("Product {} not available when expected", key),
			}),
		}
	}
}

impl Display for ReportingProducts {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"ReportingProducts {{\n{}\n}}",
			self.map
				.iter()
				.map(|(k, v)| format!("  {}: {:?}", k, v))
				.collect::<Vec<_>>()
				.join(",\n")
		))
	}
}

// ---------------
// REPORTING STEPS

/// Identifies a [ReportingStep]
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

/// Represents a step in a reporting job
#[async_trait]
pub trait ReportingStep: Debug + Display + Downcast + Send + Sync {
	/// Get the [ReportingStepId] for this [ReportingStep]
	fn id(&self) -> ReportingStepId;

	/// Return a list of statically defined dependencies for this [ReportingStep]
	#[allow(unused_variables)]
	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		vec![]
	}

	/// Called when the [ReportingStep] is initialised in [super::calculator::steps_for_targets]
	#[allow(unused_variables)]
	fn init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
		context: &ReportingContext,
	) {
	}

	/// Called when new [ReportingStep]s are initialised in [super::calculator::steps_for_targets]
	///
	/// This callback can be used to dynamically declare dependencies between [ReportingStep]s that are not known at initialisation.
	#[allow(unused_variables)]
	fn after_init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
		context: &ReportingContext,
	) {
	}

	/// Called to generate the [ReportingProduct] for this [ReportingStep]
	///
	/// Returns a [ReportingProducts] containing (only) the new [ReportingProduct]s.
	#[allow(unused_variables)]
	async fn execute(
		&self,
		context: &ReportingContext,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		todo!("{}", self);
	}
}

downcast_rs::impl_downcast!(ReportingStep);

// ------------------------
// REPORTING STEP ARGUMENTS

/// Represents arguments to a [ReportingStep]
pub trait ReportingStepArgs:
	Debug + Display + Downcast + DynClone + DynEq + DynHash + Send + Sync
{
}

downcast_rs::impl_downcast!(ReportingStepArgs);
dyn_clone::clone_trait_object!(ReportingStepArgs);
dyn_eq::eq_trait_object!(ReportingStepArgs);
dyn_hash::hash_trait_object!(ReportingStepArgs);

/// [ReportingStepArgs] implementation which takes no arguments
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VoidArgs {}

impl ReportingStepArgs for VoidArgs {}

impl Display for VoidArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(""))
	}
}

/// [ReportingStepArgs] implementation which takes a single date
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DateArgs {
	pub date: NaiveDate,
}

impl ReportingStepArgs for DateArgs {}

impl Display for DateArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.date))
	}
}

/// [ReportingStepArgs] implementation which takes a date range
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

/// [ReportingStepArgs] implementation which takes multiple [DateArgs]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MultipleDateArgs {
	pub dates: Vec<DateArgs>,
}

impl ReportingStepArgs for MultipleDateArgs {}

impl Display for MultipleDateArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"{}",
			self.dates
				.iter()
				.map(|a| a.to_string())
				.collect::<Vec<_>>()
				.join(", ")
		))
	}
}

/// [ReportingStepArgs] implementation which takes multiple [DateStartDateEndArgs]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MultipleDateStartDateEndArgs {
	pub dates: Vec<DateStartDateEndArgs>,
}

impl ReportingStepArgs for MultipleDateStartDateEndArgs {}

impl Display for MultipleDateStartDateEndArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!(
			"{}",
			self.dates
				.iter()
				.map(|a| format!("({})", a))
				.collect::<Vec<_>>()
				.join(", ")
		))
	}
}
