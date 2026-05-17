# Chunk 4 — Blind Hunter findings (Frontend Svelte 5 + Playwright)

**Revieweur** : Blind Hunter (adversarial, aucun contexte projet)
**Scope** : `exports.api.ts`, `exports.api.test.ts`, `+page.svelte`, `+page.ts`, `+layout.svelte` (nav), `export-global.spec.ts`
**Date** : 2026-05-17

---

## F01 — HIGH | Blob memory leak : `revokeObjectURL` synchrone avant fin de download

**Fichier** : `exports.api.ts:250-253`

```ts
} finally {
    if (a.parentNode) a.parentNode.removeChild(a);
    URL.revokeObjectURL(objectUrl);
}
```

`a.click()` est synchrone mais le navigateur démarre le téléchargement de façon **asynchrone** dans un microtask/event interne. Révoquer l'URL objet dans le même `finally` (exécuté sur le même tick synchrone, juste après le retour de `click()`) **annule potentiellement le download avant que le navigateur ait lu le blob**, notamment sur Firefox et Safari où le téléchargement de l'objet URL est initialisé de façon différée. Le pattern correct est de révoquer après un `setTimeout(() => URL.revokeObjectURL(objectUrl), 100)` minimal, ou mieux dans l'event `click` avec un délai via `requestAnimationFrame`. La duplication reconnue du pattern 9-2a (même bug signalé Pass 1 M11 de 9-2a apparemment non résolu dans l'abstraction) amplifie le risque.

---

## F02 — HIGH | RFC 5987 regex : capture groupe non ancrée, accepte des caractères interdits

**Fichier** : `exports.api.ts:205`

```ts
const rfc5987 = header.match(/filename\*\s*=\s*UTF-8'[^']*'([^;\r\n]+)/i);
```

Le groupe de capture `([^;\r\n]+)` accepte **n'importe quel caractère** sauf `;`, `\r`, `\n` — y compris des espaces, des guillemets, ou des caractères de contrôle. RFC 5987 §3.2 définit la `ext-value` comme `charset ' language ' value-chars` où `value-chars` est exclusivement `pct-encoded / attr-char`. La regex ne rejette pas les espaces non-encodés ni les guillemets doubles, ce qui peut donner un résultat incorrect si le serveur (ou un proxy) insère un espace avant `;` dans le header. Cas concret : `filename*=UTF-8''kesh export.zip` est accepté et renvoie `kesh export.zip` au lieu de null. Un trim seul (ligne 208) ne corrige pas ce cas structurel.

---

## F03 — HIGH | Race condition concurrence : `exporting` flag positionné **après** le guard, pas avant

**Fichier** : `+page.svelte:312-316`

```ts
async function startExport(): Promise<void> {
    if (exporting) return;      // guard
    exporting = true;           // ← set APRÈS le guard
```

En Svelte 5, `$state` mutations sont synchrones dans le même appel de fonction. Cependant, si deux appels à `startExport` arrivent depuis des sources d'événements différents (ex. double-tap mobile, event forwarding, test automation), le guard et la mutation ne sont pas atomiques du point de vue du scheduler d'événements. Le pattern correct est de tester-et-setter en une seule expression atomique ou d'utiliser un flag non-réactif (variable ordinaire) pour le guard, et de mettre `$state` uniquement pour l'affichage UI. En l'état, dans un environnement concurrent (tests Playwright, double-clic rapide < 16ms), deux coroutines peuvent passer le `if (exporting)` avant que l'une d'elles n'ait exécuté `exporting = true`.

---

## F04 — MEDIUM | Pas de `aria-busy` sur le bouton pendant l'export

**Fichier** : `+page.svelte:370-382`

```html
<button
    type="button"
    class="..."
    disabled={exporting}
    onclick={startExport}
    data-testid="export-global-start"
>
```

Lorsque `exporting = true`, le bouton est `disabled` mais ne porte ni `aria-busy="true"` ni `aria-label` dynamique décrivant l'état en cours. Un utilisateur daltonien/lecteur d'écran voit un bouton désactivé sans explication de la raison. WCAG 2.1 SC 4.1.3 (Status Messages) requiert que les mises à jour d'état soient annoncées. Le texte « Génération de l'export… » à l'intérieur du bouton n'est pas lu automatiquement par tous les screen readers quand le bouton devient `disabled`. Ajouter `aria-busy={exporting}` et éventuellement `aria-label` conditionnel.

---

## F05 — MEDIUM | `role="alert"` sur le message d'erreur déclenche une annonce immédiate même lors d'un clear-puis-set

