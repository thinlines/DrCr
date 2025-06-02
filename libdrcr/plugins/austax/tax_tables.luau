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

local tax_tables = {}

-- Base income tax
-- https://www.ato.gov.au/rates/individual-income-tax-rates/
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/itra1986174/sch7.html
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/itra1986174/s3.html (tax-free threshold)
-- Maps each financial year to list of (upper limit (INclusive), flat amount, marginal rate)
tax_tables.base_tax = {
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

-- FBT type 1 gross-up factor
-- https://www.ato.gov.au/rates/fbt/#GrossupratesforFBT
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/fbtaa1986312/s5b.html
tax_tables.fbt_grossup = 2.0802

-- Medicare levy thresholds
-- https://www.ato.gov.au/Individuals/Medicare-and-private-health-insurance/Medicare-levy/Medicare-levy-reduction/Medicare-levy-reduction-for-low-income-earners/
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/mla1986131/s3.html
-- Maps each financial year to list of (lower threshold, upper threshold)
tax_tables.medicare_levy_threshold = {
	[2025] = {27222, 34027},
	[2024] = {26000, 32500},
	[2023] = {24276, 30345},
	[2022] = {23365, 29207}
}

-- Medicare levy surcharge rates (singles)
-- https://www.ato.gov.au/individuals-and-families/medicare-and-private-health-insurance/medicare-levy-surcharge/medicare-levy-surcharge-income-thresholds-and-rates
-- https://www.austlii.edu.au/cgi-bin/viewdb/au/legis/cth/consol_act/mla1986131/s8b.html
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/phia2007248/s22.35.html
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/phia2007248/s22.45.html
-- Maps each financial year to list of (upper limit (INclusive), MLS rate)
-- FIXME: Only supports singles
tax_tables.medicare_levy_surcharge_single = {
	[2025] = {
		{97000, 0},
		{113000, 0.01},
		{151000, 0.0125},
		{math.huge, 0.015}
	},
	[2024] = {
		{93000, 0},
		{108000, 0.01},
		{144000, 0.0125},
		{math.huge, 0.015}
	}
}

-- Study and training loan (HELP, etc.) repayment thresholds and rates
-- https://www.ato.gov.au/Rates/HELP,-TSL-and-SFSS-repayment-thresholds-and-rates/
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/hesa2003271/s154.20.html
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/hesa2003271/s154.25.html
-- https://www.austlii.edu.au/cgi-bin/viewdoc/au/legis/cth/consol_act/hesa2003271/s154.30.html
-- Maps each financial year to list of (upper limit (EXclusive), repayment rate)
tax_tables.study_loan_repayment_rates = {
	[2025] = {
		{54435, 0},
		{62851, 0.01},
		{66621, 0.02},
		{70619, 0.025},
		{74856, 0.03},
		{79347, 0.035},
		{84108, 0.04},
		{89155, 0.045},
		{94504, 0.05},
		{100175, 0.055},
		{106186, 0.06},
		{112557, 0.065},
		{119310, 0.07},
		{126468, 0.075},
		{134057, 0.08},
		{142101, 0.085},
		{150627, 0.09},
		{159664, 0.095},
		{math.huge, 0.1}
	},
	[2024] = {
		{51550, 0},
		{59519, 0.01},
		{63090, 0.02},
		{66876, 0.025},
		{70889, 0.03},
		{75141, 0.035},
		{79650, 0.04},
		{84430, 0.045},
		{89495, 0.05},
		{94866, 0.055},
		{100558, 0.06},
		{106591, 0.065},
		{112986, 0.07},
		{119765, 0.075},
		{126951, 0.08},
		{134569, 0.085},
		{142643, 0.09},
		{151201, 0.095},
		{math.huge, 0.1}
	},
	[2023] = {
		{48361, 0},
		{55837, 0.01},
		{59187, 0.02},
		{62739, 0.025},
		{66503, 0.03},
		{70493, 0.035},
		{74723, 0.04},
		{79207, 0.045},
		{83959, 0.05},
		{88997, 0.055},
		{94337, 0.06},
		{99997, 0.065},
		{105997, 0.07},
		{112356, 0.075},
		{119098, 0.08},
		{126244, 0.085},
		{133819, 0.09},
		{141848, 0.095},
		{math.huge, 0.1}
	}
}

return tax_tables
