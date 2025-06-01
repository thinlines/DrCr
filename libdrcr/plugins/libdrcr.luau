--!strict
--  DrCr: Web-based double-entry bookkeeping framework
--  Copyright (C) 2022-2025  Lee Yingtong Li (RunasSudo)
--
--  This program is free software: you can redistribute it and/or modify
--  it under the terms of the GNU Affero General Public License as published by
--  the Free Software Foundation, either version 3 of the License, or
--  (at your option) any later version.
--
--  This program is distributed in the hope that it will be useful,
--  but WITHOUT ANY WARRANTY; without even the implied warranty of
--  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
--  GNU Affero General Public License for more details.
--
--  You should have received a copy of the GNU Affero General Public License
--  along with this program.  If not, see <https://www.gnu.org/licenses/>.

------------------------
-- Plugin specific types

-- Represents a libdrcr plugin specification and implementation
export type Plugin = {
	name: string,
	reporting_steps: {ReportingStep},
}

-- Specifies a ReportingStep provided by the plugin
export type ReportingStep = {
	name: string,
	product_kinds: {ReportingProductKind},
	
	requires: (
		ReportingStepArgs,
		ReportingContext
	) -> {ReportingProductId},
	
	after_init_graph: (
		ReportingStepArgs,
		{ReportingStepId},  -- steps
		(ReportingStepId, ReportingProductId) -> (),  -- add_dependency
		ReportingContext
	) -> (),
	
	execute: (
		ReportingStepArgs,
		ReportingContext,
		{[string]: {string}},  -- kinds_for_account
		(ReportingProductId) -> ReportingProduct  -- get_product
	) -> {[ReportingProductId]: ReportingProduct},
}

------------------
-- Dynamic reports

export type DynamicReport = {
	title: string,
	columns: {string},
	entries: {DynamicReportEntry},
}

export type DynamicReportEntry = 'Spacer' | { Section: Section } | { Row: Row }

export type Section = {
	text: string | nil,
	id: string | nil,
	visible: boolean,
	entries: {DynamicReportEntry},
}

export type Row = {
	text: string,
	quantity: {number},
	id: string | nil,
	visible: boolean,
	link: string | nil,
	heading: boolean,
	bordered: boolean,
}

-------------------------
-- libdrcr internal types

export type ReportingContext = {
	sofy_date: string,
	eofy_date: string,
	reporting_commodity: string,
	dps: number,
}

-- Accounting types

export type Transaction = {
	id: number | nil,
	dt: string,
	description: string,
	postings: {Posting},
}

export type Posting = {
	id: number | nil,
	transaction_id: number | nil,
	description: number | nil,
	account: string,
	quantity: number,
	commodity: string,
	quantity_ascost: number | nil,
}

-- Reporting products

export type ReportingProduct = {
	BalancesAt: BalancesAt?,
	BalancesBetween: BalancesBetween?,
	DynamicReport: DynamicReport?,
	Transactions: Transactions?,
}

export type BalancesAt = any
export type BalancesBetween = any
export type Transactions = { transactions: {Transaction} }

export type ReportingProductId = {
	name: string,
	kind: ReportingProductKind,
	args: ReportingStepArgs,
}

export type ReportingProductKind = 'BalancesAt' | 'BalancesBetween' | 'DynamicReport' | 'Transactions'

-- Reporting steps

export type ReportingStepId = {
	name: string,
	product_kinds: {ReportingProductKind},
	args: ReportingStepArgs,
}

-- Reporting step args

export type ReportingStepArgs = 'VoidArgs' | { DateArgs: DateArgs } | { DateStartDateEndArgs: DateStartDateEndArgs } | { MultipleDateArgs: MultipleDateArgs } | { MultipleDateStartDateEndArgs: MultipleDateStartDateEndArgs }

export type DateArgs = { date: string }
export type DateStartDateEndArgs = { date_start: string, date_end: string }
export type MultipleDateArgs = { dates: {DateArgs} }
export type MultipleDateStartDateEndArgs = { dates: {DateStartDateEndArgs} }

-----------------
-- Module exports

local libdrcr = {}

function libdrcr.arr_contains(haystack: {any}, needle: any): boolean
	for _, element in ipairs(haystack) do
		if element == needle then
			return true
		end
	end
	return false
end

function libdrcr.date_to_dt(date: string): string
	return date .. ' 00:00:00.000000'
end

function libdrcr.parse_date(date: string): (number, number, number)
	local year_str, month_str, day_str = string.match(date, '(%d%d%d%d)-(%d%d)-(%d%d)')
	
	local year = tonumber(year_str)
	local month = tonumber(month_str)
	local day = tonumber(day_str)
	
	assert(year ~= nil)
	assert(month ~= nil)
	assert(day ~= nil)
	
	return year, month, day
end

function libdrcr.repr(value: any): string
	local result = ''
	if type(value) == 'table' then
		result = result .. '{'
		for k, v in pairs(value) do
			result = result .. k .. ' = ' .. libdrcr.repr(v) .. ', '
		end
		result = result .. '}'
	elseif type(value) == 'string' then
		result = result .. "'" .. value .. "'"
	elseif type(value) == 'number' then
		result = result .. value
	else
		result = result .. '??'
	end
	return result
end

return libdrcr
