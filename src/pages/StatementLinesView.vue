<!--
	DrCr: Double-entry bookkeeping framework
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
-->

<template>
    <div class="flex flex-col h-full min-h-0">
    <h1 class="page-heading">
        Statement lines
    </h1>
    
    <div class="my-2 py-2 flex bg-white sticky top-0">
		<div class="grow flex gap-x-2 items-baseline">
			<button @click="reconcileAsTransfer" class="btn-secondary text-emerald-700 ring-emerald-600">
				Reconcile selected as transfer
			</button>
			<RouterLink :to="{ name: 'import-statement' }" class="btn-secondary">
				Import statement
			</RouterLink>
			<div class="flex items-baseline">
				<input id="only-unclassified" class="ml-3 mr-1 self-center checkbox-primary" type="checkbox" v-model="showOnlyUnclassified">
				<label for="only-unclassified" class="text-gray-900">Show only unclassified lines</label>
			</div>
		</div>
    </div>
    
    <div id="statement-line-list" class="flex-1 min-h-0 overflow-y-auto wk-aa relative">
        <table class="min-w-full sticky-table">
			<thead class="sticky-header">
				<tr class="">
					<th class="py-0.5 pr-1 align-bottom">
						<input id="statement-line-select-all" class="checkbox-primary" type="checkbox" @change="onToggleAll">
					</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold text-start">Source account</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold lg:w-[12ex] text-start">Date</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold text-start">Description</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold text-start">Charged to</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold lg:w-[12ex] text-end">Dr</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold lg:w-[12ex] text-end">Cr</th>
					<th class="py-0.5 pl-1 align-bottom text-gray-900 font-semibold text-end">Balance</th>
				</tr>
			</thead>
			<tbody @click="onClickTableElement">
				<tr>
					<td></td>
					<td class="py-0.5 px-1" colspan="7">Loading data…</td>
				</tr>
			</tbody>
		</table>
		
		<!-- Component for reconciling statement lines -->
		<div id="statement-line-classifier" class="hidden absolute">
			<div class="flex items-stretch">
				<ComboBoxAccounts v-model="classificationAccount" class="statement-line-classifier-input" />
				<button @click="onLineClassified" id="statement-line-classifier-button" type="button" class="relative -ml-px inline-flex items-center gap-x-1.5 px-3 py-1 text-gray-800 shadow-sm ring-1 ring-inset ring-gray-400 bg-white hover:bg-gray-50">
					<CheckIcon class="w-5 h-5" />
				</button>
			</div>
		</div>
    </div>
    </div>
</template>

