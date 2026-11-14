/**
 * Relevé CAMT.053 **synthétique** de `n` transactions, extrait de
 * `reconciliation-show-older.spec.ts` (Story 29-9) par la Story 35-4 (#1024, étape 0), qui en
 * monte un pour dépasser les cent transactions de la page *Réconciliation*.
 *
 * L'entrée de `camt053_v04_minimal.xml` est répétée `n` fois : montant, `AcctSvcrRef`,
 * `EndToEndId` et contrepartie distincts par entrée et par exécution — le dédoublonnage par
 * défaut de l'import est un *skip* silencieux sur (date, montant, référence, compte), et une
 * entrée en double fausserait le compte sans erreur. Dates : une par jour depuis le 2025-01-01
 * (`dateOfEntry`). Montants : `1000 + i` CHF, au crédit (sens de la fixture).
 *
 * Une spec n'importe pas une autre spec : Playwright y enregistrerait ses tests une seconde fois.
 * D'où ce module.
 */

import { expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'..',
	'fixtures',
	'camt053_v04_minimal.xml',
);

/** La date `AAAA-MM-JJ` de l'entrée `i` : un jour par entrée depuis le 2025-01-01. */
export function dateOfEntry(i: number): string {
	return new Date(Date.UTC(2025, 0, 1 + i)).toISOString().slice(0, 10);
}

/**
 * Le relevé synthétique : l'entrée de la fixture répétée `n` fois, chacune unique.
 *
 * `tag` marque les références (`<tag>-<unique>-<i>`) et `name` les contreparties
 * (`<name> <i> <unique>`) : deux specs sur le même compte ne se dédoublonnent pas entre elles.
 */
export function syntheticCamtStatement(
	n: number,
	unique: string,
	{ tag = 'OLDER', name = 'Ancienne' }: { tag?: string; name?: string } = {},
): string {
	const xml = fs.readFileSync(FIXTURE, 'utf8');
	const debut = xml.indexOf('<Ntry>');
	const fin = xml.indexOf('</Ntry>') + '</Ntry>'.length;
	expect(debut > 0 && fin > debut, 'fixture : bloc <Ntry> introuvable').toBeTruthy();
	const modele = xml.slice(debut, fin);
	const entrees = Array.from({ length: n }, (_, i) => {
		let e = modele;
		for (const [from, to] of [
			['<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${(1000 + i).toFixed(2)}</Amt>`],
			['<Dt>2026-05-15</Dt>', `<Dt>${dateOfEntry(i)}</Dt>`],
			['<Dt>2026-05-15</Dt>', `<Dt>${dateOfEntry(i)}</Dt>`],
			['BANK-TX-42', `${tag}-${unique}-${i}`],
			['E2E-2026-05-001', `E2E-${tag}-${unique}-${i}`],
			['<Nm>Acme SA</Nm>', `<Nm>${name} ${i} ${unique}</Nm>`],
		]) {
			expect(e.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
			e = e.replace(from, to);
		}
		return e;
	});
	let out = xml.slice(0, debut) + entrees.join('\n') + xml.slice(fin);
	for (const [from, to] of [
		['STMT-2026-05-001', `STMT-${tag}-${unique}`],
		['<FrDtTm>2026-05-01T00:00:00</FrDtTm>', '<FrDtTm>2025-01-01T00:00:00</FrDtTm>'],
		['<Dt>2026-05-01</Dt>', '<Dt>2025-01-01</Dt>'],
	]) {
		expect(out.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
		out = out.replace(from, to);
	}
	return out;
}
