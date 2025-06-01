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

use async_trait::async_trait;
use chrono::NaiveDate;
use mlua::{FromLua, Function, Lua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::account_config::kinds_for_account;
use crate::reporting::calculator::ReportingGraphDependencies;
use crate::reporting::dynamic_report::DynamicReport;
use crate::reporting::executor::ReportingExecutionError;
use crate::reporting::types::{
	BalancesAt, BalancesBetween, ReportingContext, ReportingProduct, ReportingProductId,
	ReportingProductKind, ReportingProducts, ReportingStep, ReportingStepArgs, ReportingStepId,
	Transactions,
};
use crate::util::sofy_from_eofy;

fn load_plugin(plugin_dir: &str, plugin_name: &str) -> (Lua, Plugin) {
	let lua = Lua::new();

	// Init Lua environment
	let package = lua.globals().get::<Table>("package").unwrap();
	package
		.set("path", format!("{}/?.luau", plugin_dir))
		.unwrap();

	// Require and call the plugin
	let require = lua.load("require").eval::<Function>().unwrap();
	let plugin = require.call::<Plugin>(plugin_name).expect("Lua error");

	(lua, plugin)
}

/// Call [ReportingContext::register_lookup_fn] for all steps provided by this module
pub fn register_lookup_fns(context: &mut ReportingContext) {
	for plugin_path in context.plugin_names.clone().iter() {
		let (_, plugin) = load_plugin(&context.plugin_dir, plugin_path);

		for reporting_step in plugin.reporting_steps.iter() {
			context.register_lookup_fn(
				reporting_step.spec.name.clone(),
				reporting_step.spec.product_kinds.clone(),
				PluginReportingStep::takes_args,
				PluginReportingStep::from_args,
			);
		}

		context
			.plugin_specs
			.insert(plugin_path.clone(), plugin.into());
	}
}

/// Represents a libdrcr plugin specification and implementation
#[derive(Debug)]
pub struct Plugin {
	name: String,
	reporting_steps: Vec<LuaReportingStep>,
}

impl FromLua for Plugin {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		let value = value.as_table().unwrap();
		Ok(Self {
			name: value.get("name")?,
			reporting_steps: value.get("reporting_steps")?,
		})
	}
}

/// Represents a libdrcr plugin specification
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginSpec {
	name: String,
	reporting_steps: Vec<ReportingStepSpec>,
}

impl From<Plugin> for PluginSpec {
	fn from(value: Plugin) -> Self {
		Self {
			name: value.name,
			reporting_steps: value.reporting_steps.into_iter().map(|s| s.spec).collect(),
		}
	}
}

/// [ReportingStep] provided by the plugin specification and implementation
#[derive(Debug)]
pub struct LuaReportingStep {
	spec: ReportingStepSpec,
	requires: Function,
	after_init_graph: Function,
	execute: Function,
}

impl FromLua for LuaReportingStep {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let value = value.as_table().unwrap();
		Ok(Self {
			spec: ReportingStepSpec {
				name: value.get("name")?,
				product_kinds: lua.from_value(value.get("product_kinds")?)?,
			},
			requires: value.get("requires")?,
			after_init_graph: value.get("after_init_graph")?,
			execute: value.get("execute")?,
		})
	}
}

/// [ReportingStep] provided by the plugin specification
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReportingStepSpec {
	name: String,
	product_kinds: Vec<ReportingProductKind>,
}

/// Represents a [ReportingProduct] which can be represented in Lua
#[derive(Deserialize, Serialize)]
enum LuaReportingProduct {
	BalancesAt(BalancesAt),
	BalancesBetween(BalancesBetween),
	Transactions(Transactions),
	DynamicReport(DynamicReport),
}

impl Into<LuaReportingProduct> for Box<dyn ReportingProduct> {
	fn into(self) -> LuaReportingProduct {
		if self.is::<BalancesAt>() {
			LuaReportingProduct::BalancesAt(*self.downcast().unwrap())
		} else if self.is::<BalancesBetween>() {
			LuaReportingProduct::BalancesBetween(*self.downcast().unwrap())
		} else if self.is::<Transactions>() {
			LuaReportingProduct::Transactions(*self.downcast().unwrap())
		} else if self.is::<DynamicReport>() {
			LuaReportingProduct::DynamicReport(*self.downcast().unwrap())
		} else {
			panic!("Attempt to convert unknown ReportingProduct type into LuaReportingProduct")
		}
	}
}

impl Into<Box<dyn ReportingProduct>> for LuaReportingProduct {
	fn into(self) -> Box<dyn ReportingProduct> {
		match self {
			LuaReportingProduct::BalancesAt(product) => Box::new(product),
			LuaReportingProduct::BalancesBetween(product) => Box::new(product),
			LuaReportingProduct::Transactions(product) => Box::new(product),
			LuaReportingProduct::DynamicReport(product) => Box::new(product),
		}
	}
}

/// Represents subset of [ReportingContext] which is passed to Lua\
#[derive(Deserialize, Serialize)]
struct LuaReportingContext {
	#[serde(with = "crate::serde::naivedate_to_js")]
	pub sofy_date: NaiveDate,
	#[serde(with = "crate::serde::naivedate_to_js")]
	pub eofy_date: NaiveDate,
	pub reporting_commodity: String,
	pub dps: u32,
}

impl LuaReportingContext {
	fn from(context: &ReportingContext) -> Self {
		Self {
			sofy_date: sofy_from_eofy(context.eofy_date),
			eofy_date: context.eofy_date,
			reporting_commodity: context.reporting_commodity.clone(),
			dps: context.db_connection.metadata().dps,
		}
	}
}

