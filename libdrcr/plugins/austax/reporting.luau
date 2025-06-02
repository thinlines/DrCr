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

-----------------
-- Flags

-- true = Spread income tax expense over monthly transactions
-- false = Charge income tax expense in one transaction at end of financial year
local charge_tax_monthly = true

-----------------
-- Reporting code

local libdrcr = require('../libdrcr')
local account_kinds = require('../austax/account_kinds')
local calc = require('../austax/calc')

-- Account constants
local CURRENT_YEAR_EARNINGS = 'Current Year Earnings'
local HELP = 'HELP'
local INCOME_TAX = 'Income Tax'
local INCOME_TAX_CONTROL = 'Income Tax Control'
local RETAINED_EARNINGS = 'Retained Earnings'

local reporting = {}

-- This ReportingStep calculates income tax
--
-- Generates the tax summary DynamicReport, and adds Transactions reconciling income tax expense, PAYG withholding and study loan repayments.
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
			bordered = false,
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
			bordered = false,
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
	
	-- Medicare levy row
	local tax_ml = calc.medicare_levy(net_taxable, context)
	if tax_ml ~= 0 then
		table.insert(report.entries, { Row = {
			text = 'Medicare levy',
			quantity = {tax_ml},
			id = 'tax_ml',
			visible = true,
			link = nil,
			heading = false,
			bordered = false,
		}})
	end
	
	-- Precompute RFB amount as this is required for MLS
	local rfb_taxable = 0
	for account, kinds in pairs(kinds_for_account) do
		if libdrcr.arr_contains(kinds, 'austax.rfb') then
			rfb_taxable -= balances[account] or 0  -- Invert as income = credit balances
		end
	end
	local rfb_grossedup = calc.rfb_grossup(rfb_taxable, context)
	
	-- Medicare levy surcharge row
	local tax_mls = calc.medicare_levy_surcharge(net_taxable, rfb_grossedup, context)
	if tax_mls ~= 0 then
		table.insert(report.entries, { Row = {
			text = 'Medicare levy surcharge',
			quantity = {tax_mls},
			id = 'tax_mls',
			visible = true,
			link = nil,
			heading = false,
			bordered = false,
		}})
	end
	
	-- Total income tax row
	local tax_total = tax_base + tax_ml + tax_mls
	table.insert(report.entries, { Row = {
		text = 'Total income tax',
		quantity = {tax_total},
		id = 'tax_total',
		visible = true,
		link = nil,
		heading = true,
		bordered = true,
	}})
	table.insert(report.entries, 'Spacer')
	
	-- Add tax offset entries
	local total_offset = 0
	
	do
		local entries = entries_for_kind('austax.offset', true, balances, kinds_for_account)
		if #entries ~= 0 then
			local section: libdrcr.Section = {
				text = 'Tax offsets',
				id = nil,
				visible = true,
				entries = entries,
			}
			table.insert(report.entries, { Section = section })
			total_offset += entries_subtotal(entries)
		end
	end
	
	-- Low income tax offset row
	local offset_lito = calc.lito(net_taxable, tax_total, context)
	if offset_lito ~= 0 then
		table.insert(report.entries, { Row = {
			text = 'Low income tax offset',
			quantity = {offset_lito},
			id = nil,
			visible = true,
			link = nil,
			heading = false,
			bordered = false,
		}})
		total_offset += offset_lito
	end
	
	-- Total tax offsets row
	if total_offset ~= 0 then
		table.insert(report.entries, { Row = {
			text = 'Total tax offsets',
			quantity = {total_offset},
			id = nil,
			visible = true,
			link = nil,
			heading = true,
			bordered = false,
		}})
		table.insert(report.entries, 'Spacer')
	end
	
	-- Calculate mandatory study loan repayment
	local study_loan_repayment = calc.study_loan_repayment(net_taxable, rfb_grossedup, context)
	
	-- Mandatory study loan repayment section
	if study_loan_repayment ~= 0 then
		-- Taxable value of reportable fringe benefits row
		if rfb_taxable ~= 0 then
			table.insert(report.entries, { Row = {
				text = 'Taxable value of reportable fringe benefits',
				quantity = {rfb_taxable},
				id = 'rfb_taxable',
				visible = true,
				link = nil,
				heading = false,
				bordered = false,
			}})
		end
		
		-- Grossed-up value row
		if rfb_grossedup ~= 0 then
			table.insert(report.entries, { Row = {
				text = 'Grossed-up value',
				quantity = {rfb_grossedup},
				id = 'rfb_grossedup',
				visible = true,
				link = nil,
				heading = false,
				bordered = false,
			}})
		end
		
		-- Mandatory study loan repayment row
		table.insert(report.entries, { Row = {
			text = 'Mandatory study loan repayment',
			quantity = {study_loan_repayment},
			id = 'study_loan_repayment',
			visible = true,
			link = nil,
			heading = true,
			bordered = false,
		}})
		table.insert(report.entries, 'Spacer')
	end
	
	-- Add PAYGW entries
	local total_paygw = 0
	
	do
		local entries = entries_for_kind('austax.paygw', false, balances, kinds_for_account)
		if #entries ~= 0 then
			local section: libdrcr.Section = {
				text = 'PAYG withheld amounts',
				id = nil,
				visible = true,
				entries = entries,
			}
			table.insert(report.entries, { Section = section })
			total_paygw = math.floor(entries_subtotal(entries) / 100) * 100
		end
	end
	
	-- Total PAYGW row
	if total_paygw ~= 0 then
		table.insert(report.entries, { Row = {
			text = 'Total withheld amounts',
			quantity = {total_paygw},
			id = 'total_paygw',
			visible = true,
			link = nil,
			heading = true,
			bordered = false,
		}})
		table.insert(report.entries, 'Spacer')
	end
	
	-- ATO liability row
	local ato_payable = tax_total - total_offset - total_paygw + study_loan_repayment
	table.insert(report.entries, { Row = {
		text = 'ATO liability payable (refundable)',
		quantity = {ato_payable},
		id = 'ato_payable',
		visible = true,
		link = nil,
		heading = true,
		bordered = true,
	}})
	
	-- Generate income tax transactions
	local transactions: {libdrcr.Transaction} = {}
	
	-- Estimated tax payable
	if charge_tax_monthly then
		-- Charge income tax expense in parts, one per month
		local monthly_tax = math.floor((tax_total - total_offset) / 12)
		local last_month_tax = (tax_total - total_offset) - 11 * monthly_tax  -- To account for rounding errors
		
		-- Some ad hoc calendar code
		local eofy_year, eofy_month, _ = libdrcr.parse_date(context.eofy_date)
		local last_day_of_month = { 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 }  -- Leap years handled below
		
		for month = 1, 12 do
			local this_year, this_month_tax
			if month == eofy_month then
				this_year = eofy_year
				this_month_tax = last_month_tax
			elseif month < eofy_month then
				this_year = eofy_year
				this_month_tax = monthly_tax
			else
				this_year = eofy_year - 1
				this_month_tax = monthly_tax
			end
			
			local this_day = last_day_of_month[month]
			
			-- Check for leap year
			if month == 2 and (this_year % 4 == 0) and (this_year % 100 ~= 0 or this_year % 400 == 0) then
				this_day = 29
			end
			
			-- Charge monthly tax
			table.insert(transactions, {
				id = nil,
				dt = libdrcr.date_to_dt(libdrcr.format_date(this_year, month, this_day)),
				description = 'Estimated income tax',
				postings = {
					{
						id = nil,
						transaction_id = nil,
						description = nil,
						account = INCOME_TAX,
						quantity = this_month_tax,
						commodity = context.reporting_commodity,
						quantity_ascost = this_month_tax,
					},
					{
						id = nil,
						transaction_id = nil,
						description = nil,
						account = INCOME_TAX_CONTROL,
						quantity = -this_month_tax,
						commodity = context.reporting_commodity,
						quantity_ascost = -this_month_tax,
					},
				},
			})
		end
	else
		-- Charge income tax expense in one transaction at EOFY
		table.insert(transactions, {
			id = nil,
			dt = libdrcr.date_to_dt(context.eofy_date),
			description = 'Estimated income tax',
			postings = {
				{
					id = nil,
					transaction_id = nil,
					description = nil,
					account = INCOME_TAX,
					quantity = (tax_total - total_offset),
					commodity = context.reporting_commodity,
					quantity_ascost = (tax_total - total_offset),
				},
				{
					id = nil,
					transaction_id = nil,
					description = nil,
					account = INCOME_TAX_CONTROL,
					quantity = -(tax_total - total_offset),
					commodity = context.reporting_commodity,
					quantity_ascost = -(tax_total - total_offset),
				},
			},
		})
	end
	
	-- Mandatory study loan repayment
	table.insert(transactions, {
		id = nil,
		dt = libdrcr.date_to_dt(context.eofy_date),
		description = 'Mandatory study loan repayment payable',
		postings = {
			{
				id = nil,
				transaction_id = nil,
				description = nil,
				account = HELP,
				quantity = study_loan_repayment,
				commodity = context.reporting_commodity,
				quantity_ascost = study_loan_repayment,
			},
			{
				id = nil,
				transaction_id = nil,
				description = nil,
				account = INCOME_TAX_CONTROL,
				quantity = -study_loan_repayment,
				commodity = context.reporting_commodity,
				quantity_ascost = -study_loan_repayment,
			},
		},
	})
	
	-- Transfer PAYGW balances to Income Tax Control
	for account, kinds in pairs(kinds_for_account) do
		if libdrcr.arr_contains(kinds, 'austax.paygw') then
			local balance = balances[account] or 0
			if balance ~= 0 then
				table.insert(transactions, {
					id = nil,
					dt = libdrcr.date_to_dt(context.eofy_date),
					description = 'PAYG withheld amounts',
					postings = {
						{
							id = nil,
							transaction_id = nil,
							description = nil,
							account = INCOME_TAX_CONTROL,
							quantity = balance,
							commodity = context.reporting_commodity,
							quantity_ascost = balance,
						},
						{
							id = nil,
							transaction_id = nil,
							description = nil,
							account = account,
							quantity = -balance,
							commodity = context.reporting_commodity,
							quantity_ascost = -balance,
						},
					},
				})
			end
		end
	end
	
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
		if account == CURRENT_YEAR_EARNINGS then
			link = '/income-statement'
		elseif account == RETAINED_EARNINGS then
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
