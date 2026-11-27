# Prompt — validation P2 de la fiche 53-6a « Annuler un rapprochement contre-passe dans l'exercice de l'écriture, à sa date » (#1674, P1), passe complète

Tu valides la **fiche** de la story **53-6a** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), Epic 53 « Écritures justes »,
release **v0.21.0** (vendredi 2026-12-04). Kesh est **en production** : une annulation de rapprochement dont la contre-passation tombe
dans un autre exercice que son écriture fausse deux exercices des livres réels. **Aucune migration** annoncée. Deuxième passe de la 53-6a :
**complète**, contexte frais, modèle **Sonnet** (P1 : Opus, 0/1/1/10, remédiée dans la fiche). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-53-6`, branche `story/53-6-annulation-rapprochement-exercice`. Fiche :
`_bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine.md`. Rapport de la P1 :
`/home/gcorbaz/devel/kesh-gate-logs/e53/53-6a-validation-p1.md` (**à lire** ; recontrôle ses remèdes au code, ne les crois pas). Rapports de
la fiche d'origine : `…/e53/53-6-validation-p1.md`, `…-p2.md`. **Q1 à Q6 sont tranchées par le Project Lead** (2026-11-27) : tu ne les
rouvres pas, tu contrôles qu'elles sont appliquées partout. ⚠️ Code de référence : **`origin/main` à `0e99cd2db`** (v0.20.0 ; code de la story
identique à `56d47b2cc`, contre lequel la fiche est écrite) — lis-le par `git show origin/main:<chemin>`.

**Les points les plus importants** — vérifie-les au code, ligne à ligne, avant tout le reste ; si l'un est faux, c'est un HIGH au moins :
1. **Remède H1 de la P1** : les gestes groupés (`grouped_settlements::cancel_in_tx`, `supplier_cancel_in_tx`) refusent eux-mêmes tout motif
   de la queue ; la fiche en tire, pour R4/R5, un refus `409 PERIOD_LOCKED` sans `details` (D5, AC-9, M6, notes de version, AC-14 pour la
   table de l'API). Est-ce exact (statut, code, texte, absence de `details`) ? Est-ce cohérent partout (D5 « qui refuse », commentaires des
   trois `match` de `kesh-api/src/errors.rs`, AC-9, M6, AC-14, notes de version, frontend `reconciliationCancelErrorMessage`) ? Le choix
   « ne pas filtrer les groupes » laisse-t-il un défaut (par exemple : l'écran d'un groupe, la borne absente au clic, un test qui passerait à
   vide) ? Une autre famille (`SettlementNotCancellable`, `SupplierInvoiceNotCancellable`) peut-elle, elle aussi, voir `PeriodLocked` ?
2. **Remède M1 de la P1** : AC-14 nomme-t-il maintenant **tous** les sites du manuel (`.tex` et PDF aplati) que la story rend faux, y compris
   ceux que D1 et D4 prescrivent ? Cherche ce qu'il omet encore (méthode : partir du symptôme, non de la liste).
3. D2, D5 et P7 (datation `OriginDate` dans `reverse_in_tx_inner`, rang `PeriodLocked` à sa place dans l'ordre réel du socle, inventaire R1 à
   R11 clos) : la P1 les a trouvés justes ; recontrôle-les sans t'y fier.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Ce qu'impose la production*, *Qualité du code* (dont « un appariement automatique propose »), *Review
   Iteration Rule* (dont *Inventorier les sites NON RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes
   exercés*, *Propagation post-patch*), *Priorités des défauts*, *Migration breaking policy* (pour constater qu'elle ne s'applique pas),
   *Règle de splitting préventif*, *Versions de l'API*, *Recette visuelle*, *Test Locally First*.
2. `docs/agents-orchestration.md` (§ 2 et 2.1 : interdits ; § 3 : processus).
3. `_bmad-output/planning-artifacts/epic-53-ecritures-justes.md` (la 53-6a, ses fichiers déclarés, ses recoupements ; la 53-6b sortie de l'epic).
4. Le code, à `origin/main` : `crates/kesh-db/src/repositories/journal_entries.rs` (`reverse_in_tx`, `reverse_owned_in_tx`, `ReversalDating`,
   `reverse_in_tx_inner`, `create_in_tx_inner` étapes 0-bis, 1, 1-a, 1-bis), `reconciliation_cancel.rs` (en entier),
   `settlement_cancellation.rs`, `invoice_settlements_write.rs` (`settlement_cancel_blocker*`, `cancel_settlement_in_tx`,
   `cancel_settlement`), `supplier_invoices.rs` (`supplier_settlement_cancel_blocker*`, `cancel_settlement_in_tx`, l'annulation de facture),
   `grouped_settlements.rs` (`group_cancel_blocker`, `supplier_group_cancel_blocker`, `cancel_in_tx`, `supplier_cancel_in_tx`),
   `fiscal_years.rs` (`find_open_covering_date`, `has_open_covering_date`, `reopen`, `close`), `companies.rs` (`lock_books`),
   `crates/kesh-db/src/errors.rs` (`SettlementCancelBlocker`, `ReversalBlocker`, `DbError::PeriodLocked`) ;
   `crates/kesh-api/src/routes/reconciliation.rs` (vue, geste, candidates), `crates/kesh-api/src/errors.rs` (mapping des motifs
   d'annulation), `crates/kesh-api/src/services/factures.rs:585` ; `frontend/src/lib/features/reconciliation/` (`CancelReconciliationDialog.svelte`,
   `reconciliation-cancel.ts`, `LinkSettlementModal.svelte`).
5. L'issue #1674, **en lecture seule**, **au plus deux requêtes `gh` dans toute la passe, espacées d'au moins une minute** :
   `gh issue view 1674 --json number,title,labels,body`.
6. Les fiches 29-1b (datation `NextFiscalYearStart`, confirmation verrouillante, mutation M-b3), 25-3-b (dé-rapprochement), 25-3-a-1
   (queue commune), 24-4c (verrou de période), 53-3 (ordre des verrous dans l'epic), dans `_bmad-output/implementation-artifacts/`.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — prémisses P1 à P11 relues au code** : chaque `chemin:ligne` cité est-il juste à `0e99cd2db` (copie la ligne) ? P4 (le défaut
  touche aussi un exercice jamais clôturé) est-il exact ? P6 (LIFO : aucun postérieur clos d'un exercice ouvert, hors état hérité) ? P7 (la
  date des écritures que créent les chemins d'acceptation et de rattachement — vérifie-la au moins pour l'acceptation de facture, la règle,
  le rapprochement manuel, l'éclatement et le rattachement à une écriture) ? P8 (aucun décompte de TVA déposé n'est persisté) ? l’ancienne P10, retirée au découpage avec la détection (la piste
  d'audit `reconciliation.cancelled` porte-t-elle `reversalJournalEntryId` depuis la 25-3-b, et quelles entrées en ont plusieurs pour une
  même inverse) ?
- **A2 — écarts `main` / branche d'epic** : le tableau (complété à la P1) est-il complet sur les fichiers de la story (`git diff --stat 03bb9ac6d origin/main
  -- <fichiers>`) ? Les numéros recontrôlés à `0e99cd2db` sont-ils justes ? Un écart oublié changerait-il une décision ? La fiche dit-elle assez au développeur comment reprendre `main` (T0) ?
- **A3 — inventaire clos (R1 à R11)** : rejoue la commande de la fiche et cherche ce qu'elle **omet** : `create_in_tx` appelé par un geste
  d'annulation sans passer par `reverse_*` ; un chemin MCP ou une clé d'API qui atteint le dé-rapprochement ; un lot de paiement annulé ; la
  dévalidation d'une facture fournisseur annulée (qui **supprime** une contre-passation) ; les avoirs (`credit_notes.rs`, quelle date ?).
  Chaque site est-il **changé**, **inchangé** avec raison, ou **angle mort assumé** ? L'étendue choisie (dé-rapprochement seul) est-elle
  cohérente avec le texte de #1674 ?
- **A4 — datation (D1, D2)** : la valeur `OriginDate` couvre-t-elle les cinq bras ? Le lettrage `reversal` (`Mode::System {
  held_open_fiscal_year_id }`) reste-t-il juste ? Le libellé perd-il bien son suffixe d'exercice ? Le JSON d'audit du chemin `Today` reste-
  t-il octet pour octet (cherche le test T-b2 de la 29-1b) ? Une origine en devise (`amount_currency`, `exchange_rate`), un exercice
  multi-devises, une réévaluation de fin d'exercice déjà passée : la date de l'origine change-t-elle un refus ou un solde que la fiche ne
  dit pas ?
- **A5 — verrous et concurrence (D3, AC-12)** : reconstitue toi-même l'ordre des verrous du dé-rapprochement, avant et après, pour une
  origine en N et une horloge en N+1 (transaction bancaire, règlement, facture, écriture et exercice, parcours de `find_open_covering_date`,
  `create_in_tx_inner`, compte bancaire en dernier). Un cycle **nouveau** apparaît-il (avec `close`, `reopen`, `create` d'exercice,
  `lock_books`, le lettrage, un autre dé-rapprochement du même groupe Wise) ? Le rejeu couvre-t-il ? La lecture non verrouillante de la
  borne (D5) est-elle sûre, et dans quelle limite ? L'entrelacement d'AC-12 b est-il jouable sans `sleep` ?
- **A6 — queue commune et écran (D5, D6, D4)** : la nouvelle variante `PeriodLocked` de `SettlementCancelBlocker` est-elle placée selon
  l'ordre réel du socle ? Les autres consommateurs de l'enum (annulation d'une facture fournisseur, `supplier_invoice_cancel_blocker`,
  services de facture, MCP, frontend des factures) voient-ils une variante qu'ils ne savent pas afficher ? Le maintien, inatteignable pour le dé-rapprochement, de
  `reconciliation-cancel-blocked-no-fiscal-year` laisse-t-il un site orphelin (grep du symptôme sur tout le dépôt, quatre locales,
  fallbacks Svelte et Rust, `i18n-keys.test.ts`) ? La phrase TVA de D4 est-elle juste au regard de P8 ?
- **A7 — NON EXERCÉ pour la détection (53-6b) ; exerce seulement ceci** : la fiche ne contient-elle aucun reste de la détection, du panneau, de D7, D8, de l'ancienne AC-14 ni de M9 ? Les AC renumérotés (AC-14 manuel, AC-15 E2E, AC-16 gates, AC-17 migration) et leurs renvois (tâches, Dev Notes) sont-ils cohérents ? Le découpage a-t-il laissé d'autres phrases tronquées (la P1 en a trouvé une, fin de D6) ? Compare la fiche à `git show c692c3406:_bmad-output/implementation-artifacts/53-6-annulation-rapprochement-exercice.md` section par section.
- **A8 — décisions Q1 à Q6** : chacune est-elle appliquée sans contradiction (Q1 date de l'origine partout ; Q2 refus `PERIOD_LOCKED` annoncé dans la vue et ⚠️ aux notes de version ; Q3 trois issues hors story, AC-11 fige R7 à R9 ; Q5 pas d'`/api/v2` ; Q6 refus `FISCAL_YEAR_CLOSED`) ? Une prémisse sous-jacente est-elle fausse (Q2 sur P7 : le rattachement ne crée pas d'écriture) ? La phrase de Q2 sur le principe du manuel `:826-836` est-elle juste ? Les textes de `kesh-gate-logs/e53/53-6-issues-q3/` (R7, R8, R9) disent-ils vrai au code ?
- **A9 — manuel, PDF compris** : `docs/manual/fr/user-manual.tex` (§ *Annuler un rapprochement*, `:4064` et suivantes ; encadré
  `sec:date-du-jour`, `:863` ; `:1121`, `:3878`, `:3967`, `:3971`, `:4010`) et `docs/manual/fr/admin-manual.tex` (verrou de période `:3559`,
  réouverture `:1673-1676`) ; cherche dans les `.tex` **et** dans les PDF aplatis
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o -i '.\{120\}datée du jour.\{120\}'`, puis
  `date du jour`, `aujourd'hui`, `contre-pass`, `Annuler le rapprochement` ; idem pour `admin-manual.pdf`) toute phrase que la story rend
  fausse et que l'AC-14 ne nomme pas ; et `docs/api-external.md` (`:677`, `:717`, `:788`, `:802`, `:988-992`, et toute mention de
  `NO_OPEN_FISCAL_YEAR_TODAY` ou de « datée du jour »). Les langues `de`, `en`, `it` du manuel existent-elles ?
