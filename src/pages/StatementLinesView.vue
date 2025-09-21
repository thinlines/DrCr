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
			<!-- Batch classify selected lines -->
			<button v-if="selectedCount >= 2" @click="openBatchClassifier" class="btn-secondary">
				Charge to…
			</button>
			<button @click="reconcileAsTransfer" class="btn-secondary text-emerald-700 ring-emerald-600">
				Reconcile selected as transfer
			</button>
			<RouterLink :to="{ name: 'import-statement' }" class="btn-secondary">
				Import statement
			</RouterLink>
			<div class="flex items-baseline">
				<div class="relative">
					<input type="text" class="bordered-field pr-8" v-model="searchQuery" placeholder="Search description…">
					<button v-if="searchQuery" type="button" @click="searchQuery = ''" class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700" title="Clear search">
						<XMarkIcon class="w-4 h-4" />
					</button>
				</div>
			</div>
			<div class="flex items-baseline">
				<input id="only-unclassified" class="ml-3 mr-1 self-center checkbox-primary" type="checkbox" v-model="showOnlyUnclassified">
				<label for="only-unclassified" class="text-gray-900">Show only unclassified lines</label>
			</div>
			<div class="flex items-baseline">
				<input id="only-duplicates" class="ml-3 mr-1 self-center checkbox-primary" type="checkbox" v-model="showOnlyDuplicates">
				<label for="only-duplicates" class="text-gray-900">Show only duplicates</label>
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
				<button @click="closeClassifier" id="statement-line-classifier-cancel" type="button" class="relative -ml-px inline-flex items-center gap-x-1.5 px-3 py-1 text-gray-800 shadow-sm ring-1 ring-inset ring-gray-400 bg-white hover:bg-gray-50" title="Cancel">
					<XMarkIcon class="w-5 h-5" />
				</button>
			</div>
		</div>
	</div>

	<Teleport to="body">
		<div v-if="duplicateDrawerLine" class="fixed inset-0 z-50 flex">
			<div class="flex-1 bg-black/40" @click="closeDuplicateDrawer"></div>
			<aside class="w-full max-w-md bg-white shadow-2xl border-l border-gray-200 flex flex-col">
				<div class="flex items-center justify-between border-b border-gray-200 px-4 py-3">
					<h2 class="text-base font-semibold text-gray-900">Duplicate details</h2>
					<button type="button" class="text-gray-500 hover:text-gray-700" @click="closeDuplicateDrawer">
						<XMarkIcon class="w-5 h-5" />
					</button>
				</div>
				<div class="flex-1 overflow-y-auto px-4 py-5 space-y-8">
					<section>
						<h3 class="text-sm font-semibold text-gray-700">Selected line</h3>
						<dl class="mt-2 space-y-1 text-sm text-gray-900">
							<div class="flex justify-between"><dt class="text-gray-500">Source</dt><dd>{{ duplicateDrawerLine!.source_account }}</dd></div>
							<div class="flex justify-between"><dt class="text-gray-500">Date</dt><dd>{{ dayjs(duplicateDrawerLine!.dt).format('YYYY-MM-DD') }}</dd></div>
							<div class="flex justify-between"><dt class="text-gray-500">Description</dt><dd class="text-right">{{ duplicateDrawerLine!.description }}</dd></div>
							<div class="flex justify-between"><dt class="text-gray-500">Amount</dt><dd>{{ formatAmountForTooltip(duplicateDrawerLine!.quantity, duplicateDrawerLine!.commodity) }}</dd></div>
						</dl>
						<div class="mt-3 flex flex-wrap gap-2">
							<button type="button" class="btn-secondary" @click="focusSelectedLine">Locate in table</button>
							<button type="button" class="btn-secondary" @click="markSelectedLineNotDuplicate">Mark as not duplicate</button>
							<button type="button" class="btn-secondary text-red-700 ring-red-600" @click="deleteSelectedLine">Delete line</button>
						</div>
						<p v-if="duplicateDrawerReasonText" class="mt-3 text-sm text-amber-700">{{ duplicateDrawerReasonText }}</p>
					</section>
					<section v-if="duplicateDrawerMatch">
						<h3 class="text-sm font-semibold text-gray-700">Matched line</h3>
						<dl class="mt-2 space-y-1 text-sm text-gray-900">
							<div v-if="duplicateDrawerMatchDisplay && 'source_account' in duplicateDrawerMatchDisplay" class="flex justify-between"><dt class="text-gray-500">Source</dt><dd>{{ duplicateDrawerMatchDisplay.source_account }}</dd></div>
							<div class="flex justify-between"><dt class="text-gray-500">Date</dt><dd>{{ dayjs(duplicateDrawerMatchDisplay!.dt).format('YYYY-MM-DD') }}</dd></div>
							<div class="flex justify-between"><dt class="text-gray-500">Description</dt><dd class="text-right">{{ duplicateDrawerMatchDisplay!.description }}</dd></div>
							<div class="flex justify-between"><dt class="text-gray-500">Amount</dt><dd>{{ formatAmountForTooltip(duplicateDrawerMatchDisplay!.quantity, duplicateDrawerMatchDisplay!.commodity) }}</dd></div>
						</dl>
						<div class="mt-3 flex flex-wrap gap-2">
							<button v-if="duplicateDrawerMatch?.kind === 'existing'" type="button" class="btn-secondary" @click="jumpToMatchingLine">Jump to matching line</button>
						</div>
					</section>
					<section v-else>
						<p class="text-sm text-gray-600">No matching line information is available.</p>
					</section>
				</div>
			</aside>
		</div>
	</Teleport>
	</div>
