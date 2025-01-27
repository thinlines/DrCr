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
	<h1 class="page-heading mb-4">
		Edit transaction
	</h1>
	
	<TransactionEditor :transaction="transaction" />
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	
	import { ref } from 'vue';
	import { useRoute } from 'vue-router';
	
	import { JoinedTransactionPosting, db, joinedToTransactions, serialiseAmount } from '../db.ts';
	import TransactionEditor, { EditingTransaction } from '../components/TransactionEditor.vue';
	
	const route = useRoute();
	
	const transaction = ref({
		id: null,
		dt: null!,
		description: null!,
		postings: []
	} as EditingTransaction);
	
	async function load() {
		const session = await db.load();
		
		const joinedTransactionPostings: JoinedTransactionPosting[] = await session.select(
			`SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity
			FROM transactions
			JOIN postings ON transactions.id = postings.transaction_id
			WHERE transactions.id = $1
			ORDER BY postings.id`,
			[route.params.id]
		);
		
		const transactions = joinedToTransactions(joinedTransactionPostings);
		if (transactions.length !== 1) { throw new Error('Unexpected number of transactions returned from SQL'); }
		const rawTransaction = transactions[0] as any;
		
		// Format dt
		rawTransaction.dt = dayjs(rawTransaction.dt).format('YYYY-MM-DD');
		
		// Initialise originalAccount, sign and amount_abs
		for (const posting of rawTransaction.postings) {
			posting.originalAccount = posting.account;
			posting.sign = posting.quantity >= 0 ? 'dr' : 'cr';
			posting.amount_abs = serialiseAmount(Math.abs(posting.quantity), posting.commodity);
		}
		
		transaction.value = rawTransaction as EditingTransaction;
	}
	load();
</script>
