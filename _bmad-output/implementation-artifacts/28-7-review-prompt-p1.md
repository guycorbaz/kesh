# Prompt — revue de code P1, Story 28-7 (une facture postérieure au paiement n'est plus proposée)

*Versionné le 2026-10-13. Trois lentilles (**Sonnet**), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-7`, branche `story/28-7`, **rebasée sur `origin/main`** (`5629acc2`,
28-6 fusionnée) : seuls les commits de la 28-7 sont à revoir.
**Le diff à revoir** : `git diff 9f4e5237 0d94fdc7 -- . ':(exclude)_bmad-output'` (`9f4e5237` = étape 0, validation
close ; `91e1494d` = tests rouges ; `ee36da36` = correction serveur ; `7b10e555` = libellé, **dernier commit de
code** ; `b2cdf995` = manuel, PDF et CHANGELOG ; `0d94fdc7` = PDF régénéré après le rebase ; les commits suivants ne portent que la fiche, le registre des choix,
le registre de sprint et ce prompt). ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter.
Fiche : `_bmad-output/implementation-artifacts/28-7-facture-posterieure-paiement.md` — AC1 à AC11, T0-T8, Dev Notes
(inventaires (a), (b), (c), propagation, « ce qui doit être préservé », exclusions), et le **Dev Agent Record**
(relevé T0, rouge d'abord, campagne de mutations, gates, décomptes). Choix C-28-7-1 à C-28-7-13 (C-28-7-4 retiré)
au registre `epic-28-choix-autonomes.md`. Règles : `CLAUDE.md` du dépôt — § « Recompter ses propres comptes
rendus », § « Propagation post-patch » (greper la **valeur**), § « Le prompt d'une passe doit NOMMER le manuel »,
§ « Une passe qui ne déclare pas ses axes exercés ne compte pas », § *Pattern batch* (`POST /accept` est un lot :
tout refus de date est un `FailedProposal`), § « Inventorier les sites NON RÉSOLUS », § « Migration breaking
policy » (aucune migration attendue). Issue : #548 (la PR la fermera — `closes #548`).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`.

**Contexte** : `kesh-db` — `repositories/reconciliation.rs` : constantes `INVOICE_WINDOW_DAYS_AFTER_INVOICE`,
`PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS`, fonction `payment_date`, type `InvoiceDateWindow` (`for_payment_date`,
`for_transaction`, `check`), enum `InvoiceDateRefusal`, nouvelle signature de `find_unpaid_invoices_for_window`
(SQL `BETWEEN ? AND ?`, `ORDER BY i.date ASC, i.id ASC` avant `LIMIT 50`), module `#[cfg(test)] mod tests` neuf ;
tests `tests/reconciliation_repository.rs` (test de fenêtre réécrit, test d'ordre neuf, appels adaptés) et
`tests/invoice_write_off.rs` (deux appels). `kesh-api` — `routes/reconciliation.rs` : `WINDOW_DAYS` retirée,
`get_proposals` passe `InvoiceDateWindow::for_transaction(&tx)`, `accept_one_invoice` remplace ses deux `if` de
dates par `check` ; tests `tests/reconciliation_e2e.rs` (deux tests HTTP neufs et leurs auxiliaires `*_287`).
`kesh-i18n` — une clé, quatre catalogues. Frontend — repli et doc-comment de `failed-proposal-label.ts`, son test,
commentaire de `tests/e2e/reconciliation-amount-due.spec.ts`. Documentation — `docs/manual/fr/user-manual.tex`
(trois sites) et son PDF, `CHANGELOG.md`.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :

1. **Algèbre de la fenêtre.** `check` rend-il exactement ce que rendaient les deux `if` supprimés
   (`paid < F − 1` ⇔ `F > D + 1` ; `paid > F + 30` ⇔ `F < D − 30`), bornes incluses, à toute date ? L'ordre des deux
   tests de `check` change-t-il un résultat quand les deux bornes seraient violées à la fois (est-ce possible) ? Le
   SQL `BETWEEN earliest AND latest` lie-t-il les mêmes dates, dans le bon ordre ? Démonstration « #529 non
   aggravé » (fiche) vraie au code ?
2. **Contrat d'API inchangé.** Codes, `details.reason`, `details.window_days` (= 30, désormais lu de la constante)
   identiques ; ordre des contrôles de l'étape 6 conservé (statut, payée, écriture, puis dates) ; garde de signe de
   la 28-6 (`refuse_non_credit_settlement`) toujours **avant** l'étape 6 ; la date du règlement écrit
   (`paid_at_candidate` et ses lectures : `paid_at_dt`, exercice ouvert, `entry_date`, `settled_on`) inchangée.
3. **Une seule définition.** Reste-t-il dans le dépôt une autre fenêtre de dates facture/transaction ? Méthode :
   `grep -rnE "Duration::days\(|INTERVAL \? DAY|DATE_SUB|DATE_ADD|WINDOW_DAYS|window_days" crates --include=*.rs`
   et l'inventaire (b) de la fiche (sites non résolus assumés : `settle_invoice`, `write_off_invoice`, la CHECK
   `chk_invoices_paid_at_after_date`, les deux dialogues du frontend). L'inventaire est-il **clos** au `7b10e555` ?
   Le doc-comment d'`InvoiceDateWindow` nomme-t-il ces sites par leur **nom** (non par un numéro de ligne) ?