</template>

<script setup lang="ts">
	import Clusterize from 'clusterize.js';
	
	import dayjs from 'dayjs';
	
	import { CheckIcon, PencilIcon, XMarkIcon } from '@heroicons/vue/24/outline';
	
	import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
	
	import ComboBoxAccounts from '../components/ComboBoxAccounts.vue';
	import { db } from '../db.ts';
	import type { AnnotatedStatementLine, DuplicateMatch } from '../importers/deduplicate.ts';
	import { renderComponent } from '../webutil.ts';
	import { ppWithCommodity } from '../display.ts';
	
	interface ViewStatementLine extends AnnotatedStatementLine {
		posting_accounts: string[];
		transaction_id: number | null;
		dedup_ignore: number;
	}
	
    const showOnlyUnclassified = ref(false);
    const statementLines = ref([] as ViewStatementLine[]);
    const selectAll = ref(false);
    const searchQuery = ref('');
	
	const classificationLineId = ref(0);
	const classificationAccount = ref('');
	const isBatchClassify = ref(false);
	const batchSelectedLineIds = ref([] as number[]);
	const selectedCount = ref(0);
	const showOnlyDuplicates = ref(false);
	const duplicateDrawerLineId = ref<number | null>(null);

	const duplicateDrawerLine = computed(() => {
		if (duplicateDrawerLineId.value === null) {
			return null;
		}
		return statementLines.value.find((line) => line.id === duplicateDrawerLineId.value) ?? null;
	});

	const duplicateDrawerMatch = computed(() => duplicateDrawerLine.value?.duplicateMatch ?? null);

	const duplicateDrawerMatchLine = computed(() => {
		const match = duplicateDrawerMatch.value;
		if (match && match.kind === 'existing') {
			return statementLines.value.find((line) => line.id === match.statementLine.id) ?? null;
		}
		return null;
	});

	const duplicateDrawerReasonText = computed(() => {
		const line = duplicateDrawerLine.value;
		if (!line || !line.duplicateReason) {
			return null;
		}
		return formatDuplicateReason(line.duplicateReason);
	});

	const duplicateDrawerMatchFallback = computed(() => {
		const match = duplicateDrawerMatch.value;
		if (!match) {
			return null;
		}
		if (match.kind === 'existing') {
			return match.statementLine;
		}
		return match.previousLine;
	});

	const duplicateDrawerMatchDisplay = computed(() => duplicateDrawerMatchLine.value ?? duplicateDrawerMatchFallback.value);
	
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
		const newStatementLines: ViewStatementLine[] = [];
		
		for (const joinedStatementLine of joinedStatementLines) {
			if (newStatementLines.length === 0 || newStatementLines.at(-1)!.id !== joinedStatementLine.id) {
				newStatementLines.push({
					id: joinedStatementLine.id,
					source_account: joinedStatementLine.source_account,
					dt: joinedStatementLine.dt,
					name: joinedStatementLine.name ?? '',
					memo: joinedStatementLine.memo ?? '',
					description: joinedStatementLine.description,
					quantity: joinedStatementLine.quantity,
					balance: joinedStatementLine.balance,
					commodity: joinedStatementLine.commodity,
					fitid: joinedStatementLine.fitid,
					transaction_id: joinedStatementLine.transaction_id,
					posting_accounts: [],
					dedup_ignore: joinedStatementLine.dedup_ignore ?? 0,
					duplicate: false,
					duplicateReason: null,
					duplicateMatch: null
				});
			}
			if (joinedStatementLine.posting_account !== null) {
				newStatementLines.at(-1)!.posting_accounts.push(joinedStatementLine.posting_account);
			}
		}
		
		const linesByAccount = new Map<string, ViewStatementLine[]>();
		for (const line of newStatementLines) {
			if (!linesByAccount.has(line.source_account)) {
				linesByAccount.set(line.source_account, []);
			}
			linesByAccount.get(line.source_account)!.push(line);
		}

		for (const linesForAccount of linesByAccount.values()) {
			annotateExistingDuplicates(linesForAccount);
		}

		if (duplicateDrawerLineId.value !== null) {
			const currentDrawerLine = newStatementLines.find((line) => line.id === duplicateDrawerLineId.value);
			if (!currentDrawerLine || !currentDrawerLine.duplicate) {
				duplicateDrawerLineId.value = null;
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
            // Update batch selection count for top-bar button visibility
            selectedCount.value = checkedBoxes.length;
        }
        if (event.target && (event.target as Element).classList.contains('duplicate-badge')) {
            event.preventDefault();
            const tr = (event.target as Element).closest('tr');
            const lineId = tr?.dataset.lineId ? parseInt(tr.dataset.lineId) : NaN;
            if (!Number.isNaN(lineId)) {
                openDuplicateDrawerForLine(lineId);
            }
            return;
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
			positionClassifierAtElement(td);
			
			// Focus classify line panel
			const divReconciler = document.getElementById('statement-line-classifier')!;
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
		
		const chargeAccount = classificationAccount.value;
		
		if (!chargeAccount) {
			return;
		}
		
		// Disable further submissions
		(document.querySelector('.statement-line-classifier-input')! as HTMLInputElement).disabled = true;
		(document.getElementById('statement-line-classifier-button')! as HTMLButtonElement).disabled = true;
		
		if (isBatchClassify.value) {
			await classifySelectedWithAccount(chargeAccount);
		} else {
			const lineId = classificationLineId.value;
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
		const query = searchQuery.value.trim().toLowerCase();
		
        for (const line of statementLines.value) {
            // Filter by description if a search query is provided
            if (query && !line.description.toLowerCase().includes(query)) {
                continue;
            }
            if (showOnlyDuplicates.value && !line.duplicate) {
                continue;
            }
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
			
			const duplicateBadge = duplicateBadgeHtml(line);
			const rowClassAttr = line.duplicate ? ' class="bg-amber-50"' : '';
			rows.push(
				`<tr data-line-id="${ line.id }"${ rowClassAttr }>
					<td class="py-0.5 pr-1 align-baseline">${ checkboxCell }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900"><a href="/transactions/${ encodeURIComponent(line.source_account) }" class="hover:text-blue-700 hover:underline">${ line.source_account }</a></td>
					<td class="py-0.5 px-1 align-baseline text-gray-900 lg:w-[12ex]">${ dayjs(line.dt).format('YYYY-MM-DD') }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900">${ line.description }${ duplicateBadge }</td>
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

		// Sync selected count after render in case of updates
		selectedCount.value = document.querySelectorAll('.statement-line-checkbox:checked').length;
	}

	function duplicateBadgeHtml(line: ViewStatementLine): string {
		if (!line.duplicate) {
			return '';
		}
		const tooltip = escapeHtml(duplicateTooltip(line));
		return ` <button type="button" class="duplicate-badge ml-1 inline-flex items-center rounded-full border border-amber-400 bg-amber-50 px-1.5 py-0 text-xs uppercase tracking-wide text-amber-800" title="${ tooltip }">Duplicate</button>`;
	}

	function annotateExistingDuplicates(lines: ViewStatementLine[]): void {
		const sorted = [...lines].sort(function(a, b) {
			const dtCompare = a.dt.localeCompare(b.dt);
			if (dtCompare !== 0) {
				return dtCompare;
			}
			const idA = a.id ?? Number.MIN_SAFE_INTEGER;
			const idB = b.id ?? Number.MIN_SAFE_INTEGER;
			return idA - idB;
		});

		const fitidMap = new Map<string, ViewStatementLine>();
		const signatureMap = new Map<string, ViewStatementLine>();
		const dateAmountMap = new Map<string, ViewStatementLine>();

		for (const line of sorted) {
			if (line.dedup_ignore) {
				line.duplicate = false;
				line.duplicateReason = null;
				line.duplicateMatch = null;
				continue;
			}
			let duplicate = false;
			let reason: ViewStatementLine['duplicateReason'] = null;
			let matchLine: ViewStatementLine | null = null;

			const fitid = line.fitid?.trim();
			if (fitid) {
				const existing = fitidMap.get(fitid);
				if (existing && (existing.id ?? Number.MIN_SAFE_INTEGER) !== (line.id ?? Number.MIN_SAFE_INTEGER)) {
					duplicate = true;
					reason = 'existing-fitid';
					matchLine = existing;
				} else if (!fitidMap.has(fitid)) {
					fitidMap.set(fitid, line);
				}
			}

			if (!duplicate) {
				const signature = signatureKey(line);
				const existing = signatureMap.get(signature);
				if (existing && (existing.id ?? Number.MIN_SAFE_INTEGER) !== (line.id ?? Number.MIN_SAFE_INTEGER)) {
					duplicate = true;
					reason = 'existing-signature';
					matchLine = existing;
				} else if (!signatureMap.has(signature)) {
					signatureMap.set(signature, line);
				}
			}

			if (!duplicate) {
				const dateAmount = dateAmountKey(line);
				const existing = dateAmountMap.get(dateAmount);
				if (existing && (existing.id ?? Number.MIN_SAFE_INTEGER) !== (line.id ?? Number.MIN_SAFE_INTEGER)) {
					duplicate = true;
					reason = 'existing-date-amount';
					matchLine = existing;
				} else if (!dateAmountMap.has(dateAmount)) {
					dateAmountMap.set(dateAmount, line);
				}
			}

			line.duplicate = duplicate;
			line.duplicateReason = reason;
			line.duplicateMatch = duplicate && matchLine ? toDuplicateMatch(matchLine) : null;
		}
	}

	function openDuplicateDrawerForLine(lineId: number) {
		const line = statementLines.value.find((l) => l.id === lineId);
		if (!line || !line.duplicate) {
			duplicateDrawerLineId.value = null;
			return;
		}
		duplicateDrawerLineId.value = lineId;
		scrollLineIntoView(lineId);
		highlightLine(lineId);
	}

	function closeDuplicateDrawer() {
		duplicateDrawerLineId.value = null;
	}

	function jumpToMatchingLine() {
		const match = duplicateDrawerMatch.value;
		if (match && match.kind === 'existing') {
			const targetId = match.statementLine.id;
			scrollLineIntoView(targetId);
			highlightLine(targetId);
			if (statementLines.value.some((line) => line.id === targetId)) {
				duplicateDrawerLineId.value = targetId;
			}
		}
	}

	function focusSelectedLine() {
		const line = duplicateDrawerLine.value;
		if (!line || line.id === null) {
			return;
		}
		scrollLineIntoView(line.id);
		highlightLine(line.id);
	}

	function scrollLineIntoView(lineId: number | null) {
		if (lineId === null) {
			return;
		}
		const row = document.querySelector(`#statement-line-list tr[data-line-id="${ lineId }"]`);
		if (row instanceof HTMLElement) {
			row.scrollIntoView({ block: 'center' });
		}
	}

	function highlightLine(lineId: number | null) {
		if (lineId === null) {
			return;
		}
		const row = document.querySelector(`#statement-line-list tr[data-line-id="${ lineId }"]`);
		if (!(row instanceof HTMLElement)) {
			return;
		}
		row.classList.add('ring', 'ring-amber-400');
		setTimeout(() => {
			row.classList.remove('ring', 'ring-amber-400');
		}, 1500);
	}

	async function markSelectedLineNotDuplicate() {
		const line = duplicateDrawerLine.value;
		if (!line || line.id === null) {
			return;
		}
		const session = await db.load();
		await session.execute(
			`UPDATE statement_lines SET dedup_ignore = 1 WHERE id = ?`,
			[line.id]
		);
		await load();
		closeDuplicateDrawer();
	}

	async function deleteSelectedLine() {
		const line = duplicateDrawerLine.value;
		if (!line || line.id === null) {
			return;
		}
		if (!await confirm('Delete this statement line? This action cannot be undone.')) {
			return;
		}
		const session = await db.load();
		const tx = await session.begin();
		await tx.execute(`DELETE FROM statement_line_reconciliations WHERE statement_line_id = ?`, [line.id]);
		await tx.execute(`DELETE FROM statement_lines WHERE id = ?`, [line.id]);
		await tx.commit();
		await load();
		closeDuplicateDrawer();
	}

	function duplicateTooltip(line: ViewStatementLine): string {
		if (!line.duplicate || !line.duplicateMatch) {
			return line.duplicateReason ? formatDuplicateReason(line.duplicateReason) : 'Marked as duplicate';
		}
		if (line.duplicateMatch.kind === 'existing') {
			const match = line.duplicateMatch.statementLine;
			const parts = [`Existing line #${ match.id }`, dayjs(match.dt).format('YYYY-MM-DD'), formatAmountForTooltip(match.quantity, match.commodity)];
			const description = summariseDescription(match.description, match.name, match.memo);
			if (description) {
				parts.push(description);
			}
			return parts.join(' · ');
		}
		const previous = line.duplicateMatch.previousLine;
		const parts = ['Matches earlier line in this file', dayjs(previous.dt).format('YYYY-MM-DD'), formatAmountForTooltip(previous.quantity, previous.commodity)];
		const description = summariseDescription(previous.description, previous.name, previous.memo);
		if (description) {
			parts.push(description);
		}
		return parts.join(' · ');
	}

	function formatDuplicateReason(reason: NonNullable<ViewStatementLine['duplicateReason']>): string {
		switch (reason) {
			case 'existing-fitid':
				return 'Duplicate (already imported – FITID)';
			case 'file-fitid':
				return 'Duplicate (within file – FITID)';
			case 'existing-signature':
				return 'Duplicate (already imported – date/amount/description)';
			case 'file-signature':
				return 'Duplicate (within file – date/amount/description)';
			case 'existing-date-amount':
				return 'Duplicate (already imported – date/amount)';
			case 'file-date-amount':
				return 'Duplicate (within file – date/amount)';
			default:
				return 'Duplicate';
		}
	}

	function summariseDescription(description: string, name?: string | null, memo?: string | null): string {
		const values = [description, name ?? '', memo ?? '']
			.map((value) => value.trim())
			.filter((value) => value.length > 0);
		return Array.from(new Set(values)).join(' · ');
	}

	function formatAmountForTooltip(quantity: number, commodity: string): string {
		if (quantity > 0) {
			return 'Dr ' + ppWithCommodity(quantity, commodity);
		}
		if (quantity < 0) {
			return 'Cr ' + ppWithCommodity(-quantity, commodity);
		}
		return ppWithCommodity(0, commodity);
	}

	function escapeHtml(text: string): string {
		return text.replaceAll(/&/g, '&amp;')
			.replaceAll(/</g, '&lt;')
			.replaceAll(/>/g, '&gt;')
			.replaceAll(/"/g, '&quot;')
			.replaceAll(/'/g, '&#39;');
	}

	function signatureKey(line: ViewStatementLine): string {
		return [normaliseComponent(line.dt), line.quantity, normaliseComponent(line.description), normaliseComponent(line.name), normaliseComponent(line.memo)].join('|');
	}

	function dateAmountKey(line: ViewStatementLine): string {
		return `${ normaliseComponent(line.dt) }|${ line.quantity }`;
	}

	function normaliseComponent(value: string | null | undefined): string {
		return (value ?? '').trim().replace(/\s+/g, ' ');
	}

	function toDuplicateMatch(line: ViewStatementLine): DuplicateMatch {
		return {
			kind: 'existing',
			statementLine: {
				id: line.id ?? -1,
				fitid: line.fitid,
				dt: line.dt,
				description: line.description,
				name: line.name,
				memo: line.memo,
				quantity: line.quantity,
				commodity: line.commodity,
				dedup_ignore: line.dedup_ignore
			}
		};
	}
	
	watch(showOnlyUnclassified, renderTable);
	watch(showOnlyDuplicates, renderTable);
	watch(statementLines, () => {
		if (duplicateDrawerLineId.value !== null) {
			const current = statementLines.value.find((line) => line.id === duplicateDrawerLineId.value);
			if (!current || !current.duplicate) {
				duplicateDrawerLineId.value = null;
			}
		}
		renderTable();
	});
	watch(searchQuery, renderTable);
	
	load();

	function positionClassifierAtElement(el: Element, toRightOf: boolean = false) {
		const outerDiv = document.getElementById('statement-line-list')!;
		const divReconciler = document.getElementById('statement-line-classifier')!;
		divReconciler.classList.remove('hidden');
		const elRect = el.getBoundingClientRect();
		const outerRect = outerDiv.getBoundingClientRect();
		if (toRightOf) {
			// Use fixed positioning so it overlays the sticky toolbar/buttons
			divReconciler.style.position = 'fixed';
			divReconciler.style.top = (elRect.top - 4) + 'px';
			divReconciler.style.left = (elRect.right + 8) + 'px';
		} else {
			// Position relative to the scrollable list container
			divReconciler.style.position = 'absolute';
			divReconciler.style.top = (outerDiv.scrollTop + elRect.y - outerRect.y - 4) + 'px';
			divReconciler.style.left = (elRect.x - outerRect.x) + 'px';
		}
	}

	function closeClassifier() {
		// Reset UI state and hide classifier panel
		classificationAccount.value = '';
		isBatchClassify.value = false;
		batchSelectedLineIds.value = [];
		
		const inputEl = document.querySelector('.statement-line-classifier-input') as HTMLInputElement | null;
		if (inputEl) { inputEl.disabled = false; }
		const okBtn = document.getElementById('statement-line-classifier-button') as HTMLButtonElement | null;
		if (okBtn) { okBtn.disabled = false; }
		
		const classifier = document.getElementById('statement-line-classifier');
		if (classifier) { classifier.classList.add('hidden'); }
		
		// Unhide any reconciliation cells that were hidden
		for (const el of document.querySelectorAll('#statement-line-list .charge-account > span')) {
			el.classList.remove('invisible');
		}
	}

	function onGlobalKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			const classifier = document.getElementById('statement-line-classifier');
			if (classifier && !classifier.classList.contains('hidden')) {
				e.preventDefault();
				closeClassifier();
				return;
			}
			if (duplicateDrawerLineId.value !== null) {
				e.preventDefault();
				closeDuplicateDrawer();
				return;
			}
			// Clear search if present and classifier not open
			if (searchQuery.value !== '') {
				e.preventDefault();
				searchQuery.value = '';
				return;
			}
		}
	}

	onMounted(() => {
		window.addEventListener('keydown', onGlobalKeydown);
	});

	function openBatchClassifier(event: MouseEvent) {
		// Prepare batch selection and open classifier near the clicked button
		isBatchClassify.value = true;
		batchSelectedLineIds.value = [...document.querySelectorAll('.statement-line-checkbox:checked')]
			.map((el) => parseInt(el.closest('tr')?.dataset.lineId!));
		
		// Make sure any previously hidden cells are visible
		for (const el of document.querySelectorAll('#statement-line-list .charge-account > span')) {
			el.classList.remove('invisible');
		}
		
		// Position classifier to the right of the clicked button
		const target = (event.currentTarget as Element) ?? (event.target as Element);
		positionClassifierAtElement(target, true);
		
		// Focus classify line panel
		const divReconciler = document.getElementById('statement-line-classifier')!;
		divReconciler.querySelector('input')!.focus();
	}

	async function classifySelectedWithAccount(chargeAccount: string) {
		if (batchSelectedLineIds.value.length === 0) {
			return;
		}
		
		// Check if account exists (once for batch)
		const session = await db.load();
		const countResult = await session.select('SELECT COUNT(*) FROM postings WHERE account = $1', [chargeAccount]) as any[];
		const doesAccountExist = countResult[0]['COUNT(*)'] > 0;
		if (!doesAccountExist) {
			if (!await confirm('Account "' + chargeAccount + '" does not exist. Continue to reconcile these transactions and create a new account?')) {
				(document.querySelector('.statement-line-classifier-input')! as HTMLInputElement).disabled = false;
				(document.getElementById('statement-line-classifier-button')! as HTMLButtonElement).disabled = false;
				return;
			}
		}
		
		let skipped = 0;
		for (const lineId of batchSelectedLineIds.value) {
			const statementLine = statementLines.value.find((l) => l.id === lineId);
			if (!statementLine) { continue; }
			if (statementLine.posting_accounts.length !== 0) { skipped++; continue; }
			
			const dbTransaction = await session.begin();
			const transactionResult = await dbTransaction.execute(
				`INSERT INTO transactions (dt, description)
				VALUES ($1, $2)`,
				[statementLine.dt, statementLine.description]
			);
			const transactionId = transactionResult.lastInsertId;
			
			const accountPostingResult = await dbTransaction.execute(
				`INSERT INTO postings (transaction_id, description, account, quantity, commodity)
				VALUES ($1, NULL, $2, $3, $4)`,
				[transactionId, statementLine.source_account, statementLine.quantity, statementLine.commodity]
			);
			const accountPostingId = accountPostingResult.lastInsertId;
			
			await dbTransaction.execute(
				`INSERT INTO postings (transaction_id, description, account, quantity, commodity)
				VALUES ($1, NULL, $2, $3, $4)`,
				[transactionId, chargeAccount, -statementLine.quantity, statementLine.commodity]
			);
			
			await dbTransaction.execute(
				`INSERT INTO statement_line_reconciliations (statement_line_id, posting_id)
				VALUES ($1, $2)`,
				[statementLine.id, accountPostingId]
			);
			
			await dbTransaction.commit();
		}
		
		// Reset UI state
		isBatchClassify.value = false;
		batchSelectedLineIds.value = [];
		classificationAccount.value = '';
		(document.querySelector('.statement-line-classifier-input')! as HTMLInputElement).disabled = false;
		(document.getElementById('statement-line-classifier-button')! as HTMLButtonElement).disabled = false;
		document.getElementById('statement-line-classifier')!.classList.add('hidden');
		
		// Feedback if any skipped
		if (skipped > 0) {
			await alert(skipped + ' selected lines already reconciled and were skipped');
		}
		
		// Reload and re-render
		await load();
	}
	
	onUnmounted(() => {
		window.removeEventListener('keydown', onGlobalKeydown);
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>

<style scoped>
#statement-line-classifier {
  /* Ensure classifier appears above sticky headers/buttons */
  @apply z-50;
}
</style>
