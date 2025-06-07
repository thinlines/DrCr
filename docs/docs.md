# Contents

* [1. Introduction](#1-introduction)
* [2. Quick start](#2-quick-start)

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