4. **L'`ORDER BY` et le `LIMIT 50`.** Place syntaxique (après `HAVING`, avant `LIMIT`) ; colonnes qualifiées
   `i.` ; effet sur les autres appelants (aucun hors tests ?) ; la mutation M10 déclarée équivalente au plan
   observé l'est-elle pour la bonne raison ?
5. **Les tests prouvent-ils ce qu'ils nomment.** AC4 (`P == A`, sept décalages, raisons des trois refus) ; AC5
   (cinq cas, `settled_on` au cas (b) **et** (d)) ; AC6 (unitaires, test de fenêtre réécrit, test d'ordre et son
   assertion de montage). Le rouge d'abord (`91e1494d`, journal `kesh-gate-logs/28-7-t1-rouge.log`) est-il bien le
   rouge attendu ? Montages : montants distants de plus de 0.05, références distinctes, `D` en plein exercice 2026,
   unicité des numéros de facture du helper de dépôt (horodatage en nanosecondes) sur 52 insertions.
6. **Libellé.** Le texte des quatre catalogues est-il celui de l'AC7, mot pour mot ? Le repli est-il **égal** au
   fr-CH, apostrophe `’` comprise (G13) ? `i18nMsg(` inchangé (1962 aux deux bornes, à recompter :
   `git show <ref>:… | grep -c 'i18nMsg('` sur `frontend/src`) ; doc-comment et commentaire de test vrais.
7. **Décomptes.** Recompte depuis la source ce que déclare le Dev Agent Record : tests neufs
   (`git show <ref>:crates/kesh-api/tests/reconciliation_e2e.rs | grep -c '^#\[sqlx::test'`, idem pour
   `crates/kesh-db/tests/reconciliation_repository.rs`, et `grep -c '#\[test\]'` sur
   `crates/kesh-db/src/repositories/reconciliation.rs`, aux **deux** bornes `9f4e5237` et `7b10e555`), fichiers
   touchés (`git diff --stat`), mutations tuées / équivalentes, chiffres des gates contre les journaux
   `kesh-gate-logs/28-7-*`. Le **périmètre** de chaque nombre est-il déclaré ?
