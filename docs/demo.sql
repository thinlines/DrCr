PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE account_configurations (
	id INTEGER NOT NULL,
	account VARCHAR,
	kind VARCHAR,
	data JSON,
	PRIMARY KEY(id)
);
INSERT INTO account_configurations VALUES(1,'Business Loan','drcr.liability',NULL);
INSERT INTO account_configurations VALUES(2,'Cash at Bank','drcr.asset',NULL);
INSERT INTO account_configurations VALUES(3,'Cash on Hand','drcr.asset',NULL);
INSERT INTO account_configurations VALUES(4,'Cost of Goods Sold','drcr.expense',NULL);
INSERT INTO account_configurations VALUES(5,'Depreciation','drcr.expense',NULL);
INSERT INTO account_configurations VALUES(6,'Forex Gains','drcr.income',NULL);
INSERT INTO account_configurations VALUES(7,'Interest','drcr.expense',NULL);
INSERT INTO account_configurations VALUES(8,'International Account','drcr.asset',NULL);
INSERT INTO account_configurations VALUES(9,'Inventory','drcr.asset',NULL);
INSERT INTO account_configurations VALUES(10,'Opening Balances','drcr.equity',NULL);
INSERT INTO account_configurations VALUES(11,'Plant','drcr.asset',NULL);
INSERT INTO account_configurations VALUES(12,'Plant:Less Accumulated Depreciation','drcr.asset',NULL);
INSERT INTO account_configurations VALUES(13,'Sales','drcr.income',NULL);
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
INSERT INTO metadata VALUES(1,'version','3');
INSERT INTO metadata VALUES(2,'eofy_date','2025-06-30');
INSERT INTO metadata VALUES(3,'reporting_commodity','$');
INSERT INTO metadata VALUES(4,'amount_dps','2');
INSERT INTO metadata VALUES(5,'plugins','');
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
INSERT INTO postings VALUES(1,1,NULL,'Cash at Bank',100000,'$');
INSERT INTO postings VALUES(2,1,NULL,'Opening Balances',-100000,'$');
INSERT INTO postings VALUES(3,2,NULL,'Cash on Hand',5000,'$');
INSERT INTO postings VALUES(4,2,NULL,'Opening Balances',-5000,'$');
INSERT INTO postings VALUES(5,3,NULL,'Inventory',10000,'Widgets {5.00}');
INSERT INTO postings VALUES(6,3,NULL,'Opening Balances',-10000,'Widgets {5.00}');
INSERT INTO postings VALUES(7,4,NULL,'Plant',500000,'$');
INSERT INTO postings VALUES(8,4,NULL,'Opening Balances',-500000,'$');
INSERT INTO postings VALUES(9,5,NULL,'Cash at Bank',50000,'$');
INSERT INTO postings VALUES(10,5,NULL,'Business Loan',-50000,'$');
INSERT INTO postings VALUES(11,6,NULL,'International Account',10000,'EUR {1.75}');
INSERT INTO postings VALUES(12,6,NULL,'Cash at Bank',-17500,'$');
INSERT INTO postings VALUES(13,7,NULL,'Inventory',5000,'Widgets {7.00}');
INSERT INTO postings VALUES(14,7,NULL,'Cash at Bank',-35000,'$');
INSERT INTO postings VALUES(15,8,NULL,'Cash at Bank',10000,'$');
INSERT INTO postings VALUES(16,8,NULL,'Sales',-10000,'$');
INSERT INTO postings VALUES(17,9,NULL,'Cost of Goods Sold',5000,'$');
INSERT INTO postings VALUES(18,9,NULL,'Inventory',-1000,'Widgets {5.00}');
INSERT INTO postings VALUES(19,10,NULL,'International Account',10000,'EUR {1.70}');
INSERT INTO postings VALUES(20,10,NULL,'Cash at Bank',-17000,'$');
INSERT INTO postings VALUES(21,11,NULL,'Cash at Bank',9000,'$');
INSERT INTO postings VALUES(22,11,NULL,'International Account',-5000,'EUR {1.75}');
INSERT INTO postings VALUES(23,11,NULL,'Forex Gains',-250,'$');
INSERT INTO postings VALUES(24,12,NULL,'Interest',10000,'$');
INSERT INTO postings VALUES(25,12,NULL,'Business Loan',-10000,'$');
INSERT INTO postings VALUES(26,13,NULL,'Depreciation',50000,'$');
INSERT INTO postings VALUES(27,13,NULL,'Plant:Less Accumulated Depreciation',-50000,'$');
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
	description VARCHAR,
	quantity INTEGER,
	balance INTEGER,
	commodity VARCHAR,
	PRIMARY KEY(id)
);
CREATE TABLE transactions (
	id INTEGER NOT NULL,
	dt DATETIME,
	description VARCHAR,
	PRIMARY KEY(id)
);
INSERT INTO transactions VALUES(1,'2024-06-30 00:00:00.000000','Conversion balances');
INSERT INTO transactions VALUES(2,'2024-06-30 00:00:00.000000','Conversion balances');
INSERT INTO transactions VALUES(3,'2024-06-30 00:00:00.000000','Conversion balances');
INSERT INTO transactions VALUES(4,'2024-06-30 00:00:00.000000','Opening balances');
INSERT INTO transactions VALUES(5,'2024-07-01 00:00:00.000000','Loan');
INSERT INTO transactions VALUES(6,'2024-07-02 00:00:00.000000','Application');
INSERT INTO transactions VALUES(7,'2024-07-03 00:00:00.000000','Inventory purchases');
INSERT INTO transactions VALUES(8,'2024-07-04 00:00:00.000000','Sale');
INSERT INTO transactions VALUES(9,'2024-07-04 00:00:00.000000','Sale');
INSERT INTO transactions VALUES(10,'2024-08-01 00:00:00.000000','Application');
INSERT INTO transactions VALUES(11,'2024-09-01 00:00:00.000000','Redemption');
INSERT INTO transactions VALUES(12,'2025-06-30 00:00:00.000000','Interest on business loan');
INSERT INTO transactions VALUES(13,'2025-06-30 00:00:00.000000','Depreciation');
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
CREATE VIEW joined_transactions AS
	SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity
	FROM transactions
	JOIN postings ON transactions.id = postings.transaction_id
	ORDER BY dt, transaction_id, postings.id;
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
	FROM joined_transactions
;
CREATE VIEW transactions_with_running_balances AS
	SELECT
		*,
		SUM(quantity_ascost) OVER (PARTITION BY account ROWS UNBOUNDED PRECEDING) AS running_balance
	FROM transactions_with_quantity_ascost;
COMMIT;
