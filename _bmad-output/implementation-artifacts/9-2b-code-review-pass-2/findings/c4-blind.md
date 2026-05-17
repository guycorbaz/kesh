# Pass 2 Blind Hunter — Chunk 4 Frontend (Story 9-2b)

**Reviewer**: Haiku 4.5 (Blind)  
**Date**: 2026-05-17  
**Diff**: `/home/gcorbaz/Synology/devel/kesh/frontend/src/lib/features/export/*`

---

## F01 · CRITICAL — H6 Lookahead `(?!\*)` Syntaxe Correcte Mais Ordre Fragile

**Fichier**: `exports.api.ts:294`  
**Snippet**:
```typescript
const rfc6266Unq = header.match(/filename(?!\*)\s*=\s*([^;\s]+)/i);
```

**Problème**: Le lookahead `(?!\*)` est syntaxiquement valide en JavaScript. **Mais** il teste le caractère **immédiatement après** `filename` — ce qui suppose que le token suit sans espace. La séquence réelle est `filename*=`, où le `\*` est le premier caractère post-token.

**Cependant**, la regex affiche `\s*=` ensuite, qui signifie "zéro ou plus espaces, puis `=`". Donc le lookahead regarde : après le mot `filename`, est-ce que le prochain caractère est `*` ? Si oui, rejette. Sinon, continue vers `\s*=`.

✗ **Régression**: Header `filename * = value` (espace avant `*`) passerait le lookahead car le premier caractère post-`filename` est un espace, pas `*`. Puis `\s*=` matcherait "espace-étoile-espace-égal", et capterait `value` — **faux positif**, devrait retourner `null` car c'est malformé.

✓ **Fix vérifiable** : Le lookahead doit tester `(?!\*|[\s]*\*)` ou placer `(?!\*)` **après** le `\s*`, i.e., `/filename\s*(?!\*)\s*=\s*([^;\s]+)/i`. Actuellement le test pass ne le couvre pas.

**Sévérité**: **CRITICAL**  
**AC touché**: AC #31(e) — null sur RFC 5987 valeur vide (`filename*=UTF-8''`), mais pas cas espace-malformé.

---

## F02 · HIGH — H7 Cleanup Variable `savedZipPath` Race Condition Tests Parallèles

**Fichier**: `export-global.spec.ts:538, 540-547`  
**Snippet**:
```typescript
let savedZipPath: string | null = null;

test.afterEach(async ({ page }) => {
	if (savedZipPath && fs.existsSync(savedZipPath)) {
		fs.unlinkSync(savedZipPath);
	}
	savedZipPath = null;
	await clearAuthStorage(page);
});
```

**Problème**: `savedZipPath` est une variable **module-scoped** (ligne 538), pas test-isolée. Playwright exécute plusieurs tests en parallèle par défaut. Si deux tests Playwright tournent en concurrence :

- Test A : ligne 590 écrit `savedZipPath = path.join(..., `9-2b-${Date.now()}.zip`)`
- Test B : concurrence, ligne 590 écrit `savedZipPath = path.join(..., `9-2b-${Date.now()}.zip`)` (même `Date.now()` si < 1ms)
- Test A : `afterEach` ligne 542 unlink le chemin de Test B → le fichier de Test B disparaît avant son `afterEach` 
- Test B : `afterEach` tente `unlink` un fichier déjà supprimé → `ENOENT` silencieuse mais stat fragile

✗ **Régression du patch H7** : Le commentaire Pass 1 dit "variable partagée pour cleanup robuste même si test fail" (ligne 536-537), mais ne documente pas l'isolation test. Le cleanup n'est robuste que **si tests tournent séquentiels**.

**Fix**: Déplacer la variable en scope local `test`, ou lancer les tests avec `test.describe.sequential()`, ou utiliser `test.afterEach(({ task })` pour isoler par task ID.

**Sévérité**: **HIGH**  
**AC touché**: AC #32 cleanup (pas de fuite fichier).

---

## F03 · HIGH — H8 Test Promise Délayée Ne Court-Circuite Pas le Second Appel

