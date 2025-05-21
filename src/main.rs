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

use chrono::NaiveDate;
use libdrcr::reporting::{
	builders::register_dynamic_builders,
	calculator::solve_for,
	steps::{register_lookup_fns, AllTransactionsExceptRetainedEarnings, CalculateIncomeTax},
	DateEofyArgs, DateStartDateEndArgs, ReportingContext, ReportingStep,
};

fn main() {
	let mut context = ReportingContext::new(NaiveDate::from_ymd_opt(2025, 6, 30).unwrap());
	register_lookup_fns(&mut context);
	register_dynamic_builders(&mut context);

	let targets: Vec<Box<dyn ReportingStep>> = vec![
		Box::new(CalculateIncomeTax {
			args: DateEofyArgs {
				date_eofy: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
			},
		}),
		Box::new(AllTransactionsExceptRetainedEarnings {
			args: DateStartDateEndArgs {
				date_start: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
				date_end: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
			},
		}),
	];

	match solve_for(targets, context) {
		Ok(steps) => {
			for step in steps {
				println!("- {}", step);
			}
		}
		Err(err) => panic!("Error: {:?}", err),
	}
}
