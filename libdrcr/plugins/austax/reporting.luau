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

local libdrcr = require('../libdrcr')
local account_kinds = require('../austax/account_kinds')
local calc = require('../austax/calc')

local reporting = {}

reporting.CalculateIncomeTax = {
	name = 'CalculateIncomeTax',
	product_kinds = {'DynamicReport', 'Transactions'},
} :: libdrcr.ReportingStep

function reporting.CalculateIncomeTax.requires(args, context)
	return {
		{
			name = 'CombineOrdinaryTransactions',
			kind = 'BalancesBetween',
			args = { DateStartDateEndArgs = { date_start = context.sofy_date, date_end = context.eofy_date } },
		}
	}
end

function reporting.CalculateIncomeTax.after_init_graph(args, steps, add_dependency, context)
	for _, other in ipairs(steps) do
		if other.name == 'AllTransactionsExceptEarningsToEquity' then
			-- AllTransactionsExceptEarningsToEquity depends on CalculateIncomeTax
			-- TODO: Only in applicable years
			
			local other_args: libdrcr.ReportingStepArgs
			if other.product_kinds[1] == 'Transactions' then
				other_args = 'VoidArgs'
			else
				other_args = other.args
			end
			
			add_dependency(other, {
				name = 'CalculateIncomeTax',
				kind = other.product_kinds[1],
				args = other_args,
			})
		end
	end
end

function reporting.CalculateIncomeTax.execute(args, context, kinds_for_account, get_product)
	-- Get balances for current year
	local product = get_product({
		name = 'CombineOrdinaryTransactions',
		kind = 'BalancesBetween',
		args = { DateStartDateEndArgs = { date_start = context.sofy_date, date_end = context.eofy_date } }
	})
	assert(product.BalancesBetween ~= nil)
	local balances = product.BalancesBetween.balances
	
	-- Generate tax summary report
	local report: libdrcr.DynamicReport = {
		title = 'Tax summary',
		columns = {'$'},
		entries = {},
	}
	
	-- Add income entries
	local total_income = 0
	
	for _, income_type in ipairs(account_kinds.income_types) do
		local code, label, number = unpack(income_type)
		
		local entries
		if code == 'income1' then
			-- Special case for salary or wages - round each separately
			entries = entries_for_kind_floor('austax.' .. code, true, balances, kinds_for_account, 100)
		else
			entries = entries_for_kind('austax.' .. code, true, balances, kinds_for_account)
		end
		
		if #entries == 0 then
			continue
		end
		
		local section: libdrcr.Section = {
			text = label .. ' (' .. number .. ')',
			id = nil,
			visible = true,
			entries = entries,
		}
		
		-- Add subtotal row
		local subtotal = math.floor(entries_subtotal(entries) / 100) * 100
		total_income += subtotal
		
		table.insert(section.entries, { Row = {
			text = 'Total item ' .. number,
			quantity = {subtotal},
			id = 'total_' .. code,
			visible = true,
			link = nil,
			heading = true,
			bordered = true,
		}})
		table.insert(report.entries, { Section = section })
		table.insert(report.entries, 'Spacer')
	end
	
	-- Total assessable income
	table.insert(report.entries, { Row = {
		text = 'Total assessable income',
		quantity = {total_income},
		id = 'total_income',
		visible = true,
		link = nil,
		heading = true,
		bordered = true,
	}})
	table.insert(report.entries, 'Spacer')
	
	-- Add deduction entries
	local total_deductions = 0
	
	for _, deduction_type in ipairs(account_kinds.deduction_types) do
		local code, label, number = unpack(deduction_type)
		
		local entries = entries_for_kind('austax.' .. code, false, balances, kinds_for_account)
		
		if #entries == 0 then
			continue
		end
		
		local section: libdrcr.Section = {
			text = label .. ' (' .. number .. ')',
			id = nil,
			visible = true,
			entries = entries,
		}
		
		-- Add subtotal row
		local subtotal = math.floor(entries_subtotal(entries) / 100) * 100
		total_deductions += subtotal
		
		table.insert(section.entries, { Row = {
			text = 'Total item ' .. number,
			quantity = {subtotal},
			id = 'total_' .. code,
			visible = true,
			link = nil,
			heading = true,
			bordered = true,
		}})
		table.insert(report.entries, { Section = section })
		table.insert(report.entries, 'Spacer')
	end
	
	-- Total deductions
	table.insert(report.entries, { Row = {
		text = 'Total deductions',
		quantity = {total_deductions},
		id = 'total_deductions',
		visible = true,
		link = nil,
		heading = true,
		bordered = true,
	}})
	table.insert(report.entries, 'Spacer')
	
	-- Net taxable income
	local net_taxable = total_income - total_deductions
	table.insert(report.entries, { Row = {
		text = 'Net taxable income',
		quantity = {net_taxable},
		id = 'net_taxable',
		visible = true,
		link = nil,
		heading = true,
		bordered = true,
	}})
	table.insert(report.entries, 'Spacer')
	
	-- Base income tax row
	local tax_base = calc.base_income_tax(net_taxable, context)
	table.insert(report.entries, { Row = {
		text = 'Base income tax',
		quantity = {tax_base},
		id = 'tax_base',
		visible = true,
		link = nil,
		heading = false,
		bordered = false,
	}})
	
	-- Total income tax row
	local tax_total = tax_base
	table.insert(report.entries, { Row = {
		text = 'Total income tax',
		quantity = {tax_total},
		id = 'tax_total',
		visible = true,
		link = nil,
		heading = true,
		bordered = true,
	}})
	
	-- Generate income tax transaction
	local transactions: {libdrcr.Transaction} = {
		{
			id = nil,
			dt = libdrcr.date_to_dt(context.eofy_date),
			description = 'Estimated income tax',
			postings = {
				{
					id = nil,
					transaction_id = nil,
					description = nil,
					account = 'Income Tax',
					quantity = tax_total,
					commodity = context.reporting_commodity,
					quantity_ascost = tax_total,
				},
				{
					id = nil,
					transaction_id = nil,
					description = nil,
					account = 'Income Tax Control',
					quantity = -tax_total,
					commodity = context.reporting_commodity,
					quantity_ascost = -tax_total,
				}
			},
		}
	}
	
	return {
		[{ name = 'CalculateIncomeTax', kind = 'Transactions', args = 'VoidArgs' }] = {
			Transactions = {
				transactions = transactions
			}
		},
		[{ name = 'CalculateIncomeTax', kind = 'DynamicReport', args = 'VoidArgs' }] = {
			DynamicReport = report
		},
	}