- **A10 — AC constatables et tests** : chaque AC dit-il un test, une valeur attendue, un fichier ? Un test passerait-il à vide (horloge
  égale à la date de l'origine, état ancien impossible à produire sans SQL) ? Les tests existants qui assertent « datée du jour » sur le
  dé-rapprochement sont-ils tous repérés (recompte les sites cités) ? Les mutations M1 à M8, M10 et M11 tuent-elles chacune le test annoncé, et seulement
  lui (M6 : la P1 a établi qu'elle tue aussi le clic d'un groupe — est-ce bien écrit) ? Une mutation manque-t-elle (par exemple : la date lue sur la **transaction bancaire** au lieu de l'écriture) ?
- **A11 — périmètre, splitting, API, priorité** : le décompte des modules (quatre) est-il juste au sens de CLAUDE.md ? La borne de six
  stories de l'epic tient-elle (53-1a, 53-1b, 53-2, 53-3, 53-6a, solde ; la 53-6b proposée en sortie) ? La décision Q5 (pas de `/api/v2`, `b` porté par la modification du contrat) est-elle écrite conformément à § *Versions de l'API* ? Les notes de version
  disent-elles tout ce qui change pour un utilisateur et pour une intégration ? Les recoupements (53-2, 53-1a, Epic 39, Epic 38) et le fait que la 53-6b dépend de la 53-6a sont-ils
  justes ? Aucune donnée réelle n'apparaît-elle dans la fiche (montants, IBAN, numéros d'écriture des livres de production) ?

## Ce que tu rends

Un rapport en français : pour chaque finding, **sévérité** (CRITICAL, HIGH, MEDIUM, LOW), axe, `chemin:ligne` et ligne copiée, ce qui est
faux, la correction proposée. Un finding CRITICAL ou HIGH qui affirme l'**absence** d'un code se prouve par un `grep -nF` cité. Puis la liste
des axes **exercés** et **non exercés** (avec la raison) ; un « 0 finding » sans cette liste ne compte pas. Puis le décompte
`C/H/M/L`.

## Interdits

- **Aucune écriture** : ni dans le worktree, ni dans le dépôt principal, ni sous `~/kesh/`. Pas de `git commit`, `git push`, `git checkout`
  d'une autre branche, `git merge`, `git stash`.
- Ni `gh` en écriture (`gh issue create|edit|comment|close`, `gh pr …`), ni plus de deux `gh` en lecture.
- Ni `cargo`, ni `npm`, ni `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/prepare-release.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script qui écrit ; ni `docker`, ni `sqlx migrate`, ni `mariadb` ; jamais la base du NAS.
- Jamais `pkill`, `killall`, ni `kill` par motif.
- Fichiers temporaires : seulement sous ton répertoire temporaire de session.