<script setup lang="ts">
	import Clusterize from 'clusterize.js';
	
	import dayjs from 'dayjs';
	
	import { CheckIcon, PencilIcon } from '@heroicons/vue/24/outline';
	
	import { onUnmounted, ref, watch } from 'vue';
	
	import ComboBoxAccounts from '../components/ComboBoxAccounts.vue';
	import { db } from '../db.ts';
	import { renderComponent } from '../webutil.ts';
	import { ppWithCommodity } from '../display.ts';
	
	interface StatementLine {
		id: number,
		source_account: string,
		dt: string,
		description: string,
		quantity: number,
		balance: number | null,
		commodity: string,
		transaction_id: number,
		posting_accounts: string[]
	}
	
    const showOnlyUnclassified = ref(false);
    const statementLines = ref([] as StatementLine[]);
    const selectAll = ref(false);
	
	const classificationLineId = ref(0);
	const classificationAccount = ref('');
	
	let clusterize: Clusterize | null = null;
	
	async function load() {
		const session = await db.load();
		
		const joinedStatementLines: any[] = await session.select(
			`SELECT statement_lines.*, p2.transaction_id, p2.account AS posting_account
			FROM statement_lines
			LEFT JOIN statement_line_reconciliations ON statement_lines.id = statement_line_reconciliations.statement_line_id
			LEFT JOIN postings ON statement_line_reconciliations.posting_id = postings.id
			LEFT JOIN transactions ON postings.transaction_id = transactions.id
			LEFT JOIN postings p2 ON transactions.id = p2.transaction_id
			ORDER BY statement_lines.dt DESC, statement_lines.id DESC, p2.id`
		);
		
		// Unflatten statement lines
		const newStatementLines: StatementLine[] = [];
		
		for (const joinedStatementLine of joinedStatementLines) {
			if (newStatementLines.length === 0 || newStatementLines.at(-1)!.id !== joinedStatementLine.id) {
				newStatementLines.push({
					id: joinedStatementLine.id,
					source_account: joinedStatementLine.source_account,
					dt: joinedStatementLine.dt,
					description: joinedStatementLine.description,
					quantity: joinedStatementLine.quantity,
					balance: joinedStatementLine.balance,
					commodity: joinedStatementLine.commodity,
					transaction_id: joinedStatementLine.transaction_id,
					posting_accounts: []
				});
			}
			if (joinedStatementLine.posting_account !== null) {
				newStatementLines.at(-1)!.posting_accounts.push(joinedStatementLine.posting_account);
			}
		}
		
		statementLines.value = newStatementLines;
	}
	
    function onClickTableElement(event: MouseEvent) {
        // Use event delegation to avoid polluting global scope with the event listener
        if (event.target && (event.target as Element).classList.contains('statement-line-checkbox')) {
            const allBoxes = document.querySelectorAll('.statement-line-checkbox');
            const checkedBoxes = document.querySelectorAll('.statement-line-checkbox:checked');
            const header = document.getElementById('statement-line-select-all') as HTMLInputElement | null;
            if (header) {
                if (allBoxes.length === 0) {
                    header.checked = false;
                    header.indeterminate = false;
                } else if (checkedBoxes.length === 0) {
                    header.checked = false;
                    header.indeterminate = false;
                } else if (checkedBoxes.length === allBoxes.length) {
                    header.checked = true;
                    header.indeterminate = false;
                } else {
                    header.checked = false;
                    header.indeterminate = true;
                }
            }
        }
        if (event.target && (event.target as Element).classList.contains('classify-link')) {
            // ------------------------
            // Show classify line panel
			
			// Prevent selecting a different line when already classifying one line
			if ((document.getElementById('statement-line-classifier-button')! as HTMLButtonElement).disabled) {
				return;
			}
			
			// Set global state
			const td = (event.target as Element).closest('td')!;  // Reconciliation cell
			const tr = td.closest('tr')!;
			classificationLineId.value = parseInt(tr.dataset.lineId!);
			
			// Show all other reconciliation cells
			for (const el of document.querySelectorAll('#statement-line-list .charge-account > span')) {
				el.classList.remove('invisible');
			}
			
			// Hide contents of the cell
			const span = td.querySelector('span')!;  // Span wrapper for reconciliation cell content
			span.classList.add('invisible');
			
			// Position the classify line panel in place (relative to #statement-line-list)
			const outerDiv = document.getElementById('statement-line-list')!;
			const divReconciler = document.getElementById('statement-line-classifier')!;
			divReconciler.classList.remove('hidden');
			divReconciler.style.top = (outerDiv.scrollTop + td.getBoundingClientRect().y - outerDiv.getBoundingClientRect().y - 4) + 'px';
			divReconciler.style.left = (td.getBoundingClientRect().x - outerDiv.getBoundingClientRect().x) + 'px';
			
			// Focus classify line panel
			divReconciler.querySelector('input')!.focus();
		}
    }

    function onToggleAll(event: Event) {
        const el = event.target as HTMLInputElement;
        selectAll.value = !!el.checked;
        // When toggling explicitly, clear indeterminate
        el.indeterminate = false;
        // Re-render so all rows reflect selection
        renderTable();
    }

    async function onLineClassified(event: Event) {
        // Callback when clicking OK to classify a statement line
        if ((event.target! as any).disabled) {
            return;
        }
		
		const lineId = classificationLineId.value;
		const chargeAccount = classificationAccount.value;
		
		if (!chargeAccount) {
			return;
		}
		
		// Disable further submissions
		(document.querySelector('.statement-line-classifier-input')! as HTMLInputElement).disabled = true;
		(document.getElementById('statement-line-classifier-button')! as HTMLButtonElement).disabled = true;
		
		const statementLine = statementLines.value.find((l) => l.id === lineId)!;
		
		if (statementLine.posting_accounts.length !== 0) {
			await alert('Cannot reconcile already reconciled statement line');
			return;
		}
		
		// Check if account exists
		const session = await db.load();
		const countResult = await session.select('SELECT COUNT(*) FROM postings WHERE account = $1', [chargeAccount]) as any[];
		const doesAccountExist = countResult[0]['COUNT(*)'] > 0;
		if (!doesAccountExist) {
			// Prompt for confirmation
			if (!await confirm('Account "' + chargeAccount + '" does not exist. Continue to reconcile this transaction and create a new account?')) {
				(document.querySelector('.statement-line-classifier-input')! as HTMLInputElement).disabled = false;
				(document.getElementById('statement-line-classifier-button')! as HTMLButtonElement).disabled = false;
				return;
			}
		}
		
		// Insert transaction and statement line reconciliation atomically
		const dbTransaction = await session.begin();
		
		// Insert transaction
		const transactionResult = await dbTransaction.execute(
			`INSERT INTO transactions (dt, description)
			VALUES ($1, $2)`,
			[statementLine.dt, statementLine.description]
		);
		const transactionId = transactionResult.lastInsertId;
		
		// Insert posting for this account
		const accountPostingResult = await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity)
			VALUES ($1, NULL, $2, $3, $4)`,
			[transactionId, statementLine.source_account, statementLine.quantity, statementLine.commodity]
		);
		const accountPostingId = accountPostingResult.lastInsertId;
		
		// Insert posting for the charge account - no need to remember this ID
		await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity)
			VALUES ($1, NULL, $2, $3, $4)`,
			[transactionId, chargeAccount, -statementLine.quantity, statementLine.commodity]
		);
		
		// Insert statement line reconciliation
		await dbTransaction.execute(
			`INSERT INTO statement_line_reconciliations (statement_line_id, posting_id)
			VALUES ($1, $2)`,
			[statementLine.id, accountPostingId]
		);
		
		await dbTransaction.commit();
		
		// Reset statement line classifier state
		classificationAccount.value = '';
		(document.querySelector('.statement-line-classifier-input')! as HTMLInputElement).disabled = false;
		(document.getElementById('statement-line-classifier-button')! as HTMLButtonElement).disabled = false;
		
		// Hide the statement line classifier and unhide any hidden reconciliation cells
		document.getElementById('statement-line-classifier')!.classList.add('hidden');
		for (const el of document.querySelectorAll('#statement-line-list .charge-account > span')) {
			el.classList.remove('invisible');
		}
		
		// Reload transactions and re-render the table
		await load();
	}
	
	async function reconcileAsTransfer() {
		const selectedCheckboxes = document.querySelectorAll('.statement-line-checkbox:checked');
		
		if (selectedCheckboxes.length !== 2) {
			await alert('Must select exactly 2 statement lines');
			return;
		}
		
		const selectedLineIds = [...selectedCheckboxes].map((el) => parseInt(el.closest('tr')?.dataset.lineId!));
		
		const line1 = statementLines.value.find((l) => l.id === selectedLineIds[0])!;
		const line2 = statementLines.value.find((l) => l.id === selectedLineIds[1])!;
		
		// Sanity checks
		if (line1.quantity + line2.quantity !== 0 || line1.commodity !== line2.commodity) {
			await alert('Selected statement line debits/credits must equal');
			return;
		}
		if (line1.posting_accounts.length !== 0 || line2.posting_accounts.length !== 0) {
			await alert('Cannot reconcile already reconciled statement lines');
			return;
		}
		
		// Insert transaction and statement line reconciliation atomically
		const session = await db.load();
		const dbTransaction = await session.begin();
		
		// Insert transaction
		const transactionResult = await dbTransaction.execute(
			`INSERT INTO transactions (dt, description)
			VALUES ($1, $2)`,
			[line1.dt, line1.description]
		);
		const transactionId = transactionResult.lastInsertId;
		
		// Insert posting for line1
		const postingResult1 = await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity)
			VALUES ($1, $2, $3, $4, $5)`,
			[transactionId, line1.description, line1.source_account, line1.quantity, line1.commodity]
		);
		const postingId1 = postingResult1.lastInsertId;
		
		// Insert statement line reconciliation
		await dbTransaction.execute(
			`INSERT INTO statement_line_reconciliations (statement_line_id, posting_id)
			VALUES ($1, $2)`,
			[line1.id, postingId1]
		);
		
		// Insert posting for line2
		const postingResult2 = await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity)
			VALUES ($1, $2, $3, $4, $5)`,
			[transactionId, line2.description, line2.source_account, line2.quantity, line2.commodity]
		);
		const postingId2 = postingResult2.lastInsertId;
		
		// Insert statement line reconciliation
		await dbTransaction.execute(
			`INSERT INTO statement_line_reconciliations (statement_line_id, posting_id)
			VALUES ($1, $2)`,
			[line2.id, postingId2]
		);
		
		await dbTransaction.commit();
		
		// Reload transactions and re-render the table
		await load();
	}
	
	function renderTable() {
		const PencilIconHTML = renderComponent(PencilIcon, { 'class': 'w-4 h-4 inline align-middle -mt-0.5' });  // Pre-render the pencil icon
		const rows = [];
		
        for (const line of statementLines.value) {
            let reconciliationCell, checkboxCell;
            if (line.posting_accounts.length === 0) {
                // Unreconciled
                reconciliationCell =
                    `<a href="#" class="classify-link text-red-500 hover:text-red-600 hover:underline" onclick="return false;">Unclassified</a>`;
                checkboxCell = `<input class="checkbox-primary statement-line-checkbox" type="checkbox"${ selectAll.value ? ' checked' : '' }>`;  // Only show checkbox for unreconciled lines
            } else if (line.posting_accounts.length === 2) {
				// Simple reconciliation
				const otherAccount = line.posting_accounts.find((a) => a !== line.source_account);
				reconciliationCell =
					`<span>${ otherAccount }</span>
					<a href="/journal/edit/${ line.transaction_id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
				checkboxCell = '';
				
				if (showOnlyUnclassified.value) { continue; }
			} else {
				// Complex reconciliation
				reconciliationCell =
					`<i>(Complex)</i>
					<a href="/journal/edit/${ line.transaction_id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
				checkboxCell = '';
				
				if (showOnlyUnclassified.value) { continue; }
			}
			
			rows.push(
				`<tr data-line-id="${ line.id }">
					<td class="py-0.5 pr-1 align-baseline">${ checkboxCell }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900"><a href="/transactions/${ encodeURIComponent(line.source_account) }" class="hover:text-blue-700 hover:underline">${ line.source_account }</a></td>
					<td class="py-0.5 px-1 align-baseline text-gray-900 lg:w-[12ex]">${ dayjs(line.dt).format('YYYY-MM-DD') }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900">${ line.description }</td>
					<td class="charge-account py-0.5 px-1 align-baseline text-gray-900"><span>${ reconciliationCell }</span></td>
					<td class="py-0.5 px-1 align-baseline text-gray-900 lg:w-[12ex] text-end">${ line.quantity >= 0 ? ppWithCommodity(line.quantity, line.commodity) : '' }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900 lg:w-[12ex] text-end">${ line.quantity < 0 ? ppWithCommodity(-line.quantity, line.commodity) : '' }</td>
					<td class="py-0.5 pl-1 align-baseline text-gray-900 text-end">${ line.balance ?? '' }</td>
				</tr>`
			)
		}
		
		if (clusterize === null) {
			clusterize = new Clusterize({
				'rows': rows,
				scrollElem: document.getElementById('statement-line-list')!,
				contentElem: document.querySelector('#statement-line-list tbody')!,
				show_no_data_row: false
			});
        } else {
            clusterize.update(rows);
        }

        // Update header checkbox state based on current rows
        const header = document.getElementById('statement-line-select-all') as HTMLInputElement | null;
        if (header) {
            const allBoxes = document.querySelectorAll('.statement-line-checkbox');
            const checkedBoxes = document.querySelectorAll('.statement-line-checkbox:checked');
            if (allBoxes.length === 0) {
                header.checked = false;
                header.indeterminate = false;
            } else if (checkedBoxes.length === 0) {
                header.checked = false;
                header.indeterminate = false;
            } else if (checkedBoxes.length === allBoxes.length) {
                header.checked = true;
                header.indeterminate = false;
            } else {
                header.checked = false;
                header.indeterminate = true;
            }
        }

        // Hide the statement line classifier
        document.getElementById('statement-line-classifier')!.classList.add('hidden');
    }
	
	watch(showOnlyUnclassified, renderTable);
	watch(statementLines, renderTable);
	
	load();
	
	onUnmounted(() => {
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>

<style scoped>
.sticky-table {
  @apply border-separate;
  border-spacing: 0;
}
.sticky-header th {
  @apply sticky top-0 bg-white z-10 border-b border-gray-300;
}

#statement-line-classifier {
  @apply z-20;
}
</style>
