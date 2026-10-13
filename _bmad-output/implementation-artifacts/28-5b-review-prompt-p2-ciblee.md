# Prompt — revue de code P2 **ciblée**, Story 28-5b (annuler le rejet d'une transaction, côté écran, #526)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **la seule
remédiation de la revue de code P1** (`CLAUDE.md`, § *La passe ciblée* ; D6, rétrospective de l'Epic 25 : Haiku
réservé aux passes ciblées). Trend : P1 (Sonnet, lentilles B, E, A) **0 CRITICAL, 0 HIGH, 1 MEDIUM, 11 LOW** (E-1 ;
B-L1 à B-L4, E-2 à E-5, A-1 à A-3, dont E-2 = A-1 = B-L3 et B-L1 = E-3).*

Dépôt `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5b`. **Périmètre : la remédiation P1, lue en DIFF UNIQUE
aplati** — ne lis pas la séquence des commits (règle Haiku du `CLAUDE.md` : les numéros de ligne d'un hunk de
commit intermédiaire ne sont pas ceux du fichier final) :

```sh
git diff e6675f30 HEAD -- frontend CHANGELOG.md          # tests et CHANGELOG (un seul commit de remédiation : f0ebd1d8)
git diff e6675f30 HEAD -- _bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md
```

Rapports P1 que ce diff remédie (à lire) : `/home/gcorbaz/devel/kesh-gate-logs/28-5b-review-p1-B.md`,
`28-5b-review-p1-E.md`, `28-5b-review-p1-A.md`. Journal des mutations de la remédiation :
`/home/gcorbaz/devel/kesh-gate-logs/28-5b/mutations-p1-run.txt` et `28-5b/mutations/M21.log`, `M22.log`, `M23.log`.
Journal du gate frontend : `/home/gcorbaz/devel/kesh-gate-logs/28-5b/p1-remed-frontend.log`.