**Fichier** : `+page.svelte:386-393`

```html
{#if errorMsg}
    <p role="alert" data-testid="export-global-error">
        {errorMsg}
    </p>
{/if}
```

Le pattern `{#if errorMsg}` + `role="alert"` crée et détruit le nœud DOM à chaque changement. Cela fonctionne correctement à l'insertion. Cependant, si `errorMsg` passe de `"Erreur A"` → `null` → `"Erreur B"` dans deux exports consécutifs, le nœud est recréé et l'annonce ARIA se déclenche bien. En revanche, si on écrit sur le même nœud sans le supprimer (ex. refactoring futur vers un `class:hidden`), l'annonce ne se répète pas. Ce n'est pas un bug actuel mais une fragilité architecturale : la sémantique d'accessibilité dépend implicitement du rendu conditionnel `{#if}`. Documenter cette invariante ou utiliser un wrapper `aria-live="assertive"` permanent avec contenu conditionnel.

---

## F06 — MEDIUM | Test Vitest : `revokeObjectURL` appelé **une seule fois** — mais ne vérifie pas le timing

**Fichier** : `exports.api.test.ts:120`

```ts
expect(revokeObjectURLSpy).toHaveBeenCalledOnce();
```

Le test confirme que `revokeObjectURL` est appelé, mais **pas quand** il est appelé par rapport à la résolution de la Promise. Si `triggerDownload` revient avant que le download soit terminé (cf. F01), ce test passe quand même car il ne fait aucune assertion sur le timing. Ce test valide une implémentation cassée autant qu'une correcte.

---

## F07 — MEDIUM | RFC 5987 regex : flag `i` (case-insensitive) sur `UTF-8` ouvre des variantes non-standard

**Fichier** : `exports.api.ts:205`

```ts
const rfc5987 = header.match(/filename\*\s*=\s*UTF-8'[^']*'([^;\r\n]+)/i);
```

Le flag `/i` s'applique à toute la regex, y compris `UTF-8`. RFC 5987 §3.2 spécifie que le charset est case-insensitive, donc `utf-8` est valide. Cependant, cela accepte aussi `Utf-8`, `UTF8`, `UTF-8` — les deux premiers sont dans le standard, mais `UTF8` (sans tiret) ne l'est pas. Or la regex `/UTF-8/i` accepterait `utf8` (sans tiret) parce que le `-` n'est pas un métacaractère ici — **si** on tente `utf8`, le tiret est littéral donc `utf8` ne matche pas. Ce point est donc un faux positif ; mais en revanche `UTF8''` matcherait `UTF-8''` si quelqu'un retire le tiret, ce qui n'est pas le cas. Risque mineur mais la regex devrait être `(?:UTF-8|utf-8)/i` limité au charset ou utiliser `[Uu][Tt][Ff]-8` pour être explicite. Sévérité abaissée à MEDIUM (comportement correct en pratique pour le cas nominal).

---

## F08 — MEDIUM | Playwright : assertion `disabled` avec `catch` vide masque un flake réel

**Fichier** : `export-global.spec.ts:492-494`

```ts
await expect(startButton).toBeDisabled({ timeout: 1000 }).catch(() => {
    // Backend très rapide — le bouton est déjà revenu enabled.
});
```

Ce pattern transforme un échec de test en succès silencieux. Si le bouton **n'est jamais passé en disabled** à cause d'un bug (ex. le guard `if (exporting) return` n'a pas été déclenché), le test passe quand même. Un test qui ne peut pas échouer n'est pas un test. Option correcte : utiliser `toBeDisabled({ timeout: 200 })` dans un bloc qui log l'absence d'état disabled comme warning, mais ne pas `catch` silencieusement. Si l'AC #25 est important (ce que le story file prétend), il mérite une assertion non-catchée ou une architecture différente (mock du backend avec un délai artificiel pour rendre l'état observable).

---

## F09 — MEDIUM | `parseContentDispositionFilename` : regex RFC 6266 unquoted accepte `filename*=` comme fallback

**Fichier** : `exports.api.ts:222-225`

```ts
const rfc6266Unq = header.match(/filename\s*=\s*([^;\s]+)/i);
```