end

function entries_for_kind(kind: string, invert: boolean, balances:{ [string]: number }, kinds_for_account:{ [string]: {string} }): {libdrcr.DynamicReportEntry}
	-- Get accounts of specified kind
	local accounts = {}
	for account, kinds in pairs(kinds_for_account) do
		if libdrcr.arr_contains(kinds, kind) then
			table.insert(accounts, account)
		end
	end
	table.sort(accounts)
	
	local entries = {}
	for _, account in ipairs(accounts) do
		local quantity = balances[account] or 0
		if invert then
			quantity = -quantity
		end
		
		-- Do not show if all quantities are zero
		if quantity == 0 then
			continue
		end
		
		-- Some exceptions for the link
		local link: string | nil
		if account == 'Current Year Earnings' then
			link = '/income-statement'
		elseif account == 'Retained Earnings' then
			link = nil
		else
			link = '/transactions/' .. account
		end
		
		local row: libdrcr.Row = {
			text = account,
			quantity = {quantity},
			id = nil,
			visible = true,
			link = link,
			heading = false,
			bordered = false,
		}
		table.insert(entries, { Row = row })
	end
	
	return entries
end

-- Call `entries_for_kind` then round results down to next multiple of `floor`
function entries_for_kind_floor(kind: string, invert: boolean, balances:{ [string]: number }, kinds_for_account:{ [string]: {string} }, floor: number): {libdrcr.DynamicReportEntry}
	local entries = entries_for_kind(kind, invert, balances, kinds_for_account)
	for _, entry in ipairs(entries) do
		local row = (entry :: { Row: libdrcr.Row }).Row
		row.quantity[1] = math.floor(row.quantity[1] / floor) * floor
	end
	return entries
end

function entries_subtotal(entries: {libdrcr.DynamicReportEntry}): number
	local subtotal = 0
	for _, entry in ipairs(entries) do
		local row = (entry :: { Row: libdrcr.Row }).Row
		subtotal += row.quantity[1]
	end
	return subtotal
end

return reporting
