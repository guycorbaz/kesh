# Prompt — validation P3 de la fiche 53-6 « Annuler un rapprochement contre-passe dans l'exercice de l'écriture » (#1674, P1), passe complète

Tu valides la **fiche** de la story **53-6** (Epic 53 « Écritures justes », release v0.21.0, vendredi 2026-12-04 ; Kesh est **en production**,
**aucune migration**). Troisième passe : **complète**, contexte frais, modèle **Sonnet** (la P2 était Opus 5.5). Langue du rapport : français.
Même cadre, mêmes axes A1 à A11, mêmes interdits que le prompt de la P1 (`…-validation-prompt-p1.md`, **à lire d'abord** : il fait foi pour les
chemins, l'inventaire du code et les interdits) ; la fiche est écrite contre `origin/main` à `56d47b2cc`, lue par `git show origin/main:<chemin>`.

## Ce que la P2 a rendu (0 CRITICAL, 0 HIGH, 3 MEDIUM, 8 LOW) et que la fiche a remédié — à **recontrôler au code**, non à croire

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e53/53-6-validation-p2.md`. Change Log de la fiche : dernière entrée.

- M1 : `PERIOD_LOCKED` devient le **neuvième motif** du dé-rapprochement à l'écran (D5) ; au clic, le `400 PERIOD_LOCKED` du socle se traduit en
  ce motif avec `details.lockedThrough` ; deux tests du frontend qui le prenaient pour « code hors motifs » (`reconciliation-cancel.test.ts:97-105`,
  `CancelReconciliationDialog.test.ts:101-117`) changent de sens ; inventaire des tests recompté (Dev Notes). Vérifie : (a) qu'aucun **autre**
  consommateur du type `ReconciliationCancelCode` ou de `reconciliationCancelErrorMessage` (le dialogue de rattachement, `LinkSettlementModal.svelte`,
  `failed-proposal-label.ts`, la fiche de facture qui monte le dialogue) ne change de comportement sans que la fiche le dise ; (b) que les deux
  commandes d'inventaire de la fiche rendent bien ce qu'elle en dit (rejoue-les, compte) ; (c) ce que l'inventaire omet encore (Playwright
  `frontend/tests/e2e/`, tests unitaires du frontend qui assertent « aujourd'hui »).
- M2 : Q3 recommande désormais **trois P1**. L'argument (« bilan seul » fausse le solde de clôture de N ; une priorité par geste) est-il juste au
  regard de CLAUDE.md § *Priorités des défauts* ? La conséquence (trois P1 dans la release suivante) est-elle bien dite ?
- M3 : Q4 (a2) dit que le cas ne quitte jamais le panneau. Est-ce vrai au code (une contre-passation R peut-elle être modifiée, redatée ou
  supprimée par un chemin quelconque — `PUT /journal-entries/{id}`, suppression, dévalidation —, ce qui ferait sortir le cas) ?
- L1 à L8 : type public étroit `CancellationDating` (D2) — compile-t-il avec `pub fn reverse_owned_in_tx` et les appelants de `kesh-api` ? ; clé
  sans variable et borne ajoutée par l'écran (D5) ; seuil de D4 et `local-date.ts` ; mutation M6 ; sites `api-external.md:990`, `:1006`, manuel
  `:4113`, `LinkSettlementModal.svelte:394-399` ; raison des commentaires « inatteignable » ; dialogue monté sur la fiche de facture.

## Ce que tu cherches de plus

Les défauts **nés des remèdes de la P2** (un remède qui en contredit un autre : D2, D4, D5, Q3, Q4, AC-9, AC-15, mutations, inventaire), et les
axes que la P2 déclare **non exercés** (voir son rapport). La P2 a levé le **signal de recyclage** (§ *Règle de splitting préventif* de la fiche) :
dis si ta passe le confirme (un MEDIUM qui revient sous une autre forme ou naît d'un correctif de la P2) ou l'infirme. Rappel de la règle de
boucle : une passe qui rend un finding au-dessus de LOW appelle la passe suivante (Opus) ; **les findings CRITICAL ou HIGH qui affirment
l'absence d'un code se prouvent par un `grep -nF` cité**. Le manuel se vérifie contre le code **PDF compris**, aplati avant de greper :
`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` (PDF de `origin/main` : `git show origin/main:docs/manual/fr/user-manual.pdf > <tmp>/u.pdf`).

## Ce que tu rends

Comme la P1 : findings classés CRITICAL/HIGH/MEDIUM/LOW avec sévérité, axe, `chemin:ligne` et ligne copiée, correction ; puis les axes **exercés et
non exercés** (raison) ; puis le décompte `C/H/M/L`. Un « 0 finding » sans cette liste ne compte pas.

## Interdits

Ceux de la P1 : aucune écriture (ni dans le worktree, ni dans le dépôt principal, ni sous `~/kesh/`) ; pas de `git commit`, `git push`,
`git checkout` d'une autre branche, `git merge`, `git stash` ; ni `gh` en écriture, ni plus de deux `gh` en lecture ; ni `cargo`, ni `npm`, ni
`make`, ni `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/prepare-release.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni
aucun script qui écrit ; ni `docker`, ni `sqlx migrate`, ni `mariadb` ; jamais la base du NAS ; jamais `pkill`, `killall`, ni `kill` par motif.
Fichiers temporaires : seulement sous ton répertoire temporaire de session.