Cette regex matche aussi `filename*=UTF-8''...` — le `*` n'est pas dans le pattern de gauche (`filename\s*=`) donc si on arrive ici (rfc5987 a échoué), la regex `filename\s*=\s*([^;\s]+)` peut matcher `filename*=UTF-8''kesh-export.zip` et retourner `UTF-8''kesh-export.zip` comme nom de fichier. Cas de reproduction : header malformé `filename*=invalid-encoding''kesh.zip` → rfc5987 échoue (decodeURIComponent throw) → fallback rfc6266 (quoted) ne matche pas → rfc6266Unq matche et retourne `invalid-encoding''kesh.zip`. La regex devrait être `filename(?!\*)\s*=\s*([^;\s]+)/i` avec un lookahead négatif pour exclure `filename*`.

---

## F10 — MEDIUM | Test mock `Headers` : casing du header `Content-Disposition` non testé

**Fichier** : `exports.api.test.ts:105-109`

```ts
headers: new Headers({
    'content-type': 'application/zip',
    'content-disposition': '...',
}),
```

Les tests utilisent exclusivement la clé `content-disposition` (minuscules). La fonction `downloadGlobalExport` appelle `response.headers.get('Content-Disposition')` (capitale C, capitale D). La classe `Headers` du browser (et de `undici`/Node fetch) est case-insensitive, donc ce test passe. Mais **l'API** (`apiClient.getBlob`) retourne peut-être un objet custom qui ne normalise pas les headers. Si jamais `apiClient` wrappait la réponse avec un objet non-`Headers`, le test ne détecterait pas le bug. Il manque un cas de test avec capitales différentes pour garantir la robustesse du parser contre le type concret retourné par `apiClient`.

---

## F11 — LOW | `+page.svelte` : pas de `aria-live` permanent pour `successMsg`

**Fichier** : `+page.svelte:395-403`

```html
{#if successMsg}
    <p role="status" data-testid="export-global-success">
        {successMsg}
    </p>
{/if}
```

`role="status"` implique `aria-live="polite"`. La création du nœud via `{#if}` fonctionne sur Chrome/Firefox mais est moins fiable sur Safari VoiceOver (le nœud live region doit être dans le DOM **avant** que le contenu soit inséré pour être annoncé de façon fiable). Ce pattern est identique à celui utilisé dans 9-2a (dette connue), mais mérite un signalement low pour cohérence.

---

## F12 — LOW | Playwright : `seedTestState` appelé dans `beforeAll` — pas de teardown après le test

**Fichier** : `export-global.spec.ts:458-460`

```ts
test.beforeAll(async () => {
    await seedTestState('with-company');
});
```

Il n'y a pas de `afterAll` pour nettoyer l'état seeded, ni de vérification que le seed `with-company` ne pollue pas d'autres suites de tests si elles s'exécutent dans le même worker Playwright. Si d'autres spec files supposent un état propre, le `beforeAll` sans `afterAll` crée une dépendance d'ordre non déclarée. Pattern cohérent avec 9-2a (dette connue) mais à documenter.

---

## F13 — LOW | `export-global.spec.ts` : hardcoded `/tmp/kesh-test-9-2b.zip` — collision si tests parallèles

**Fichier** : `export-global.spec.ts:499`

```ts
const savedPath = '/tmp/kesh-test-9-2b.zip';
```

Si Playwright exécute les workers en parallèle (ce qui est le cas par défaut avec plusieurs spec files), deux instances du même test pourraient écrire sur le même chemin. `tmpdir()` + nom unique (via `test.info().testId` ou `crypto.randomUUID()`) serait plus robuste.

---

## Récapitulatif par sévérité

| # | Sévérité | Titre court |
|---|----------|-------------|
| F01 | HIGH | Blob revoke synchrone — download peut être annulé |
| F02 | HIGH | RFC 5987 regex groupe trop large — caractères interdits acceptés |
| F03 | HIGH | Race guard/set `exporting` — non atomique |
| F04 | MEDIUM | Bouton manque `aria-busy` pendant export |
| F05 | MEDIUM | `role="alert"` fragilité ARIA sur re-render |
| F06 | MEDIUM | Test Vitest ne valide pas le timing de `revokeObjectURL` |
| F07 | MEDIUM | Flag `/i` trop large sur regex RFC 5987 |
| F08 | MEDIUM | `catch` vide sur assertion `disabled` — flake masqué |
| F09 | MEDIUM | Regex rfc6266Unq matche `filename*=` comme fallback |
| F10 | MEDIUM | Mock headers sans test casing `Content-Disposition` |
| F11 | LOW | `role="status"` Safari VoiceOver fiabilité |
| F12 | LOW | `beforeAll` seed sans `afterAll` teardown |
| F13 | LOW | Chemin `/tmp` hardcodé — collision parallèle |
