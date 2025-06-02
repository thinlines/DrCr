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

import { resolveResource } from '@tauri-apps/api/path';
import { readTextFile } from '@tauri-apps/plugin-fs';

// Dodgy implementation to parse Lua table as JSON
function parseLua(luaArray: string): any[] {
	luaArray = luaArray.trim();
	
	// Remove surrounding { and }
	if (!luaArray.startsWith('{') || !luaArray.endsWith('}')) {
		throw new Error('Unparseable Lua array');
	}
	luaArray = luaArray.substring(1, luaArray.length - 1).trim();
	
	if (luaArray.endsWith(',')) {
		// Remove trailing comma as this is invalid JSON
		luaArray = luaArray.substring(0, luaArray.length - 1);
	}
	
	// Replace Lua {...} with JSON [...]
	luaArray = luaArray.replaceAll('{', '[').replaceAll('}', ']');
	
	// Replace single quotes with double quotes
	luaArray = luaArray.replaceAll(/(?<!\\)'/g, '"');
	luaArray = luaArray.replaceAll("\\'", "'");
	
	return JSON.parse('[' + luaArray + ']');
}

export async function getAccountKinds(): Promise<[string, string][]> {
	// Read contents of account_kinds.luau
	const luaFilePath = await resolveResource('plugins/austax/account_kinds.luau');
	const luaFileContent = await readTextFile(luaFilePath);
	
	const accountKinds: [string, string][] = [];
	
	// Parse income_types
	const incomeTypesLua = luaFileContent.match(/local income_types = ({.*?\n})\n/s)![1];
	const incomeTypes = parseLua(incomeTypesLua);
	for (const [code, name, number] of incomeTypes) {
		accountKinds.push(['austax.' + code, name + ' (' + number + ')']);
	}
	
	// Parse deduction_types
	const deductionTypesLua = luaFileContent.match(/local deduction_types = ({.*?\n})\n/s)![1];
	const deductionTypes = parseLua(deductionTypesLua);
	for (const [code, name, number] of deductionTypes) {
		accountKinds.push(['austax.' + code, name + ' (' + number + ')']);
	}
	
	// Hard-coded types
	accountKinds.push(['austax.offset', 'Tax offset']);
	accountKinds.push(['austax.paygw', 'PAYG withheld amounts']);
	accountKinds.push(['austax.cgtasset', 'CGT asset']);
	accountKinds.push(['austax.rfb', 'Reportable fringe benefit']);
	
	return accountKinds;
}
