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
local tax_tables = require('../austax/tax_tables')

local calc = {}

-- Get the amount of base income tax
function calc.base_income_tax(net_taxable: number, context: libdrcr.ReportingContext): number
	local year, _, _ = libdrcr.parse_date(context.eofy_date)
	local base_tax_table = tax_tables.base_tax[year]
	
	for i, row in ipairs(base_tax_table) do
		local upper_limit = row[1] * (10 ^ context.dps)
		local flat_amount = row[2] * (10 ^ context.dps)
		local marginal_rate = row[3]
		
		-- Lower limit is the upper limit of the preceding bracket
		local lower_limit = 0
		if i > 1 then
			lower_limit = base_tax_table[i - 1][1] * (10 ^ context.dps)
		end
		
		if net_taxable <= upper_limit then
			return flat_amount + math.floor(marginal_rate * (net_taxable - lower_limit))
		end
	end
	
	error('Taxable income not within any tax bracket')
end

-- Get the amount of low income tax offset
-- https://www.ato.gov.au/forms-and-instructions/low-and-middle-income-earner-tax-offsets
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/itaa1997240/s61.115.html
function calc.lito(net_taxable: number, tax_total: number, context: libdrcr.ReportingContext): number
	if net_taxable <= 37500 * (10 ^ context.dps) then
		-- LITO is non-refundable
		-- FIXME: This will not work if we implement multiple non-refundable tax offsets
		if tax_total <= 700 * (10 ^ context.dps) then
			return tax_total
		else
			return 700 * (10 ^ context.dps)
		end
	elseif net_taxable <= 45000 * (10 ^ context.dps) then
		return 700 * (10 ^ context.dps) - math.floor(0.05 * (net_taxable - 37500 * (10 ^ context.dps)))
	elseif net_taxable <= 66667 * (10 ^ context.dps) then
		return 325 * (10 ^ context.dps) - math.floor(0.015 * (net_taxable - 45000 * (10 ^ context.dps)))
	else
		return 0
	end
end

-- Get the amount of Medicare levy
function calc.medicare_levy(net_taxable: number, context: libdrcr.ReportingContext): number
	local year, _, _ = libdrcr.parse_date(context.eofy_date)
	local threshold_table = tax_tables.medicare_levy_threshold[year]
	local lower_threshold = threshold_table[1] * (10 ^ context.dps)
	local upper_threshold = threshold_table[2] * (10 ^ context.dps)
	
	if net_taxable < lower_threshold then
		return 0
	elseif net_taxable < upper_threshold then
		-- Medicare levy is 10% of the amount above the lower threshold
		return math.floor((net_taxable - lower_threshold) * 0.1)
	else
		-- Normal Medicare levy
		return math.floor(net_taxable * 0.02)
	end
end

-- Get the amount of Medicare levy surcharge
function calc.medicare_levy_surcharge(net_taxable: number, rfb_grossedup: number, context: libdrcr.ReportingContext): number
	local mls_income = net_taxable + rfb_grossedup
	
	local year, _, _ = libdrcr.parse_date(context.eofy_date)
	local mls_table = tax_tables.medicare_levy_surcharge_single[year]
	
	for _, row in ipairs(mls_table) do
		local upper_limit = row[1] * (10 ^ context.dps)
		local rate = row[2]
		
		if mls_income <= upper_limit then
			return math.floor(rate * mls_income)
		end
	end
	
	error('MLS income not within any MLS bracket')
end

-- Calculate the grossed-up reportable fringe benefit
function calc.rfb_grossup(rfb_taxable: number, context: libdrcr.ReportingContext): number
	return math.floor(rfb_taxable * tax_tables.fbt_grossup)
end

-- Get the amount of mandatory study loan repayment
function calc.study_loan_repayment(net_taxable: number, rfb_grossedup: number, context: libdrcr.ReportingContext): number
	local repayment_income = net_taxable + rfb_grossedup
	
	local year, _, _ = libdrcr.parse_date(context.eofy_date)
	local repayment_table = tax_tables.study_loan_repayment_rates[year]
	
	for _, row in ipairs(repayment_table) do
		local upper_limit = row[1] * (10 ^ context.dps)
		local rate = row[2]
		
		if repayment_income < upper_limit then
			return math.floor(rate * repayment_income)
		end
	end
	
	error('HELP repayment income not within any repayment bracket')
end

return calc
