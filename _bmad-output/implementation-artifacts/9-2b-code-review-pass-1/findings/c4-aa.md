# Acceptance Audit — Chunk 4 Frontend (Story 9-2b)

**Rôle** : Acceptance Auditor  
**Périmètre** : Chunk 4 = frontend Story 9-2b (export global ZIP)  
**Diff analysé** : `chunk-4-frontend.diff`  
**Spec** : `9-2b-export-global-zip.md`  
**Date** : 2026-05-17  

---

## Résumé exécutif

| Sévérité | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 1 |
| LOW | 2 |
| PASS | 9 ACs vérifiés conformes |

---

## Findings

### AA-HIGH-01 — AC #31(c) manquant : test guard re-entrancy double-clic absent

**AC violée** : AC #31 — ≥ 5 tests Vitest couvrant (a) à (e) ; (c) = guard re-entrancy double-clic obligatoire  
**Fichier** : `frontend/src/lib/features/export/exports.api.test.ts` (ensemble du fichier)  
**Description** :  
La spec AC #31(c) exige explicitement un test Vitest vérifiant le guard re-entrancy : mock `downloadGlobalExport` slow (delayed Promise) → appeler `startExport()` deux fois en succession → assert que `downloadGlobalExport` n'est appelé qu'une seule fois. Le diff contient 7 `it()` couvrant les tests (a), (b), (d), (e) — dont 3 variantes pour (e) — mais aucun test pour (c).

Le commentaire T12.1(c) dans le fichier de test signale explicitement : « testé au niveau caller ; le guard est dans `+page.svelte` ». Mais aucun test dans `+page.svelte` ne couvre ce comportement non plus (ni via `@testing-library/svelte` ni via Playwright). La spec autorise « si le test du composant Svelte est complexe : alternative — extraire la logique de guard en helper testable » mais cette alternative n'est pas non plus implémentée.

L'AC #31(c) est marqué Pass 1 AA-HIGH-05 dans la spec, sévérité originellement HIGH. Il manque complètement dans le diff.

**Poids** : HIGH — AC spécifique, test marqué HIGH dans la spec validate, comportement crítico-fonctionnel (double-clic submit pouvant déclencher deux exports côté serveur).

---

### AA-MEDIUM-01 — AC #32 : assertion disabled + libellé "Génération…" non-bloquante dans Playwright

**AC violée** : AC #32 — scénario Playwright doit asserter bouton `disabled` avec libellé `Génération de l'export…` (AC #25 état UX)  
**Fichier** : `frontend/tests/e2e/export-global.spec.ts`, ligne 492  
**Description** :  
La spec AC #32 inclut explicitement : « assert bouton `disabled` avec libellé `Génération de l'export…` (Pass 1 AA-MEDIUM-07 — AC #25 état UX) ». L'implémentation utilise :

```typescript
await expect(startButton).toBeDisabled({ timeout: 1000 }).catch(() => {
    // Backend très rapide — le bouton est déjà revenu enabled.
});
```

Deux problèmes :
1. L'assertion `.catch(() => {})` vide rend la vérification `disabled` **totalement non-bloquante** — le test passe même si le bouton n'est jamais `disabled`. Ce n'est plus une assertion, c'est un no-op conditionnel.
2. Le libellé `Génération de l'export…` (`export-global-loading`) n'est **jamais asserté** dans le test Playwright — ni via `toHaveText()`, ni via `textContent`.

AC #25 + AC #32 requièrent que cet état soit validé. Le test peut théoriquement être vert sans que le bouton soit jamais passé en `disabled`.

**Note** : L'argument "backend très rapide" est légitime en CI, mais la solution correcte est de mocker la réponse avec un délai ou d'utiliser `page.route()` pour intercepter la requête — pas de silencer l'assertion entièrement.

**Poids** : MEDIUM — l'AC #25 état UX est un critère d'acceptance explicite de la story, non couvert par le test E2E.

---

### AA-LOW-01 — Déviation T8.4 : nav-export-global dans le groupe existant plutôt que nouveau groupe séparé

**AC violée** : T8.4 (prescriptif spec) — impact UX mineur sur AC #1  
**Fichier** : `frontend/src/routes/(app)/+layout.svelte`, lignes 264-267  
**Description** :  
La spec T8.4 spécifie de créer un **nouveau groupe** `{ label: null, items: [{ i18nKey: 'nav-export-global', ... }] }` séparé, à insérer **avant** le groupe contenant `nav-settings`. L'implémentation a au contraire ajouté `nav-export-global` comme premier item **dans le groupe existant** qui contient déjà `nav-settings`.