/// Generic reporting step which is implemented by a plugin
#[derive(Debug)]
pub struct PluginReportingStep {
	pub plugin_path: String,
	pub spec: ReportingStepSpec,
	pub args: ReportingStepArgs, // Currently only VoidArgs is supported
}

impl PluginReportingStep {
	fn takes_args(_name: &str, args: &ReportingStepArgs, _context: &ReportingContext) -> bool {
		*args == ReportingStepArgs::VoidArgs
	}

	fn from_args(
		name: &str,
		args: ReportingStepArgs,
		context: &ReportingContext,
	) -> Box<dyn ReportingStep> {
		// Look up plugin
		for (plugin_path, plugin_spec) in context.plugin_specs.iter() {
			if let Some(reporting_step_spec) =
				plugin_spec.reporting_steps.iter().find(|s| s.name == name)
			{
				return Box::new(Self {
					plugin_path: plugin_path.to_string(),
					spec: reporting_step_spec.clone(),
					args,
				});
			}
		}

		panic!("No plugin provides step {}", name);
	}
}

impl Display for PluginReportingStep {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{} {{PluginReportingStep}}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for PluginReportingStep {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: self.spec.name.clone(),
			product_kinds: self.spec.product_kinds.clone(),
			args: self.args.clone(),
		}
	}

	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		// Call to plugin
		let (lua, plugin) = load_plugin(&context.plugin_dir, &self.plugin_path);
		let plugin_step = plugin
			.reporting_steps
			.iter()
			.find(|s| s.spec == self.spec)
			.unwrap();

		let result_table = plugin_step
			.requires
			.call::<Table>((
				lua.to_value(&self.args).unwrap(),
				lua.to_value(&LuaReportingContext::from(context)).unwrap(),
			))
			.expect("Lua error");

		// Convert result to Rust
		let result = result_table
			.sequence_values()
			.map(|s| s.expect("Lua error"))
			.map(|v| lua.from_value(v).expect("Deserialise error"))
			.collect::<Vec<ReportingProductId>>();

		result
	}

	fn after_init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
		context: &ReportingContext,
	) {
		// Load plugin
		let (lua, plugin) = load_plugin(&context.plugin_dir, &self.plugin_path);
		let plugin_step = plugin
			.reporting_steps
			.iter()
			.find(|s| s.spec == self.spec)
			.unwrap();

		// Create a new scope since `add_dependency` depends on `dependencies`
		lua.scope(|scope| {
			// Init Lua environment
			let add_dependency = scope.create_function_mut(|_, (step, product)| {
				let step_id = lua.from_value::<ReportingStepId>(step)?;
				let product_id = lua.from_value::<ReportingProductId>(product)?;
				dependencies.add_dependency(step_id, product_id);
				Ok(())
			})?;

			// Call to plugin
			plugin_step.after_init_graph.call::<Value>((
				lua.to_value(&self.args).unwrap(),
				lua.to_value(&steps.iter().map(|s| s.id()).collect::<Vec<_>>())
					.unwrap(),
				add_dependency,
				lua.to_value(&LuaReportingContext::from(context)).unwrap(),
			))?;

			Ok(())
		})
		.expect("Lua error");
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		// Pre-compute some context for Lua
		let kinds_for_account =
			kinds_for_account(context.db_connection.get_account_configurations().await);

		let products = products.read().await;

		// Load plugin
		let (lua, plugin) = load_plugin(&context.plugin_dir, &self.plugin_path);
		let plugin_step = plugin
			.reporting_steps
			.iter()
			.find(|s| s.spec == self.spec)
			.unwrap();

		// Create a new scope since `get_product` depends on `products`
		let result_table = lua
			.scope(|scope| {
				// Init Lua environment
				let get_product = scope.create_function(|_, product| {
					let product_id = lua.from_value::<ReportingProductId>(product)?;
					let product = products.get_or_err(&product_id).unwrap();
					let product_enum: LuaReportingProduct = product.clone().into();
					Ok(lua.to_value(&product_enum))
				})?;

				// Call to plugin
				let result_table = plugin_step.execute.call::<Table>((
					lua.to_value(&self.args).unwrap(),
					lua.to_value(&LuaReportingContext::from(context)).unwrap(),
					lua.to_value(&kinds_for_account).unwrap(),
					get_product,
				))?;

				Ok(result_table)
			})
			.expect("Lua error");

		// Convert to Rust
		let mut products = ReportingProducts::new();
		for pair in result_table.pairs::<Value, Value>() {
			let pair = pair.expect("Lua error");
			let product_id = lua
				.from_value::<ReportingProductId>(pair.0)
				.expect("Deserialise error");
			let product = lua
				.from_value::<LuaReportingProduct>(pair.1)
				.expect("Deserialise error");

			products.insert(product_id, product.into());
		}

		Ok(products)
	}
}

/// Format the [Table] as a string
fn _dbg_table(table: &Table) -> String {
	format!(
		"{{{}}}",
		table
			.pairs::<Value, Value>()
			.map(|p| p.expect("Lua error"))
			.map(|(k, v)| format!(
				"{}: {}",
				if k.is_table() {
					_dbg_table(k.as_table().unwrap())
				} else {
					format!("{:?}", k)
				},
				if v.is_table() {
					_dbg_table(v.as_table().unwrap())
				} else {
					format!("{:?}", v)
				}
			))
			.collect::<Vec<_>>()
			.join(", ")
	)
}
