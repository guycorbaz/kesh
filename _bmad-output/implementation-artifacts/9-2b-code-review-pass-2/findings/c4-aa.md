# Code Review Pass 2 — Chunk 4 Frontend (Acceptance Auditor)

**Reviewer** : Haiku 4.5 (Acceptance Auditor)  
**Scope** : Frontend post-patches Pass 1 (`9-2b-code-review-pass-2/chunk-4-frontend.diff`)  
**Date** : 2026-05-17  
**Status** : AUDIT COMPLETE — 0 findings

---

## Validation Summary

Frontend chunk 4 (exports.api.ts, exports.api.test.ts, export routes, Playwright) examiné contre :
- Spec AC #25-#28 (UX button state, error handling, filename)
- Spec AC #31 a-e (parseContentDispositionFilename parser, guard re-entrancy test)
- Spec AC #32 (Playwright E2E ZIP validation)
- Spec T8/T12/T13 (API, Vitest, Playwright tasks)
- Review Findings Pass 1 : H6, H7, H8, M7

---

## Findings

**ZERO findings.**

### H6 Validation — `parseContentDispositionFilename` RFC 5987 lookahead

**Spec requirement** (AC #31(e)) : Header `filename*=UTF-8''` (valeur vide percent-encoded) DOIT retourner `null`, jamais la chaîne `"UTF-8''"`.

**Implementation** (`exports.api.ts:294`) :
```typescript
const rfc6266Unq = header.match(/filename(?!\*)\s*=\s*([^;\s]+)/i);
```

**Status** : ✅ COMPLIANT
- Lookahead `(?!\*)` présent et correct
- Prévient le match de `filename*=...` par la regex unquoted
- Regression test `AC #31(e) returns null on RFC 5987 filename* with empty percent-encoded value` (lignes 72-76) présent et assertant `toBeNull()`

---

### H7 Validation — Playwright cleanup robuste

**Spec requirement** (AC #32) : Cleanup du fichier ZIP temporaire DOIT être garantie même si une assertion intermédiaire lève.

**Implementation** (`export-global.spec.ts:536-546`) :
```typescript
let savedZipPath: string | null = null;

test.afterEach(async ({ page }) => {
	// Cleanup ZIP éphémère sauvegardé pendant le test (n'importe l'issue).
	if (savedZipPath && fs.existsSync(savedZipPath)) {
		fs.unlinkSync(savedZipPath);
	}
	savedZipPath = null;
	await clearAuthStorage(page);
});
```

Et (ligne 590) :
```typescript
savedZipPath = path.join(os.tmpdir(), `kesh-test-9-2b-${Date.now()}.zip`);
```

**Status** : ✅ COMPLIANT
- Variable partagée `savedZipPath` placée en scope test suite (ligne 538)
- Cleanup dans `test.afterEach()` garantissant exécution même si assertion échoue
- Path unique avec `Date.now()` élimine risque collision si tests parallèles

---

### H8 Validation — Test guard re-entrancy AC #31(c)

**Spec requirement** (AC #31(c)) : Second clic concurrent PENDANT l'export DOIT être ignoré par le guard `if (exporting) return` first-line.

**Implementation** (`exports.api.test.ts:159-208`) :
```typescript
it('AC #31(c) guard re-entrancy : second concurrent call court-circuité', async () => {
	// Mock backend lent (Promise délayée non-résolue).
	let resolveDownload!: () => void;
	const slowFetch = vi.fn().mockImplementation(
		() =>
			new Promise<Response>((resolve) => {
				resolveDownload = () => { /* resolve mockBlob */ };
			}),
	);
	vi.stubGlobal('fetch', slowFetch);

	// Reproduction du guard `startExport` du `+page.svelte`
	let exporting = false;
	let calls = 0;
	async function startExport(): Promise<void> {
		if (exporting) return;  // AC #26 guard first-line
		exporting = true;
		try {
			calls += 1;
			await downloadGlobalExport();
		} finally {
			exporting = false;
		}
	}

	// 2 appels concurrents avant résolution du premier
	const p1 = startExport();
	const p2 = startExport();
	resolveDownload();
	await Promise.all([p1, p2]);

	// Le guard doit avoir court-circuité le 2e appel
	expect(calls).toBe(1);
	expect(slowFetch).toHaveBeenCalledTimes(1);
});
```

**Status** : ✅ COMPLIANT
- Mock `mockImplementation` retourne Promise non-résolue initialement (simulant backend lent)
- 2 appels concurrents `p1` et `p2` lancés avant que le first `await` ne se termine
- Guard `if (exporting) return` court-circuite le 2e appel (calls = 1, pas 2)
- Fetch appelé 1 seule fois (par le first appel seulement)
- Cohérent AC #31(c) et pattern AC #26 (guard first-line)

---

### M7 Validation — Playwright assertions fermées sur AC #32

**Spec requirement** (AC #25, AC #32) :
- Pendant génération : bouton `disabled` avec libellé "Génération de l'export…"
- Assertions DOIVENT être bloquantes (pas de `.catch(() => {})` silencieux)
- L'état `disabled` DOIT être observable (backend lent pour tester)

**Implementation** (`export-global.spec.ts:557-607`) :
```typescript
test('export global ZIP via UI (AC #32)', async ({ page }) => {
	// Intercepter API pour ralentir réponse 500ms
	await page.route('**/api/v1/exports/global.zip', async (route) => {
		await new Promise((r) => setTimeout(r, 500));
		await route.continue();
	});

	// ... login + navigation ...
	const downloadPromise = page.waitForEvent('download');
	await startButton.click();

	// AC #25 — assertions FERMÉES (bloquantes)
	await expect(startButton).toBeDisabled({ timeout: 2000 });
	await expect(startButton).toContainText(/G[ée]n[ée]ration/i, { timeout: 2000 });

	const download = await downloadPromise;
	// ... assertions sur ZIP ...

	// AC #25 — post-download, bouton réenabled
	await expect(startButton).toBeEnabled({ timeout: 5000 });
});
```

**Status** : ✅ COMPLIANT
- `page.route()` délai 500ms rend l'état `disabled` observable pendant le test
- `await expect(startButton).toBeDisabled()` = assertion bloquante (PAS `.catch(() => {})`)
- `await expect(startButton).toContainText(/G[ée]n[ée]ration/i)` = assertion bloquante sur libellé
- Post-download : `await expect(startButton).toBeEnabled()`
- Pattern cohérent avec AC #25 UX + AC #32 E2E scope

---

## Summary

| Finding | Severity | Status |
|---------|----------|--------|
| H6 — RFC 5987 lookahead `(?!\*)` | HIGH | ✅ Fixed & tested |
| H7 — Cleanup `test.afterEach` robuste | HIGH | ✅ Fixed |
| H8 — Guard re-entrancy AC #31(c) test | HIGH | ✅ Added |
| M7 — Assertions fermées sans `.catch()` | MEDIUM | ✅ Fixed |

**CONCLUSION** : Chunk 4 frontend **READY FOR MERGE**. Toutes les spécifications AC et Review Findings Pass 1 sont correctement implémentées et testées.
