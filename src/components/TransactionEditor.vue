<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2025  Lee Yingtong Li (RunasSudo)
	
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
	<table class="min-w-full">
		<thead>
			<tr class="border-b border-gray-300">
				<th class="pt-0.5 pb-1 pr-1 text-gray-900 font-semibold text-start">Date</th>
				<th class="pt-0.5 pb-1 px-1 text-gray-900 font-semibold text-start" colspan="2">Description</th>
				<th class="pt-0.5 pb-1 px-1 text-gray-900 font-semibold text-start">Dr</th>
				<th class="pt-0.5 pb-1 pl-1 text-gray-900 font-semibold text-start">Cr</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td class="pt-2 pb-1 pr-1">
					<input type="date" class="bordered-field" v-model="transaction.dt">
				</td>
				<td class="pt-2 pb-1 px-1" colspan="2">
					<input type="text" class="bordered-field" v-model="transaction.description">
				</td>
				<td></td>
				<td></td>
			</tr>
			<tr v-for="posting in transaction.postings">
				<td></td>
				<td class="py-1 px-1">{{ posting.description }}</td>
				<td class="py-1 px-1">
					<div class="relative flex">
						<div class="relative flex flex-grow items-stretch shadow-sm">
							<div class="absolute inset-y-0 left-0 flex items-center z-10">
								<select class="h-full border-0 bg-transparent py-0 pl-2 pr-8 text-gray-900 focus:ring-2 focus:ring-inset focus:ring-emerald-600" v-model="posting.sign">
									<option value="dr">Dr</option>
									<option value="cr">Cr</option>
								</select>
							</div>
							<ComboBoxAccounts v-model="posting.account" class="w-full" inputClass="pl-16" />
						</div>
						<button class="relative -ml-px px-2 py-2 text-gray-500 hover:text-gray-700" @click="addPosting(posting)">
							<PlusIcon class="w-4 h-4" />
						</button>
					</div>
				</td>
				<template v-if="posting.sign == 'dr'">
					<td class="amount-dr has-amount py-1 px-1">
						<div class="relative shadow-sm">
							<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
								<span class="text-gray-500">{{ db.metadata.reporting_commodity }}</span>
							</div>
							<input type="text" class="bordered-field pl-7" v-model="posting.amount_abs">
						</div>
					</td>
					<td class="amount-cr py-1 pl-1"></td>
				</template>
				<template v-if="posting.sign == 'cr'">
					<td class="amount-dr py-1 px-1"></td>
					<td class="amount-cr has-amount py-1 pl-1">
						<div class="relative shadow-sm">
							<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
								<span class="text-gray-500">{{ db.metadata.reporting_commodity }}</span>
							</div>
							<input type="text" class="bordered-field pl-7" v-model="posting.amount_abs">
						</div>
					</td>
				</template>
			</tr>
		</tbody>
	</table>
	
	<div class="flex justify-end mt-4 space-x-2">
		<button class="btn-secondary text-red-600 ring-red-500" @click="deleteTransaction" v-if="transaction.id !== null">Delete</button>
		<button class="btn-primary" @click="saveTransaction">Save</button>
	</div>
	
	<div class="rounded-md bg-red-50 mt-4 p-4 col-span-2" v-if="error !== null">
		<div class="flex">
			<div class="flex-shrink-0">
				<XCircleIcon class="h-5 w-5 text-red-400" />
			</div>
			<div class="ml-3 flex-1">
				<p class="text-sm text-red-700">{{ error }}</p>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	
	import { PlusIcon, XCircleIcon } from '@heroicons/vue/24/solid';
	
	import { emit } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	
	import { ref } from 'vue';
	
	import { DT_FORMAT, Posting, Transaction, db, deserialiseAmount } from '../db.ts';
	import ComboBoxAccounts from './ComboBoxAccounts.vue';
	
	interface EditingPosting {
		id: number | null,
		description: string | null,
		account: string,
		originalAccount: string | null,
		sign: string,  // Keep track of Dr/Cr status so this can be independently changed in the UI
		amount_abs: string,
	}
	export interface EditingTransaction {
		id: number | null,
		dt: string,
		description: string,
		postings: EditingPosting[]
	}
	
	const { transaction } = defineProps<{ transaction: EditingTransaction }>();
	
	const error = ref(null as string | null);
	
	function addPosting(posting: EditingPosting) {
		const index = transaction.postings.indexOf(posting);
		transaction.postings.splice(index + 1, 0, {
			id: null,
			description: null,
			account: '',
			originalAccount: null,
			sign: posting.sign,  // Create the new posting with the same sign as the entry clicked on
			amount_abs: ''
		});
	}
	
	async function saveTransaction() {
		error.value = null;
		
		// Prepare transaction for save
		const newTransaction = new Transaction(
			transaction.id,
			dayjs(transaction.dt).format(DT_FORMAT),
			transaction.description,
			[]
		);
		
		for (const posting of transaction.postings) {
			const amount_abs = deserialiseAmount(posting.amount_abs);
			
			newTransaction.postings.push({
				id: posting.id,
				description: posting.description,
				account: posting.account,
				originalAccount: posting.originalAccount,
				quantity: posting.sign === 'dr' ? amount_abs.quantity : -amount_abs.quantity,
				commodity: amount_abs.commodity
			} as Posting);
		}
		
		// Validate transaction
		if (!newTransaction.doesBalance()) {
			error.value = 'Debits and credits do not balance.';
			return;
		}
		
		const session = await db.load();
		
		// Validate statement line reconciliations
		// Keep track of mapping, so we can fix up the reconciliation posting_id if renumbering occurs
		const postingsToReconciliations = new Map();
		
		if (newTransaction.id !== null) {
			// Get statement line reconciliations affected by this transaction
			const joinedReconciliations: any[] = await session.select(
				`SELECT statement_line_reconciliations.id, postings.id AS posting_id, source_account, statement_lines.quantity, statement_lines.commodity
				FROM statement_line_reconciliations
				JOIN postings ON statement_line_reconciliations.posting_id = postings.id
				JOIN statement_lines ON statement_line_reconciliations.statement_line_id = statement_lines.id
				WHERE postings.transaction_id = $1`,
				[newTransaction.id]
			);
			
			for (const joinedReconciliation of joinedReconciliations) {
				for (const posting of newTransaction.postings) {
					if (posting.id === joinedReconciliation.posting_id) {
						if (posting.account !== joinedReconciliation.source_account || posting.quantity !== joinedReconciliation.quantity || posting.commodity !== joinedReconciliation.commodity) {
							error.value = 'Edit would break reconciled statement line.';
							return;
						}
						postingsToReconciliations.set(posting, joinedReconciliation);
					}
				}
			}
		}
		
		// Save changes to database atomically
		const dbTransaction = await session.begin();
		
		if (newTransaction.id === null) {
			// Insert new transaction
			const result = await dbTransaction.execute(
				`INSERT INTO transactions (dt, description)
				VALUES ($1, $2)`,
				[newTransaction.dt, newTransaction.description]
			);
			newTransaction.id = result.lastInsertId;
		} else {
			// Update existing transaction
			await dbTransaction.execute(
				`UPDATE transactions
				SET dt = $1, description = $2
				WHERE id = $3`,
				[newTransaction.dt, newTransaction.description, newTransaction.id]
			);
		}
		
		let insertPostings = false;
		
		for (const posting of newTransaction.postings) {
			if (posting.id === null) {
				// When we encounter a new posting, delete and re-insert all subsequent postings to preserve the order
				insertPostings = true;
			}
			
			if (insertPostings) {
				// Delete existing posting if required
				if (posting.id !== null) {
					await dbTransaction.execute(
						`DELETE FROM postings
						WHERE id = $1`,
						[posting.id]
					);
				}
				
				// Insert new posting
				const result = await dbTransaction.execute(
					`INSERT INTO postings (transaction_id, description, account, quantity, commodity)
					VALUES ($1, $2, $3, $4, $5)`,
					[newTransaction.id, posting.description, posting.account, posting.quantity, posting.commodity]
				);
				
				// Fixup reconciliation if required
				const joinedReconciliation = postingsToReconciliations.get(posting);
				if (joinedReconciliation) {
					await dbTransaction.execute(
						`UPDATE statement_line_reconciliations
						SET posting_id = $1
						WHERE id = $2`,
						[result.lastInsertId, joinedReconciliation.id]
					);
				}
			} else {
				// Update existing posting
				await dbTransaction.execute(
					`UPDATE postings
					SET description = $1, account = $2, quantity = $3, commodity = $4
					WHERE id = $5`,
					[posting.description, posting.account, posting.quantity, posting.commodity, posting.id]
				);
			}
		}
		
		await dbTransaction.commit();
		
		await emit('transaction-updated', {id: newTransaction.id});
		await getCurrentWindow().close();
	}
	
	async function deleteTransaction() {
		if (!await confirm('Are you sure you want to delete this transaction? This operation is irreversible.')) {
			return;
		}
		
		// Delete atomically
		const session = await db.load();
		const dbTransaction = await session.begin();
		
		// Cascade delete statement line reconciliations
		await dbTransaction.execute(
			`DELETE FROM statement_line_reconciliations
			WHERE posting_id IN (
				SELECT postings.id FROM postings WHERE transaction_id = $1
			)`,
			[transaction.id]
		);
		
		// Delete postings
		await dbTransaction.execute(
			`DELETE FROM postings
			WHERE transaction_id = $1`,
			[transaction.id]
		);
		
		// Delete transaction
		await dbTransaction.execute(
			`DELETE FROM transactions
			WHERE id = $1`,
			[transaction.id]
		);
		
		await dbTransaction.commit();
		
		await getCurrentWindow().close();
	}
</script>