**Fichier**: `exports.api.test.ts:159-208`  
**Snippet**:
```typescript
let resolveDownload!: () => void;
const slowFetch = vi.fn().mockImplementation(
	() =>
		new Promise<Response>((resolve) => {
			resolveDownload = () => {
				const zipBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x00, 0x00]);
				const mockBlob = new Blob([zipBytes], { type: 'application/zip' });
				resolve({
					ok: true,
					status: 200,
					blob: () => Promise.resolve(mockBlob),
					headers: new Headers({
						'content-type': 'application/zip',
						'content-disposition':
							'attachment; filename="kesh-export-test.zip"',
					}),
				} as unknown as Response);
			};
		}),
);
// ...
const p1 = startExport();
const p2 = startExport();
resolveDownload();
await Promise.all([p1, p2]);

expect(calls).toBe(1);
expect(slowFetch).toHaveBeenCalledTimes(1);
```

**Problème**: Le test simule un guard re-entrancy via variable locale `exporting` (ligne 185-196). Mais la logique teste le **même pattern dans `+page.svelte`** — donc ce test est une duplication tautologique de la implémentation, pas une test du comportement de la **vraie fonction** `downloadGlobalExport()`.

Plus grave : la Promise `slowFetch()` **n'est jamais résolue** tant que `resolveDownload()` n'est pas appelé (ligne 202). Mais `downloadGlobalExport()` ligne 192 await cet appel. Donc :
- `p1 = startExport()` → calls le guard `if (exporting) return` → passe (exporting=false)
- Définit `exporting = true`
- Appelle `downloadGlobalExport()` → await `apiClient.getBlob()` → await `slowFetch()` → **bloque sur Promise non-résolue**
- `p2 = startExport()` → test le guard `if (exporting) return` → **early return vrai, p2 = Promise.resolve()**
- Ligne 202 : `resolveDownload()` → débloque p1
- `await Promise.all([p1, p2])` → resolvé

✗ **Mais le test ne mesure que le guard local**, pas `downloadGlobalExport` lui-même. Si le backend était lent IRL, `downloadGlobalExport` **n'a aucun mécanisme interne** de guard — le caller (`+page.svelte`) gère tout. Le test valide le **caller**, pas l'API.

**Sévérité**: **HIGH** (test faux positif — passe mais valide rien de robuste)  
**AC touché**: AC #31(c) — guard re-entrancy testé, mais au mauvais niveau.

---

## F04 · MEDIUM — M7 Playwright `page.route` Délai 500ms Peut Causer Timeout Flaky

**Fichier**: `export-global.spec.ts:562-565`  
**Snippet**:
```typescript
await page.route('**/api/v1/exports/global.zip', async (route) => {
	await new Promise((r) => setTimeout(r, 500));
	await route.continue();
});
```

**Problème**: Le test ajoute 500ms à chaque requête backend. Puis ligne 582 : `await expect(startButton).toBeDisabled({ timeout: 2000 })` observe le disabled. Mais :

1. Clic le bouton (ligne 578) → `startExport()` appelle `downloadGlobalExport()`
2. Route interceptée → délai 500ms
3. Assertion ligne 582 : attend disabled **dans 2000ms**

Problème : le délai route (500ms) s'ajoute au temps pour atteindre l'assertion. Si la navigation SvelteKit ou le re-render du composant prend >1500ms, l'assertion timeout malgré `timeout: 2000`. En CI avec chargement élevé, ça arrive.

✓ **Observation** : le commentaire ligne 558-561 dit "rendant l'état disabled observable de manière fiable" — l'intention est juste. Mais le délai fixe (500ms) est fragile.

**Sévérité**: **MEDIUM** (flaky potentiel en CI)  
**AC touché**: AC #25 — observable disabled state.

---

## F05 · MEDIUM — H6 Regex RFC 6266 Unquoted Accepte `filename=UTF-8''` Contre Spec

**Fichier**: `exports.api.ts:288-297`  
**Snippet**:
```typescript
// RFC 6266 unquoted : `filename=…` (rare, accepté défensivement).
const rfc6266Unq = header.match(/filename(?!\*)\s*=\s*([^;\s]+)/i);
if (rfc6266Unq && rfc6266Unq[1]) {
	return rfc6266Unq[1];
}
```

**Problème**: RFC 6266 unquoted est `filename=token`, où `token` est ASCII alphanumérique sans `*`. Mais la regex capture `([^;\s]+)`, i.e., "un ou plus non-semicolon non-whitespace". Cela inclurait `UTF-8''` (valide capture par `[^;\s]+`).