Résultat fonctionnel : l'entrée apparaît bien avant `nav-settings` dans la sidebar — AC #1 est donc satisfait fonctionnellement. La déviation est purement structurelle (1 groupe modifié vs 1 groupe ajouté + 1 groupe existant inchangé). La spec autorise explicitement cette flexibilité : « Si Sally recommande un groupe nommé `Souveraineté`... refactor low-risk post-merge, ne pas bloquer dev-story. »

**Poids** : LOW — l'intent UX de l'AC #1 est respecté. Nit structurel.

---

### AA-LOW-02 — AC #27 : condition `isApiError(e)` sans vérification `e.code` dans formatError

**AC violée** : AC #27 — pattern `isApiError(e) && e.code` → `formatError(e)` spécifié  
**Fichier** : `frontend/src/routes/(app)/export/+page.svelte`, ligne 303  
**Description** :  
La spec AC #27 et T8.2 prescrivent le pattern :
```typescript
if (isApiError(e) && e.code) {
    errorMsg = formatError(e);
} else {
    errorMsg = i18nMsg('export-global-error-generic', '...');
}
```

L'implémentation utilise :
```typescript
function formatError(err: unknown): string {
    if (isApiError(err)) {
        return err.message;
    }
    return i18nMsg('export-global-error-generic', "...");
}
```
puis `catch (e) { errorMsg = formatError(e); }` — i.e. `isApiError(err)` sans `&& e.code`. 

En pratique, si `isApiError(err)` est vrai, `err.message` et `err.code` sont tous deux définis dans `ApiError` — la différence de comportement observable est nulle. Cependant, le pseudocode de la spec incluait la garde `e.code` pour se protéger d'une `ApiError` avec `code = ""` ou `code = undefined` (cas théorique). C'est un nit cosmétique sans impact fonctionnel connu.

**Poids** : LOW — nit de conformité au pseudocode spec, aucun impact comportemental observé.

---

## ACs vérifiés conformes

| AC | Description | Statut |
|----|-------------|--------|
| AC #1 | Entrée `nav-export-global` dans menu principal AVANT `nav-settings` | PASS |
| AC #2 | Page `/export` : titre + description + bouton + zone alerte | PASS |
| AC #25 | Bouton `disabled` + libellé `Génération de l'export…` pendant export (implémentation) | PASS (impl) |
| AC #26 | Guard re-entrancy `if (exporting) return` première ligne | PASS |
| AC #27 | Erreur via `isApiError` + fallback i18n `export-global-error-generic` | PASS (nit LOW-02) |
| AC #28 | Filename = serveur via `Content-Disposition`, `parseContentDispositionFilename` | PASS |
| AC #31(a) | Vitest : fetch URL + Blob + download déclenché | PASS |
| AC #31(b) | Vitest : erreur 500 propagée | PASS |
| AC #31(d) | Vitest : `parseContentDispositionFilename` ASCII fallback | PASS |
| AC #31(e) | Vitest : RFC 5987 UTF-8 percent-decoded + null + empty | PASS |
| AC #32 (partiel) | Playwright : login → `/export` → bouton visible → download → bytes ZIP → filename regex → bouton re-enabled | PASS (partiel, voir AA-MEDIUM-01) |
| T11.3 | Feature folder `export/` singular (PAS `exports/` plural) | PASS |
| §triggerDownload-reuse | Duplication locale acceptée (Decision Pass 1 BH-MEDIUM-02) | PASS |
| formatError local | Duplication locale acceptée (Pass 1 BH-MEDIUM-03) | PASS |
| parseContentDispositionFilename | Nouvelle fn locale, non importée de 9-2a (Pass 3 ECH3-H4) | PASS |

---

## Actions recommandées

1. **AA-HIGH-01** (bloquant) : Implémenter le test AC #31(c) guard re-entrancy. Option recommandée : mock `downloadGlobalExport` avec un `Promise` retardé via `vi.fn().mockImplementation(() => new Promise(resolve => setTimeout(resolve, 100)))`, puis dans le test appeler deux fois consécutivement la logique de guard (soit via `@testing-library/svelte` sur `+page.svelte`, soit en extrayant le guard en helper `export function withReentrancyGuard(fn, flagRef)` testable isolément).

2. **AA-MEDIUM-01** (recommandé) : Remplacer le `.catch(() => {})` non-bloquant par un vrai mock réseau via `page.route('/api/v1/exports/global.zip', route => { /* slow response */ })` ou accepter une assertion plus souple comme `page.waitForLoadState('networkidle')` puis vérifier séquentiellement, en garantissant quand même que le libellé `Génération de l'export…` est asserté à un moment du scénario.
