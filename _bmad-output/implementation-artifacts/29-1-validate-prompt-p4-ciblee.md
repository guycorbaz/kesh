# Prompt — passe ciblée P4 de la spec, Story 29-1 (préparer la clôture d'un exercice)

*Versionné le 2026-10-14. **Une seule lentille**, contexte frais, lecture seule stricte (CLAUDE.md § *La passe ciblée*).
Rotation (D6, rétro 25) : Haiku est réservé aux passes ciblées — c'en est une. P1 : Sonnet ; P2 : Opus ; P3 : Sonnet ;
remédiations par Opus. Tu ne reprends rien de la session qui a écrit la remédiation P3.*

Dépôt `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1`. Fiche :
`_bmad-output/implementation-artifacts/29-1-preparer-cloture.md`. Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-1.md` (C-29-1-1 à 30). Registre de sprint :
`_bmad-output/implementation-artifacts/sprint-status.yaml` (clés `29-1*` et `29-7-repetition-a-blanc`). Rapports de
P3 : `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p3-R.md` et `-F.md`. Le Change Log de la fiche (dernière
entrée, « validation P3 ») résume la remédiation, finding par finding, et déclare le signal D5.

## Ton périmètre — le SEUL diff de la remédiation P3

```sh
git diff a28e548e 647b961d -- _bmad-output/
```

Lis-le **en entier**, et **seulement** lui (plus ce qu'il faut lire du fichier final pour juger un passage). Tu ne
revalides pas la story : tu cherches ce que **cette remédiation** a cassé, oublié ou mal affirmé. ⚠️ Ce diff est **un
seul commit** : les numéros de ligne de ses hunks sont ceux du fichier final ; pour toute affirmation, lis le fichier
final (`sed -n`, `grep -nF`), pas le diff.

## La lentille — chasseur de régression de la remédiation

### Axe 1 (prioritaire) — les deux retraits sont-ils COMPLETS, et RIEN ne les remplace-t-il ?

Décisions de l'orchestrateur, à vérifier telles quelles (« une forme qui produit un défaut à chaque correction se
retire ») :

1. **M-b13 retirée.** Dans la fiche **vivante** (tout ce qui précède `## Dev Agent Record`, et les Completion Notes),
   il ne doit rester **aucune** affirmation d'équivalence ni d'inatteignabilité au sujet de M-b13 ou de l'état
   « ordinaire ouvert / verrouillante clos », ni de « défense en profondeur hors d'atteinte » :
   `grep -nE "M-b13|équivalent|atteignable|défense en profondeur|hors d'atteinte|par la course" <fiche>` et trie
   chaque occurrence — sont admises : les entrées **datées** du Change Log (P1, P2, P3), les deux annotations de
   retrait (AC12 et sous la table des mutations), M-a9 (équivalence propre, hors retrait). Vérifie aussi que :
   - M-b3 reste dans la table, tuée par T-b3 ; l'AC11 garde « le statut de N+1 n'est jugé QUE par cette confirmation
     verrouillante » ;
   - l'AC12 dit seulement que la réouverture de N+1 peut changer l'état lu entre la vue et la confirmation, et que
     c'est la confirmation qui décide — **sans** argument de plus ;
   - **rien ne remplace M-b13** : aucune mutation neuve sur la confirmation, aucun ordre (iii) de T-b5, aucun test neuf
     de course sur `reopen` ; M-b1 à M-b12 seulement, et la Tb8 dit M-b1 à M-b12 ;
   - le registre (C-29-1-27) est **annoté**, non réécrit (le texte d'origine reste, l'annotation datée suit).
2. **Volets (c) et (d) de T-a2 retirés.** L'AC2 et T-a2 n'ont plus que (a) — tout appel de requête a pour premier
   argument une constante de `ALL_SQL` — et (b) — aucun mot de verrou dans le source décommenté :
   `grep -nE "\(c\)|\(d\)|liste close|quatre volets|pub\(crate\)|BALANCES_SQL|EARLIER_OPEN_SQL|NEXT_AFTER_SQL" <fiche>`
   et trie (les « étape (c) / (d) » de `close` sont sans rapport). Vérifie que :
   - l'angle mort est écrit **comme angle mort** (fonctions de dépôt externes nommées dans le doc-comment du module,
     vérifiées en revue de code, « aucun test ne le garantit »), à l'AC2, à T-a2 et à Ta2 ;
   - **rien ne remplace (c) et (d)** : aucun volet neuf, aucun cas de contrôle neuf, aucune mutation neuve visant les
     fonctions externes ou leur SQL ;
   - le passage de `BALANCES_SQL`, `EARLIER_OPEN_SQL`, `NEXT_AFTER_SQL` en constantes **privées** est cohérent avec
     leurs usages prescrits ailleurs dans la fiche (qui les lit ? T-a15 est-il un test **unitaire** de
     `fiscal_years.rs`, donc capable de voir une `const` privée ? `closing_checks` les nomme-t-il encore ?) — et que les
     `pub(crate)` restants (`balances_by_bank_account`, `BalancesRow`, `is_shared`) sont bien appelés depuis un autre
     module ;
   - C-29-1-26 est annoté, non réécrit.

