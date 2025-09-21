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

-- Current version: 6 (see db.ts)

---------
-- Tables

CREATE TABLE account_configurations (
	id INTEGER NOT NULL,
	account VARCHAR,
	kind VARCHAR,
	data JSON,
	PRIMARY KEY(id)
);

CREATE TABLE balance_assertions (
	id INTEGER NOT NULL,
	dt DATETIME,
	description VARCHAR,
	account VARCHAR,
	quantity INTEGER,
	commodity VARCHAR,
	PRIMARY KEY(id)
);

CREATE TABLE metadata (
	id INTEGER NOT NULL,
	key VARCHAR,
	value VARCHAR,
	PRIMARY KEY(id)
);

CREATE TABLE postings (
	id INTEGER NOT NULL,
	transaction_id INTEGER,
	description VARCHAR,
	account VARCHAR,
	quantity INTEGER,
	commodity VARCHAR,
	PRIMARY KEY(id),
	FOREIGN KEY(transaction_id) REFERENCES transactions(id)
);

CREATE TABLE statement_line_reconciliations (
	id INTEGER NOT NULL,
	statement_line_id INTEGER,
	posting_id INTEGER,
	PRIMARY KEY(id),
	FOREIGN KEY(statement_line_id) REFERENCES statement_lines(id),
	FOREIGN KEY(posting_id) REFERENCES postings(id)
);

CREATE TABLE statement_lines (
	id INTEGER NOT NULL,
	source_account VARCHAR,
	dt DATETIME,
	name VARCHAR,
	memo VARCHAR,
	description VARCHAR,
	quantity INTEGER,
	balance INTEGER,
	commodity VARCHAR,
	fitid VARCHAR,
	dedup_ignore INTEGER DEFAULT 0,
	PRIMARY KEY(id)
);

CREATE TABLE transactions (
	id INTEGER NOT NULL,
	dt DATETIME,
	description VARCHAR,
	PRIMARY KEY(id)
);

---------
-- austax

CREATE TABLE austax_cgt_cost_adjustments (
	id INTEGER NOT NULL,
	quantity INTEGER,
	commodity VARCHAR,
	account VARCHAR,
	acquisition_dt DATETIME,
	dt DATETIME,
	description VARCHAR,
	cost_adjustment INTEGER,
	PRIMARY KEY (id)
);

--------
-- Views

-- Join transactions and postings
CREATE VIEW joined_transactions AS
	SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity
	FROM transactions
	JOIN postings ON transactions.id = postings.transaction_id
	ORDER BY dt, transaction_id, postings.id;

-- Convert amounts into cost basis in reporting commodity
CREATE VIEW transactions_with_quantity_ascost AS
	SELECT
		*,
		CAST(ROUND(
			-- If already in reporting commodity
			IIF(
				commodity = '$',
				quantity,
				-- Else if specified as total cost
				IIF(
					commodity LIKE '% {{%}}',
					substr(commodity, instr(commodity, ' {{') + 3, length(commodity) - instr(commodity, ' {{') - 4) * sign(quantity) * 100,
					-- Else if specified as unit cost
					IIF(
						commodity LIKE '% {%}',
						substr(commodity, instr(commodity, ' {') + 2, length(commodity) - instr(commodity, ' {') - 2) * quantity,
						-- Unexpected
						NULL
					)
				)
			)
		) AS INTEGER) AS quantity_ascost
	FROM joined_transactions;

-- Sum running balances
CREATE VIEW transactions_with_running_balances AS
	SELECT
		*,
		SUM(quantity_ascost) OVER (PARTITION BY account ROWS UNBOUNDED PRECEDING) AS running_balance
	FROM transactions_with_quantity_ascost;
