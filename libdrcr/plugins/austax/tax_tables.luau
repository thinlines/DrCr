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

-- Base income tax
-- https://www.ato.gov.au/rates/individual-income-tax-rates/
-- Maps each financial year to list of (upper limit (INclusive), flat amount, marginal rate)
local base_tax = {
	[2025] = {
		{18200, 0, 0},
		{45000, 0, 0.16},
		{135000, 4288, 0.30},
		{190000, 31288, 0.37},
		{math.huge, 51638, 0.45}
	},
	[2024] = {
		{18200, 0, 0},
		{45000, 0, 0.19},
		{120000, 5092, 0.325},
		{180000, 29467, 0.37},
		{math.huge, 51667, 0.45}
	},
	[2023] = {
		{18200, 0, 0},
		{45000, 0, 0.19},
		{120000, 5092, 0.325},
		{180000, 29467, 0.37},
		{math.huge, 51667, 0.45}
	}
}

local tax_tables = {
	base_tax = base_tax,
}

return tax_tables