Header malformé `filename=UTF-8''` (RFC 5987 mal écrit sans l'astérisque) serait capturé comme unquoted filename `UTF-8''`, ce qui n'est pas un nom de fichier valide.

**Test couvre AC #31(e)** (line 72-76) : `header="attachment; filename*=UTF-8''"` → lookahead `(?!\*)` **devrait** rejeter. Mais test ne couvre pas `header="attachment; filename=UTF-8''"` (sans `*`, typo malformée).

✗ **Régression potentielle**: Si backend malformerait l'en-tête RFC 5987 en `filename=UTF-8''` au lieu de `filename*=`, le frontend retournerait `UTF-8''` comme nom de fichier fallback.

**Sévérité**: **MEDIUM** (dépend erreur backend)  
**AC touché**: AC #31(e).

---

## F06 · MEDIUM — H7 Playwright Cleanup N'Isole Pas les Runs Parallèles par Path

**Fichier**: `export-global.spec.ts:590`  
**Snippet**:
```typescript
savedZipPath = path.join(os.tmpdir(), `kesh-test-9-2b-${Date.now()}.zip`);
await download.saveAs(savedZipPath);
```

**Problème**: `Date.now()` a résolution **milliseconde**. Si deux tests lancés dans le même milliseconde (possible en CI avec machine multi-core), `Date.now()` peut retourner la même valeur. Filename clash → deux tests écrivent/lisent le même chemin.

**Fix**: Utiliser `Math.random()` ou UUID, ou `process.pid + Date.now()`, ou l'ID du test Playwright.

**Sévérité**: **MEDIUM** (très rare mais possible)  
**AC touché**: AC #32 cleanup.

---

## F07 · LOW — H8 Test Vitest Mock Promise Délayée Est Style Overkill

**Fichier**: `exports.api.test.ts:162-180`  
**Snippet**:
```typescript
let resolveDownload!: () => void;
const slowFetch = vi.fn().mockImplementation(
	() =>
		new Promise<Response>((resolve) => {
			resolveDownload = () => {
				// ...
			};
		}),
);
```

**Problème**: Le test crée une Promise qui ne se résout jamais jusqu'à appel manuel de `resolveDownload()`. C'est un pattern valide (Promise délayée manuelle), mais Vitest expose `vi.waitFor()` et `vi.useRealTimers()` plus idiomatiques. Le code marche mais est moins lisible.

**Sévérité**: **LOW** (style/lisibilité)  
**AC touché**: AC #31(c).

---

## F08 · LOW — M7 Playwright Timeout 2000ms vs Délai 500ms Ratio Élevé

**Fichier**: `export-global.spec.ts:582-583`  
**Snippet**:
```typescript
await expect(startButton).toBeDisabled({ timeout: 2000 });
await expect(startButton).toContainText(/G[ée]n[ée]ration/i, { timeout: 2000 });
```

**Problème**: Délai route 500ms + timeout assertion 2000ms = ratio 4:1. Idéalement, timeout = délai + overhead (ex. 500 + 300 = 800ms). Ratio 2000ms donne trop de marge, masque les vraies lenteurs.

**Sévérité**: **LOW** (tuning perf)  
**AC touché**: AC #25.

---

## Résumé Findings

| ID | Sévérité | Fichier | Ligne | Titre |
|----|---------:|---------|-------|-------|
| F01 | CRITICAL | exports.api.ts | 294 | Lookahead `(?!\*)` fragile sur espace-malformé |
| F02 | HIGH | export-global.spec.ts | 538-547 | Variable module `savedZipPath` race condition |
| F03 | HIGH | exports.api.test.ts | 159-208 | Test Promise tautologie du guard caller |
| F04 | MEDIUM | export-global.spec.ts | 562-565 | Délai 500ms flaky en CI haute charge |
| F05 | MEDIUM | exports.api.ts | 288-297 | Regex unquoted accepte `UTF-8''` malformé |
| F06 | MEDIUM | export-global.spec.ts | 590 | Collision `Date.now()` path parallèle |
| F07 | LOW | exports.api.test.ts | 162-180 | Promise délayée style overkill |
| F08 | LOW | export-global.spec.ts | 582-583 | Timeout 2000ms vs délai 500ms ratio élevé |

**Findings > LOW**: 5 (F01 + F02 + F03 + F04 + F05 + F06)
