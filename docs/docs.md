# Contents

* [1. Introduction](#1-introduction)
* [2. Quick start](#2-quick-start)
* [3. Introduction to double-entry bookkeeping](#3-introduction-to-double-entry-bookkeeping)
* [4. Conceptual overview of DrCr](#4-conceptual-overview-of-drcr)
	* [Key concepts](#key-concepts)
	* [Workflow overview](#workflow-overview)

# 1. Introduction

DrCr (‘debit-credit’) is a self-contained open-source double-entry bookkeeping framework.

Conceptually, DrCr exists in a middle ground between lightweight ledger-keeping tools (e.g. [ledger](https://ledger-cli.org/), [hledger](https://hledger.org/), [beancount](https://beancount.github.io/)) and heavyweight accounting packages (e.g. [Xero](https://www.xero.com/), [MYOB](https://www.myob.com/)). Compared to the ledger-likes:

* DrCr is GUI driven, but data is stored in easily computer-readable format (SQL database)
* DrCr embraces accounting conventions (debits/credits, account types) – DrCr is not suitable for managing arbitrary non-financial double-entry-like data
* Applying accounting conventions to generate standard financial reports is core functionality in DrCr
* DrCr also preserves the conventional accounting relationship between source documents and ledger transactions – while the ledger is the core structure consolidating accounting information, ledger transactions can be generated programatically from source data of different forms

Compared to heavyweight accounting packages:

* DrCr's core function is to manage the double-entry ledger – management of other information is non-core, keeping functionality lightweight
	* For example, there is no invoicing functionality in DrCr – it is the user's responsibility to post appropriate transactions to the receivables ledger accounts
* DrCr provides tools with sharp edges – it is assumed the user is familiar with double-entry bookkeeping
* DrCr is tailored towards small-scale individual use for personal finance (but use for business accounting is entirely possible)

# 2. Quick start

If a binary build has been provided, simply double click the DrCr application. Otherwise, follow the build instructions in the README.

The welcome screen is displayed – ‘Welcome to DrCr. No file is currently open.’ The first step is to create a new DrCr file. Click *New file*. Click *OK* to accept the default settings. A file chooser dialog is displayed – choose where to save the new DrCr file.

The main menu is now displayed. Click *Trial balance*. The trial balance report is generated, showing there are no accounts, and the total debits and credits are accordingly both zero. To navigate back to the main menu:

* Click *DrCr* in the top left at the menu bar, or
* Right-click anywhere and select *Back* from the context menu, or
* If the window is sufficiently wide, click the back icon to the left of *DrCr* at the menu bar

We will now enter a new transaction. From the main menu, click *Journal* → *New transaction*. The transaction editor is displayed. Enter:

<table>
	<tr><th>Date</th><th colspan="2">Description</th><th>Dr</th><th>Cr</th></tr>
	<tr><td>Today's date</td><td colspan="2">Opening balance</td><td></td><td></td></tr>
	<tr><td></td><td>Dr</td><td>Cash at Hand</td><td>$100</td><td></td></tr>
	<tr><td></td><td>Cr</td><td>Opening Balances</td><td></td><td>$100</td></tr>
</table>

Click *Save*. The journal page has updated to show the new transaction.

From the main menu, click *Trial balance*. The trial balance now correctly shows a $100 debit balance in the *Cash at Hand* account, and a $100 credit balance in the *Opening Balances* account.

If we click *Balance sheet* from the main menu, the report will show zero balances for assets, liabilities and equity. This is because we have not configured *Cash at Hand* as an asset or *Opening Balances* as equity. We will do so now.

From the main menu, click *Chart of accounts*. The dropdown box at the top of the page is pre-populated with *Asset*. Select the checkbox next to *Cash at Hand*, and click *Add type*. The table updates to show that *Cash at Hand* is now configured as an asset account. Select the checkbox next to *Opening Balances*. Change the dropdown box from *Asset* to *Equity*, and click *Add type*. The table now shows:

&nbsp;|Account|Associated types
-|-|-
☐|Cash at Hand|• Asset
☐|Opening Balances|• Equity

Return to the *Balance sheet* report. The report is now correct:

&nbsp;|$
-|-:
**Assets**|
Cash at Hand|100.00
**Total assets**|**100.00**
&nbsp;|
**Liabilities**|
**Total liabilities**|**0.00**
&nbsp;|
**Equity**|
Opening Balances|100.00
**Total equity**|**100.00**

Now add another transaction from the journal page:

<table>
	<tr><th>Date</th><th colspan="2">Description</th><th>Dr</th><th>Cr</th></tr>
	<tr><td>Today's date</td><td colspan="2">Cash sales for the day</td><td></td><td></td></tr>
	<tr><td></td><td>Dr</td><td>Cash at Hand</td><td>$10</td><td></td></tr>
	<tr><td></td><td>Cr</td><td>Sales</td><td></td><td>$10</td></tr>
</table>

If we now click *Balance sheet* from the main menu, a warning is displayed ‘Total assets do not equal total liabilities and equity.’ As the warning goes on to note, this is because the *Sales* account has not been configured.

Configure the *Sales* account as an income account in the chart of accounts.

Now, from the main menu, click *Income statement*. The income statement report is displayed:

&nbsp;|$
-|-:
**Income**|
Sales|10.00
**Total income**|**10.00**
&nbsp;|
**Expenses**|
**Total expenses**|**0.00**
&nbsp;|
**Net surplus (deficit)**|**10.00**

From the main menu, click *Balance sheet*. The current year surplus is automatically displayed under equity:

&nbsp;|$
-|-:
**Assets**|
Cash at Hand|110.00
**Total assets**|**110.00**
&nbsp;|
**Liabilities**|
**Total liabilities**|**0.00**
&nbsp;|
**Equity**|
Current Year Earnings|10.00
Opening Balances|100.00
**Total equity**|**110.00**

We can add additional transactions and configure additional accounts in like manner. Note that, whenever possible, it is preferred to generate transactions via the statements feature, rather than create manual journal entries.

# 3. Introduction to double-entry bookkeeping

In this section, we briefly introduce the principles of double-entry bookkeeping, which underlies the conventional modern approach to accounting.

We first consider the balance sheet accounts – assets, liabilities and equity. Equity is defined as net assets, i.e. *Assets − Liabilities = Equity* at all times. For reasons which will become apparent, we will rearrange this so each side of the equation is expressed only in terms of addition, hence *Assets = Liabilities + Equity*. This is the *accounting equation*. From the equation, we can note that no account can change balance independently of the others. An increase in assets must be associated with either an increase in equity, or if equity remains constant, an increase in liabilities.

Equity can be further decomposed into total accumulated surpluses, being income minus expenses, plus all other equity accounts. Separating out the income and expense components in the equation, we can write *Assets = Liabilities + Equity + Income − Expenses*. Rearranging, we obtain *Assets + Expenses = Liabilities + Equity + Income*. This is the *expanded* accounting equation.

An increase to an account on the *left*-hand side of the equation (asset or expense) is a *debit*. An increase to an account on the *right*-hand side of the equation (liability, equity or income) is a *credit*. Similarly, a decrease to an asset or expense is a credit, and a decrease to a liability, equity or income is a debit.

The key principle behind double-entry bookkeeping is that any debit must be matched by a corresponding credit, and vice versa, in order to keep the accounting equation true. For example a debit to assets (increase in assets) must be associated with a credit to expenses, liabilities, equity or income (decrease in expenses, or increase in liabilities, equity or income).

<table><tr>
	<td>ⓘ</td>
	<td>
		<p><b>Comparison with signed-number bookkeeping</b></p>
		<p style="margin:0">Users of ledger-like plaintext accounting software may be familiar with ‘signed-number’ bookkeeping. Signed-number bookkeeping is exactly equivalent to conventional double-entry bookkeeping – a debit is equivalent to a positive-valued posting, and a credit is equivalent to a negative-valued posting.</p>
	</td>
</tr></table>

# 4. Conceptual overview of DrCr

## Key concepts

The core data structure in DrCr is the *ledger*, which is a collection of transactions. A *transaction* is a collection of postings, with an associated date and description. A *posting* represents a debit or credit to a single account.

An *account* is simply a name, associated with zero or more chart of account types. The basic account types are asset, liability, income, expense and equity. An account name is represented by a string of one or more Unicode characters. In some accounting software, the `:` character is used as a delimiter in account names (e.g. `Asset:Current:Cash:Cash at Bank`). This is permitted in DrCr, but the `:` character has no special meaning (i.e. DrCr has no concept of a tree of accounts and ‘sub-accounts’).

An *amount* represents a particular quantity of a commodity. Quantities are represented internally using fixed-point arithmetic – the number of decimal places precision can be configured at the time of database creation. The number of decimal places is the same for all commodities.

A *commodity* represents an amount of currency or other fungible asset. A commodity name is represented by a string of one or more Unicode characters except space. If the commodity name is a single character, it may precede the quantity in an amount (e.g. `$100`). Otherwise, it must follow the quantity separated by a space (e.g. `100 AUD`).

The *reporting commodity* is the default commodity, in terms of which all other commodities will be valued. Every commodity which is not the reporting commodity must be associated with a *cost base*, which values the commodity in terms of the reporting commodity. A cost base may be either specified as a unit price (specifying the cost of one unit of the commodity, in terms of the reporting commodity), or a total price (specifying the total cost of the entire quantity of the commodity, in terms of the reporting commodity). The cost base follows the quantity and commodity name separated by a space. A unit price is specified using single curly braces (e.g. `100 USD {1.50}`, each USD costs $1.50 in the reporting commodity). A total price is specified using double curly braces (e.g. `100 USD {{150}}`, 100 USD costs $150 in the reporting commodity).

<table><tr>
	<td>ⓘ</td>
	<td>All commodities, other than the reporting commodity, must specify a cost base.</td>
</tr></table>

## Workflow overview

The process of using DrCr can be conceptualised in 4 stages:

* **Source data** – The user inputs source data into DrCr
* **Transactions** – DrCr generates ledger transactions from the source data
* **Balances** – DrCr computes account balances from the ledger transactions
* **Reports** – DrCr produces balance sheet, income statement and other reports from account balances

Note that, conceptually, reports are generated based on account balances. In other words, it is conceived that reports will not be more granular than the account level. Therefore, there is limited functionality for filtering transactions at a level more granular than accounts.

<table><tr>
	<td>ⓘ</td>
	<td>The account is the most granular unit of reporting in DrCr.</td>
</tr></table>
