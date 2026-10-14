# Prompt — revue de code P1, Story 28-8 (cinq petits correctifs)

*Versionné le 2026-10-14. **Une seule lentille** (**Sonnet**), contexte frais, lecture seule. Elle combine les lentilles
B (bugs et états), E (bords, mutations) et A (conformité, décomptes, documentation) : les cinq correctifs sont petits
et indépendants (C-28-8-1), un seul touche du code de production (une suppression).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-8`, branche `story/28-8`.
**Le diff à revoir** : `git diff 6c98a75c HEAD -- . ':(exclude)_bmad-output'` (`6c98a75c` = validation close de la
fiche ; les commits suivants sont : `ebb9bc81` test #421, `e5370eb6` suppression #561, `df40a6a5` testing.md + api-external,
`cf4c0604` manuels + PDF, `7d69e5ab` CHANGELOG, puis des commits qui ne portent que le relevé E2E de `docs/testing.md`,
le Dev Agent Record, le registre et ce prompt). ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`. Fiche :
`_bmad-output/implementation-artifacts/28-8-petits-correctifs.md` (AC1-AC9, T0-T8, Dev Notes, **Dev Agent Record**).
Choix C-28-8-1 à C-28-8-8 (et suivants éventuels) au registre `epic-28-choix-autonomes.md`. Règles : `CLAUDE.md` du
dépôt — § « Recompter ses propres comptes rendus » (et « recompter n'est pas réécrire l'historique »), § « Propagation
post-patch » (greper la valeur), § « Le prompt d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses
axes exercés ne compte pas ». Issues : #421, #561, #595, #612, #615 (la PR les fermera ; les commits ne portent que
`refs`).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`.

## Axes (déclare chacun exercé ou non)

1. **#421 — le helper et sa preuve.** `recordManualReminderViaApi` (`frontend/tests/e2e/invoices.spec.ts`) : `sentAt`
   est-il un `NaiveDateTime` complet (`AAAA-MM-JJTHH:MM:SS`, sans fuseau ni millisecondes — #249) ? Le calcul
   `new Date(Date.now() - 3_600_000).toISOString().slice(0, 19)` est-il juste, y compris autour de minuit UTC (la date
   d'avant-hier/hier est-elle acceptée par la route : `dunning_reminders.rs`, `record_manual_reminder`, autres gardes de
   date ?). Jumeaux : `git grep -nE "T[0-9]{2}:[0-9]{2}:[0-9]{2}" frontend/tests/e2e` et `git grep -n sentAt
   frontend/tests/e2e` — le Dev Agent Record en copie-t-il la sortie et dit-il vrai ? La mutation M1 (date future)
   est-elle plausible et son résultat consigné ?
2. **#561 — la suppression.** `git grep -nw delete_all_by_company` : chaque ligne restante concerne-t-elle
   `journal_entries::delete_all_by_company` (à ne pas toucher) ou un commentaire/une archive ? Un import devenu
   inutilisé, un doc-comment qui renvoie à la fonction supprimée (`accounts::delete_all_by_company`) ? `git diff
   6c98a75c HEAD -- crates/kesh-db/src/repositories/accounts.rs` ne retire-t-il **que** la fonction ?
3. **#595 — `docs/api-external.md`.** Les deux entrées `MATCHED_BANK_TRANSACTION` sont-elles **au bon rang** (après
   `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, avant `ACCOUNT_ARCHIVED`) et vraies au code ? Relis la chaîne :
   `supplier_invoices.rs` (`cancel_settlement_in_tx`, `supplier_invoice_cancel_blocker`), `journal_entries.rs`
   (`reverse_owned_in_tx`, `exempts`, `reversal_blockers`), `errors.rs` (`MatchedBankTransaction`, `details.documentId`).
   `grep -c MATCHED_BANK_TRANSACTION docs/api-external.md` = 8 ? « Défensif » est-il justifié (les cinq sites qui posent
   `matched_entry_id` ne rapprochent que des écritures qu'ils viennent de créer) ?
4. **#612 — la brochure.** `marketing-brochure.tex` : la puce quitte-t-elle *Backlog* et le bloc livré est-il placé
   entre TVA et lettrage ? La réserve sur le bouclement est-elle vraie (#232 ouvert, `CHANGELOG.md` de la 0.13.0) ?
   Le bloc dit-il vrai de la v0.8.0 (`CHANGELOG.md`, rubrique 0.8.0) ?
5. **#615 — le manuel d'administration.** Le listing `lstlisting` suit-il `.env.example` (`:354-372`) ? Les deux
   phrases de prose (« décommentez… », « décommentée sans valeur… ») sont-elles vraies (relis `.env.example` et le
   comportement de Compose) ? Les chaînes gabarit des manuels 0.13.0-0.13.2 ajoutées au paragraphe des bases déjà
   créées : `git show v0.13.2:docs/manual/fr/admin-manual.tex | sed -n 255,265p`. Aucun refus n'est-il écrit pour les
   `MARIADB_*` ? `git grep -nE "MARIADB_(ROOT_)?PASSWORD=<" -- ':(exclude)_bmad-output'` ne rend rien ?
6. **`docs/testing.md` (AC2).** Le tableau a perdu `invoices.spec.ts:415` et `:439` ; `sidebar-navigation.spec.ts:75`
   nomme KF-052 (#424) ; la formule est `7 + (1 si la KF-052 se déclenche) + 0 à 2 de pollution` = 7 à 10 ; le tableau
   historique du 2026-08-30 **est resté intact** (colonne `KF-045`, « plus **KF-046** ») avec son annotation datée ;
   les relevés datés n'ont pas été réécrits. `git grep -nE "KF-045|KF-046" -- ':(exclude)_bmad-output'` ne rend que le
   tableau historique, son annotation, la note historique et le `CLAUDE.md:706` (renumérotation) ? Aucune phrase
   restante ne dit encore que le nombre d'échecs dépend de l'heure (`grep -nE "le matin|matinal|DÉPEND"`) ?
7. **CHANGELOG (AC7).** Une seule puce, dans la section unique `## [0.13.3] — Non publié`, rubrique `### Corrigé`, liens
   **en ligne** ; aucune section doublée ; rien pour #421 ni #561.
8. ⛔ **LE MANUEL ET LES PDF.** Les deux PDF (`docs/manual/fr/admin-manual.pdf`, `marketing-brochure.pdf`) sont-ils
   **à jour du `.tex`** ? Contrôle aplati :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem brochure) vers
   `/home/gcorbaz/devel/kesh-gate-logs/28-8-review-p1-<admin|brochure>-flat.txt`, puis `grep` : `#MARIADB_ROOT_PASSWORD=`
   et `#MARIADB_PASSWORD=` présents, `MARIADB_ROOT_PASSWORD=<sortie` absent, nouveau bloc « Report à-nouveau
   automatique » présent, puce absente sous « Backlog ». Cherche aussi dans `user-manual.tex`, le README, `website/`
   une phrase que ces correctifs rendraient fausse. Le PDF commité date-t-il de **après** le dernier commit qui touche
   le `.tex` (`git log -1 --format=%H -- docs/manual/fr/admin-manual.tex` contre celui du `.pdf`) ?
9. **Périmètre et décomptes.** Aucun fichier hors des six emplacements de la fiche (`git diff --stat 6c98a75c HEAD`) ;
   aucune migration, aucun `Cargo.toml`. **Recompte depuis la source** chaque décompte du Dev Agent Record (gates,
   tests, échecs E2E) contre les journaux sous `/home/gcorbaz/devel/kesh-gate-logs/28-8/` s'ils y sont, et exige que le
   record ne déclare que ce qui a tourné. Le registre des choix a-t-il été mis à jour par **union** ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-8-review-p1.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les neuf axes ci-dessus) — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et les textes aplatis des PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, **`git checkout` et `git switch` sous toute forme** (y compris `git checkout -- fichier`, `git checkout
<ref>`, `git switch -`), `git commit`/`add`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`/`worktree`, `sqlx`, `cargo`
(aucune sous-commande), `rustc`, `npm`, `npx`, `node`, `deno`, `bun`, `python`/`python3`, `perl`, `ruby`, `php`,
`sed -i`, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget` (aucune requête
HTTP), `gh` en écriture, `pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. Pour relire un fichier à une
autre révision : `git show <ref>:<chemin>`, jamais un `checkout`. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
