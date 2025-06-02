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

-- NB: Content of this file is also parsed by frontend src/plugins/austax/account_kinds.ts
-- Check the syntax is compatible with that parser!
local income_types = {
	{'income1', 'Salary or wages', '1'},
	{'income2', 'Allowances, earnings, tips, director\'s fees etc.', '2'},
	{'income3', 'Employer lump sum payments', '3'},
	{'income4', 'Employment termination payments', '4'},
	{'income5', 'Australian Government allowances and payments', '5'},
	{'income6', 'Australian Government pensions and allowances', '6'},
	{'income7', 'Australian annuities and superannuation income streams', '7'},
	{'income8', 'Australian superannuation lump sum payments', '8'},
	{'income9', 'Attributed personal services income', '9'},
	{'income10', 'Gross interest', '10'},
	{'income11', 'Dividends', '11'},
	{'income12', 'Employee share schemes', '12'},
	{'income13', 'Partnerships and trusts', '13'},
	{'income14', 'Personal services income', '14'},
	{'income15', 'Net income or loss from business', '15'},
	{'income16', 'Deferred non-commercial business losses', '16'},
	{'income17', 'Net farm management deposits or repayments', '17'},
	{'income18', 'Capital gains', '18'},
	{'income19', 'Foreign entities', '19'},
	{'income20', 'Foreign source income and foreign assets or property', '20'},
	{'income21', 'Rent', '21'},
	{'income22', 'Bonuses from life insurance companies and friendly societies', '22'},
	{'income23', 'Forestry managed investment scheme income', '23'},
	{'income24', 'Other income', '24'},
}

-- NB: Content of this file is also parsed by frontend src/plugins/austax/account_kinds.ts
-- Check the syntax is compatible with that parser!
local deduction_types = {
	{'d1', 'Work-related car expenses', 'D1'},
	{'d2', 'Work-related travel expenses', 'D2'},
	{'d3', 'Work-related clothing, laundry and dry cleaning expenses', 'D3'},
	{'d4', 'Work-related self-education expenses', 'D4'},
	{'d5', 'Other work-related expenses', 'D5'},
	{'d6', 'Low value pool deduction', 'D6'},
	{'d7', 'Interest deductions', 'D7'},
	{'d8', 'Dividend deductions', 'D8'},
	{'d9', 'Gifts or donations', 'D9'},
	{'d10', 'Cost of managing tax affairs', 'D10'},
	{'d11', 'Deductible amount of undeducted purchase price of a foreign pension or annuity', 'D11'},
	{'d12', 'Personal superannuation contributions', 'D12'},
	{'d13', 'Deduction for project pool', 'D13'},
	{'d14', 'Forestry managed investment scheme deduction', 'D14'},
	{'d15', 'Other deductions', 'D15'},
}

local account_kinds = {
	income_types = income_types,
	deduction_types = deduction_types,
}

return account_kinds
