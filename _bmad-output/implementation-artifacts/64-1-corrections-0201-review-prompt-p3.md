# Prompt — revue de code P3 de la story `64-1-corrections-0201`, passe CIBLÉE (Haiku)

Tu conduis, **en lecture seule**, une passe **ciblée** : une seule lentille, sur **le seul commit `db9f5c37c`** (lot des LOW de code de
la revue P2 : L2-1 et L2-4) de la story `64-1-corrections-0201` de Kesh (comptabilité suisse en production ; SvelteKit 5 runes).

- **Modèle : Haiku.** Contexte frais. Langue : français. Les P1 (Opus) et P2 (Sonnet) étaient des passes complètes.
- Worktree `kesh-64-1`, branche `story/64-1-corrections-0201`. **Donne-toi un diff unique aplati** : `git diff e2c287b3e db9f5c37c`
  (ne lis pas les commits un à un). Rapport de la P2 : `$GATE_LOGS/e64/64-1-revue-p2.md` (L2-1 à L2-4).
- Lis la section « Revue de code P2 — lot des LOW de code » de la fiche `_bmad-output/implementation-artifacts/64-1-corrections-0201.md`.

## La lentille

**Un seul représentant par valeur de taux, indépendant de l'ordre, sans rien perdre de ce que l'écran affichait ou envoyait.**

Fichiers du commit : `frontend/src/lib/features/vat-rates/vat-rates.distinct.ts` (+ `.test.ts`), `frontend/src/routes/(app)/products/+page.svelte`,
`frontend/src/lib/components/invoices/InvoiceForm.svelte`, `frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte`, et trois fichiers de test.

Questions :

1. `distinctVatRates` : le représentant est-il le même quel que soit l'ordre d'entrée (libellé propre, puis catégorie, puis `id`) ? Les
   entrées sans `id`, `category` ou `label` (les mocks des tests ne portent que `rate`) sont-elles tolérées ? L'ordre de sortie
   (première apparition d'une valeur) est-il celui que les trois écrans attendent ?
2. Catalogue : le libellé de l'option vient-il bien du représentant ? La valeur reprise par `formVatRate` (`vatOptions.some(o => o.value === p.vatRate)`) est-elle inchangée ?
3. `InvoiceForm.svelte` : la liste des valeurs (`vatOptions`) est-elle la même qu'avant, à l'ordre près ? Une ligne existante garde-t-elle son taux ?
4. `SupplierInvoiceForm.svelte` : `{#each distinctVatRates(vatRates) as r (r.rate)}` : la clé est-elle unique ? `extraRates` (taux d'une ligne absent
   des taux actifs) reste-t-il cohérent avec la liste dédoublonnée ? Y a-t-il un `{#each}` sur des taux qui garde une clé non unique ? Inventaire clos :
   `grep -rn "#each .* as .* (" frontend/src --include=*.svelte`, filtre des clés qui ne sont ni un `id` ni un index.
5. Les tests rougissent-ils sans le correctif ? **Joue la mutation** sur une copie sauvegardée (retire l'appel de `distinctVatRates` au catalogue,
   puis dans `SupplierInvoiceForm.svelte`), restaure par copie puis `touch`, et vérifie que `git status --porcelain` est vide.
6. Le manuel (`docs/manual/`, PDF compris, aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) décrit-il autre chose que ce comportement
   (liste des taux d'une facture ou d'un article) ? Vérifie au grep.
7. Propagation : greppe la valeur, pas la formulation (`getVatRates`, `vatRates`, `vatOptions`, `\bSet(`) sur tout le dépôt ; chaque site non
   dédoublonné est résolu ou écrit comme angle mort assumé.

## Interdits

- **Le seul fichier que tu écris** : ton rapport, `$GATE_LOGS/e64/64-1-revue-p3.md`.
- **Commandes interdites** : `scripts/*` (dont `prepare-release.sh`, `test-fast.sh`, `gate-ci.sh`, `mem-guard.sh`), `docker`, `mariadb`, `sqlx`, `curl`, `wget`,
  `git commit|checkout|stash|restore|reset|push|worktree`, `gh` en écriture, `sed -i`, `perl -pi`, `tee`, `>`, `>>` vers le dépôt, `pkill`, `killall` et `kill` par motif.
- Rien sous `~/kesh/`, rien hors du worktree, de `/tmp` et de `$GATE_LOGS/e64/`.
- Permis : `git log|show|diff|grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -` en tube ; Vitest d'un fichier sous `flock $GATE_LOGS/.gate-front.lock`.

## Rapport (obligatoire)

- Les findings CRITICAL, HIGH, MEDIUM, LOW, chacun avec sa preuve (ligne lue citée, commande et sortie, ou mutation).
- **Tout finding qui affirme une ABSENCE** (code attendu manquant, site non traité) se vérifie d'abord par `grep -nF "<chaîne>" <fichier>` (`-F` obligatoire) et cite la
  commande et sa sortie ; un finding réfuté est écarté et consigné comme faux positif.
- **La liste des axes exercés et non exercés** (questions 1 à 7), avec la raison de chaque non-exercice. Un « 0 finding » sans cette liste ne compte pas comme une passe ;
  l'orchestrateur reprend lui-même les axes déclarés non exercés.
- Le décompte, recompté depuis ton rapport.
