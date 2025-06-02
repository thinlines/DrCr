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
			return flat_amount + marginal_rate * (net_taxable - lower_limit)
		end
	end
	
	error('Taxable income not within any tax bracket')
end

return calc
