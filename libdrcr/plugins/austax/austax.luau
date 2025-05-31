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

function requires(args, context)
	return {
		{
			name = 'CombineOrdinaryTransactions',
			kind = 'BalancesBetween',
			args = { DateStartDateEndArgs = { date_start = context.sofy_date, date_end = context.eofy_date } },
		}
	}
end

function after_init_graph(args, steps, add_dependency, context)
	for i = 1, #steps do
		local other = steps[i]
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

function execute(args, context, get_product)
	print('Stub: CombineOrdinaryTransactions.execute')
	
	local product = get_product({
		name = 'CombineOrdinaryTransactions',
		kind = 'BalancesBetween',
		args = { DateStartDateEndArgs = { date_start = context.sofy_date, date_end = context.eofy_date } }
	})
	
	print(libdrcr.repr(product))
	
	return {
		[{ name = 'CalculateIncomeTax', kind = 'Transactions', args = 'VoidArgs' }] = {
			Transactions = {
				transactions = {}
			}
		}
	}
end

local plugin: libdrcr.Plugin = {
	name = 'austax',
	reporting_steps = {
		{
			name = 'CalculateIncomeTax',
			product_kinds = {'DynamicReport', 'Transactions'},
			requires = requires,
			after_init_graph = after_init_graph,
			execute = execute,
		}
	},
}

return plugin