8. ⛔ **LE MANUEL — `.tex` ET PDF.** `docs/manual/fr/user-manual.tex`, § *Algorithme de matching*, le paragraphe
   `\label{sec:facture-posterieure}` (titre et corps disent-ils « plus d'un jour après » ? « depuis la version 0.13.3 »,
   jamais « à venir »), et la FAQ « aucun match automatique » : disent-ils **vrai au code** ? Le règlement depuis la
   fiche **et** depuis la page des échéances est-il décrit tel que l'écran le montre (`SettleInvoiceDialog.svelte`,
   `WriteOffDialog.svelte` : `min={invoiceDate}`, garde `settledOn < invoiceDate`) — sans tolérance d'un jour, sans
   borne de 30 jours après ? Le manuel promet-il quelque chose de faux ailleurs (grep `30 jours`, `proposée`,
   `fenêtre` sur `docs/manual/fr/*.tex`, `marketing-brochure.tex:167` compris), ou le README, `website/`,
   `docs/api-external.md` ? Contrôle le **PDF aplati et normalisé** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g"`
   vers `/home/gcorbaz/devel/kesh-gate-logs/28-7-review-p1-<lettre>-user-manual-flat.txt` : aucune `??`, l'ancienne
   phrase « peut être proposée » absente, les nouvelles présentes. Le CHANGELOG (section `## [0.13.3] — Non publié`)
   dit-il vrai, et rien de plus que le manuel (plafond de 50 non décrit, C-28-7-13) ?

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Débordement de `NaiveDate` dans
  `for_payment_date` (`d ± Duration` panique-t-il, et sur quelle entrée atteignable ? cf. C-28-7-12, angle mort
  écrit) ; effet d'un `?` d'erreur SQL dans la boucle de `get_proposals` ; transactions sans date de valeur ;
  transaction dont la date de valeur précède ou suit de beaucoup la date de comptabilisation ; facture redatée entre
  la proposition et l'acceptation ; la sélection groupée du frontend (`p.candidates[0]`, revalidé au serveur) ;
  imports devenus inutilisés, `clippy`.
- **E — Edge Case Hunter : bords, et mutations.** Rejoue mentalement M1-M15 (journal
  `kesh-gate-logs/28-7-mutations-run.txt`, sorties dans `kesh-gate-logs/28-7-mutations/`) et cherche une
  **mutation plausible qui resterait verte** (ex. : `check` qui teste `earliest` avant `latest` ; `ORDER BY i.date`
  sans `i.id` ; `for_transaction` qui lit `booking_date` dans un seul des deux lecteurs ; `window_days` codé en dur
  à 31 ; `payment_date` inversé dans le seul règlement écrit). Les équivalences déclarées sont-elles vraies ? Les
  tests passent-ils à vide dans un cas (ensembles vides comparés, `failed` vide) ? Indépendance et rejouabilité
  (IBAN, hash, numéros de facture).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC1-AC11) contre le code :
  satisfait, partiel, absent. Les tests nommés par la fiche existent-ils sous **ces noms** ? Les régions touchées
  sont-elles celles de la fiche, et « ce qui doit être préservé » l'est-il (`git diff 9f4e5237 7b10e555 --
  crates/kesh-api/src/routes/reconciliation.rs`) ? Aucun fichier de `migrations/`, de `test-schema/`, ni
  `post_restore.rs`, ni `Cargo.toml` touché (`git diff --stat`). Les chiffres de gates du Dev Agent Record
  correspondent-ils aux journaux ? Les choix C-28-7-1 à 13 : justifiés, ou défaut ? Le relevé T0 (tableau des
  lignes) est-il exact au `7b10e555` ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-7-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée,
origine (conception d'origine ou développement). Pour tout finding affirmant qu'un code ou un texte est absent ou
présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo`
(aucune sous-commande), `rustc`, `npm`, `npx`, `node`, `deno`, `bun`, `python`/`python3`, `perl`, `ruby`, `php`,
`sed -i`, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget` (aucune requête
HTTP), `gh` en écriture, `pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`,
`sed -n`, `wc`, `sort`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