### Axe 2 — chaque passage neuf est-il vrai au code ?

Pour chaque ligne, fichier, constante, numéro de ligne que la remédiation P3 cite, `grep -nF` ou `sed -n` sur le dépôt
(les cotes sont à `16eeed21` = l'état actuel du code, que la branche ne touche pas) — en particulier :
`frontend/src/lib/features/homepage/HomepageCards.test.ts` l. 166-188 ; `crates/kesh-api/tests/textes_coherents.rs`
l. 679-705 (`NOM`, `TABLE`) ; `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` l. 1120-1146 ;
`crates/kesh-db/src/repositories/journal_entries.rs` l. 333-345, 398-402, 2771-2773 ;
`crates/kesh-db/src/repositories/companies.rs` l. 909-912 ;
`frontend/src/routes/(app)/journal-entries/[id]/+page.svelte` l. 211-235 et
`journal-entry-page.test.ts` l. 49 ; `crates/kesh-db/migrations/20260504000001_bank_imports.sql` (colonnes du
quatrième `EXISTS`) ; `docs/api-external.md` l. 284 et § 10. Le grep élargi de l'AC17
(`"datée du jour\|daté du jour\|celle du jour"` sur `docs/ README.md website/ crates/`) rend-il bien les deux
commentaires nommés, et d'autres sites que la fiche ne nomme pas ?

### Axe 3 — la remédiation a-t-elle introduit un défaut ?

- **Contradiction** entre un passage neuf et un passage resté (ex. : le quatrième `EXISTS` de `NO_BALANCE_STATEMENT`
  contre « Trois `EXISTS` séparés » de la population ; l'état de chargement de l'AC5 contre l'AC6 ; la fixture neuve
  de M-a4 contre le reste de T-a5) ;
- **mutation déclarée tuée qui ne le serait pas** : M-a4 (relevé arrêté avant `Y.end_date`, cas de contrôle non
  archivé), M-a22 (chargement), M-b5 (assertion neuve d'égalité exacte), M-b11 réécrite (le `JOIN` retiré : T-b5 (ii)
  rougit-il vraiment ?) ;
- **décompte faux** — recompte depuis la source : 19 AC vivants, T-a1 à T-a16, T-b1 à T-b9, M-a1 à M-a22 (M-a9
  équivalente), M-b1 à M-b12, C-29-1-1 à 30 (`grep -c "^### C-29-1-"`) ; 17 findings P3 (R 2 MEDIUM + 12 LOW, F
  1 MEDIUM + 2 LOW) ;
- **entrées passées réécrites** : `git diff a28e548e 647b961d` ne doit montrer, dans les entrées datées de la création,
  de P1 et de P2 (Change Log, Completion Notes, registre, commentaires du registre de sprint), **que des ajouts**.

### Axe 4 — manuel (nommé, CLAUDE.md § *Le prompt d'une passe doit NOMMER le manuel*)

La remédiation P3 ne prescrit qu'un changement de texte visible : l'état de chargement du panneau (AC5, AC7) et la
description de la route dans `docs/api-external.md` (AC17). Ce que l'AC17 prescrit d'écrire à `api-external.md` est-il
vrai au code prescrit (date, absence de corps, `201` et `JournalEntryResponse`, codes, `PERIOD_LOCKED`,
`LATER_FISCAL_YEAR_CLOSED`) ? Le manuel utilisateur (`docs/manual/fr/user-manual.tex`, et le PDF aplati :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) dit-il aujourd'hui quoi que ce soit que la
remédiation contredise ?

## Ce que tu rends

Findings numérotés `P4-1`, `P4-2`… avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**,
correction proposée, et pour chacun : naît-il de la remédiation P3 (passage du diff `a28e548e..647b961d`) ou
préexistait-il ? ⛔ **Toute affirmation de présence ou d'absence d'un texte est étayée par la sortie de `grep -nF`
sur le fichier FINAL**, copiée dans le rapport — jamais par une lecture du diff seul (CLAUDE.md § *Haiku-specific
guardrails*). Un finding « X n'a pas été retiré » ou « X manque » sans cette sortie ne compte pas.

⛔ **La liste des axes exercés ET non exercés** (axes 1 à 4, et pour l'axe 1 chacun des deux retraits séparément) —
un « 0 finding » sans elle ne compte pas comme passe. Pour chaque axe exercé, la commande qui l'a exercé.

Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p4-ciblee.md` ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`, et là seulement ton rapport. Lecture seule stricte :
**aucun `git checkout`, `git switch`, `git checkout --detach`, `git worktree` ni aucune autre forme** ; aucune commande
qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`stash`/
`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` et
tout autre interpréteur**, `docker`, `docker compose`, **`mariadb`, `mysql` et toute requête SQL**, `curl` vers une
instance, `bash` sur un script du dépôt, `sed -i`, `gh` en écriture (`issue`, `pr`, `comment`…). Aucune donnée réelle
de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`, aucun `.keshbackup`). Autorisés : lecture, `grep`, `sed -n`,
`awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
