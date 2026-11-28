# Prompt — validation P2 de la fiche 53-6 « Annuler un rapprochement contre-passe dans l'exercice de l'écriture » (#1674, P1), passe complète

Tu valides la **fiche** de la story **53-6** (Epic 53 « Écritures justes », release v0.21.0, vendredi 2026-12-04 ; Kesh est **en production**,
**aucune migration**). Deuxième passe : **complète**, contexte frais, modèle **Opus** (la P1 était Sonnet 5.5, la fiche d'Opus 5.5). Langue du
rapport : français. Même cadre, mêmes axes A1 à A11, mêmes interdits que le prompt de la P1 (`…-validation-prompt-p1.md`, **à lire d'abord** : il
fait foi pour les chemins, l'inventaire du code et les interdits) ; la fiche est écrite contre `origin/main` à `56d47b2cc`, lue par
`git show origin/main:<chemin>`.

## Ce que la P1 a rendu (0 CRITICAL, 0 HIGH, 9 MEDIUM, 9 LOW) et que la fiche a remédié — à **recontrôler au code**, non à croire

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e53/53-6-validation-p1.md`. Change Log de la fiche : dernière entrée.

- M1 : le code `NO_OPEN_FISCAL_YEAR_TODAY` n'existe pas (`FISCAL_YEAR_INVALID`, `kesh-db/src/errors.rs:800`) ; D5 ne retire plus clé ni bras (trois
  `match` exhaustifs de `kesh-api/src/errors.rs`). Vérifie que la nouvelle consigne compile **et** n'orpheline aucune clé (`i18n-keys.test.ts`).
- M2 : `ReversalDating` doit être `pub` (appelants de `kesh-api` et tests d'intégration). Y a-t-il une autre conséquence de l'exposer ?
- M3 : tests omis (`jour_suisse_nuit_e2e.rs` : `ac3_annulation_d_un_rapprochement_la_nuit`, `ac7_les_services_jugent_au_jour_suisse`, etc.).
  Recompte l'inventaire par la commande de la fiche et cherche ce qu'elle omet encore.
- M4 : P7 corrigée (le rattachement à une écriture existante ne crée pas d'écriture) ; Q2 gagne l'option (c) et le principe du manuel
  `user-manual.tex:826-836`. L'argumentation de Q2 est-elle désormais juste ? Manque-t-il une conséquence produit ?
- M5 : la mutation M7 est déclarée angle mort d'avance (l'exercice de l'origine est verrouillé à l'étape (3) avant toute lecture). Réfute ou
  confirme au code.
- M6 : Q3 — effets comptes de R7, R8, R9 et priorités proposées. Vérifie les lignes de chaque écriture (règlement client avec écart, règlement
  fournisseur, achat) avant d'accepter « bilan seul » pour R7/R8 sans écart.
- M7 et M8 : détection (D7) — portée lue sur `journal_entries`, sortie d'un cas du panneau (Q4 enrichie).
- M9 : sites de manuel (`user-manual.tex:384-386`, `:826-836`, `:3971`, `:4144`, `admin-manual.tex:3559`) et d'API (`api-external.md:1005`, `:1007`,
  `:1065`). Relis le **PDF aplati** et cherche ce que la liste omet encore.
- L1 à L9 : colonnes lues à l'étape (1) (`je.fiscal_year_id`), bras `PeriodLocked` des trois `match`, mutations M10 et M11, Q5 (la migration de la
  53-2 n'est pas acquise), devises et réévaluation, noms de D6, candidates (AC-9), numérotation chronologique (Q1).

## Ce que tu cherches de plus

Les défauts **nés des remèdes** (un remède qui contredit un autre passage de la fiche : D2, D5, Q2, AC-11, AC-15, T-tâches, mutations), et ce que
la P1 n'a pas exercé (voir sa liste d'axes non exercés dans son rapport). Rappel de la règle de boucle : une passe qui rend un finding au-dessus de LOW
appelle la passe suivante (Sonnet) ; **les findings CRITICAL ou HIGH qui affirment l'absence d'un code se prouvent par un `grep -nF` cité**.

## Ce que tu rends

Comme la P1 : findings classés CRITICAL/HIGH/MEDIUM/LOW avec sévérité, axe, `chemin:ligne` et ligne copiée, correction ; puis les axes **exercés et
non exercés** (raison) ; puis le décompte `C/H/M/L`. Un « 0 finding » sans cette liste ne compte pas. Interdits : ceux de la P1 (aucune écriture,
ni `cargo`/`npm`/`make`/`docker`/`gh` en écriture, ni `scripts/prepare-release.sh`, jamais `pkill`/`killall`/`kill` par motif).