Fichier de production touché : **aucun**. Vérifie-le d'abord :
`git diff --stat e6675f30 HEAD -- frontend/src ':(exclude)*.test.ts' ':(exclude)*.spec.ts'` doit ne rien rendre,
et `git diff --stat e6675f30 HEAD -- frontend/tests` non plus (la spec E2E n'est pas touchée : c'est ce qui dispense de
rejouer l'E2E complet).

## La lentille — chasseur de régressions de la remédiation P1

Pour **chaque hunk**, vérifie contre le fichier final :

1. **E-1, A-2 — la date exacte** (`UnrejectTransactionAction.test.ts`) : la fixture `2026-03-04T11:00:00Z` donne-t-elle
   « 4 mars 2026 » en `fr-CH`, `dateStyle: 'medium'`, sur toute la plage de fuseaux annoncée (UTC-11 à UTC+12) ?
   L'assertion `toBe('Rejetée le 4 mars 2026')` dépend-elle d'un espace insécable que l'ICU de Node pourrait produire
   ou non selon sa version (`mention.trim()` suffit-il) ? M23 (`new Date()`) rougit-elle bien, et sur ce test
   (`mutations/M23.log`) ? Les autres tests du fichier, qui gardaient l'ancienne fixture, en dépendent-ils encore ?
2. **E-4 — les replis contre fr-CH** : le `Map` hoisté (`vi.hoisted`) est-il rempli **par le rendu** — et vidé avant le
   test qui le lit (`replis.clear()`) ? Un repli d'une autre clé, ou d'un rendu précédent, peut-il rendre l'assertion
   verte à tort ? `valeurFr` passe-t-elle à vide si la clé manque (`toBeDefined` présent avant `toBe`) ? La
   comparaison de l'indication (V4) se fait sur `textContent.trim()` : un retour à la ligne ou une indentation du
   `<p>` peut-il la faire rougir ou la rendre verte à tort ? Les **quatre** clés neuves sont-elles bien couvertes
   (`reconciliation-unreject-button`, `-rejected-on`, `-success`, `reconciliation-labels-rejected-hint`) ?
3. **E-2 = A-1 = B-L3 — garde et `finally`** : lever `bouton.disabled = false` à la main exerce-t-il vraiment le
   gestionnaire (le clic est-il délivré, l'appel compté) ? Svelte peut-il rétablir `disabled` entre l'affectation et le
   clic, rendant le test vert sans exercer la garde ? M21 et M22 rougissent-elles, sur les tests annoncés ? L'attente
   « le bouton redevient actif » après l'échec est-elle distincte d'un état initial (le bouton n'est-il pas
   simplement le même nœud, jamais désactivé) ?
4. **E-5 — CHANGELOG** : l'entrée *Corrigé* bornée est-elle vraie au code de la 27-1 (`409
   BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`) ? La phrase prescrite par AC9 (b) reste-t-elle en tête, mot pour mot ?
   Section unique `## [0.13.3] — Non publié` ?
5. **Fiche** : l'angle mort « fuseau du navigateur » (inventaire des sites non résolus) dit-il vrai au composant
   (`grep -nF "dateStyle" frontend/src/lib/features/reconciliation/UnrejectTransactionAction.svelte`) ? Le tableau des
   mutations (M21-M23) correspond-il au journal `mutations-p1-run.txt` ? Les LOW non corrigés (B-L1 = E-3, B-L2) sont-ils
   écrits avec leur raison, et la raison est-elle vraie (le serveur sérialise-t-il toujours un `DateTime<Utc>` :
   `grep -nF "and_utc" crates/kesh-api/src/routes/bank_imports.rs`) ?
6. **Décomptes** — recompte, ne relis pas : « 1515 passés, 142 fichiers » contre `p1-remed-frontend.log` ; « 0/0/1/11 »
   contre les trois rapports P1 (en-têtes de décompte) ; un seul test neuf (`grep -cE "^\s*it\(" ` du fichier aux
   bornes `e6675f30` et `HEAD`).
7. **Manuel contre code (obligatoire), `.tex` ET PDF aplati** : la remédiation ne change aucun texte visible de
   l'utilisateur — vérifie-le (`git diff --stat e6675f30 HEAD -- docs`) ; la puce *Rejeter* du manuel parle-t-elle du
   format ou du fuseau de la date d'une façon que la remédiation rendrait fausse ?
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` pour « Rejetée le ».
8. **Propagation** : greper la **valeur** — `2026-10-13T08` dans `frontend/src` (résidus de l'ancienne fixture, à
   trier : `bank-import-detail-page.test.ts` (`08:30`) et `reconciliation-unreject.test.ts` (`08:00`) l'emploient pour
   un autre motif — la date n'y est pas assertée, seule la présence du marqueur compte) ; `\b1514\b` et `\b1515\b` dans la fiche.

⚠️ **Entrées passées** : les entrées antérieures du Change Log et du Dev Agent Record gardent leurs chiffres d'alors
(consigne 15) — « 1514 », « 17 tests Vitest neufs » du gate final au `31df9d74` : ce n'est pas un résidu.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée, sur
le **fichier final** (`git show HEAD:<chemin>` ou le fichier du worktree). Un CRITICAL ou HIGH qui affirme qu'un patch
« n'a pas été appliqué » se vérifie ainsi avant d'être écrit.
⚠️ **Erreur de catégorie** : les LOW déclarés non corrigés (B-L1 = E-3, B-L2) et l'angle mort de la limite des 100
(#649, C-28-5-39) ne sont pas des défauts de cette remédiation ; ne les reprends que si leur **raison écrite** est fausse.

## Ce que tu rends

Findings numérotés `P2-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (remédiation P1 / rédaction antérieure), correction proposée. ⛔ **La liste des axes exercés
ET non exercés** (les huit ci-dessus) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-5b-review-p2-ciblee.md` ; dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `vitest`, `playwright`, `docker`, `bash` ou `sh` sur un script, **tout interpréteur**
(`python`, `python3`, `perl`, `ruby`, `deno`, `awk` avec un programme qui écrit), `gh` en écriture, toute requête SQL.
Rien de `~/kesh-test/` (relevé réel de Guy). Autorisés : lecture, `grep`, `sed -n`, `wc`, `find`, `git log`/`show`/
`diff`/`blame`/`grep`, `gh api` en lecture, `pdftotext` vers la sortie standard.
