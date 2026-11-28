# Story 63-1 — prompt de la revue de code P1 (Opus, passe complète, contexte frais)

Tu conduis la **revue de code P1 de la story 63-1 de Kesh** avec le workflow BMAD `bmad-code-review` : invoque la
compétence et suis-la. Tu travailles en contexte frais, avec le modèle **Opus**. C'est une **première passe** : complète,
jamais ciblée.

## Contexte

Kesh est un logiciel de comptabilité suisse ; Guy y tient ses livres réels en production. La story corrige **trois P1**
(#1752, #1753, #1754) : annuler depuis la fiche un **règlement** ou un **solde** client, le **paiement** d'une facture
fournisseur, ou une **facture fournisseur** contre-passait l'écriture **à la date du jour** — dans un autre exercice si
l'annulation était faite après la fin de celui de l'écriture (bilan faux dans deux exercices ; pour le solde et la facture
fournisseur, résultat et TVA faux). Désormais, ces trois gestes passent `CancellationDating::OriginDate` (mécanisme de la
53-6a) : l'inverse prend la date de l'écriture, dans son exercice ; un exercice d'origine clos reste refusé (`409
FISCAL_YEAR_CLOSED`) ; une écriture sous la borne du verrou est **annoncée** par les fiches (`PERIOD_LOCKED`) et refusée
au clic par le socle (`400 PERIOD_LOCKED`). **Une erreur ici fausse les livres.**

## Où travailler

- Worktree `kesh-63-1` (voisin du dépôt principal), branche `story/63-1-annulations-datees-origine`. Relis la tête par `git log -1`.
- Diff de la story : `a9b551d5a..HEAD` (`a9b551d5a` est la fusion d'`origin/epic/53-ecritures-justes`, 53-6a comprise).
- Fiche : `_bmad-output/implementation-artifacts/63-1-annulations-datees-origine.md` — décisions D1 à D8, AC-1 à AC-17,
  mutations M1 à M8, **Dev Agent Record** (relevé T0, inventaire des sites, tests par AC, mutations, gates, captures).
- Preuves hors dépôt, `$GATE_LOGS` = `kesh-gate-logs/`, voisin des worktrees : `e63/rouge/` (le rouge d'abord),
  `e63/t2-retrait-date-tests*.log` (les 232 arguments retirés des tests, un par ligne), `e63/t2-nextest-*.log`,
  `e63/mutations/` (`resume.txt`, un `.diff` et un `.log` par mutation), `e63/captures/` (**regarde les images**, et lis
  les `.txt` des boîtes natives), `idefix/<id>/` (gates distants).
- `CLAUDE.md` du worktree, en particulier : *Review Iteration Rule*, *Le prompt d'une passe doit NOMMER le manuel*,
  *Inventorier les sites NON RÉSOLUS*, *Pattern batch* (sans objet ici : le dire), *Recette visuelle*, *Recompter*,
  *Haiku-specific guardrails* (vérifier par `grep -nF` tout finding CRITICAL ou HIGH qui affirme une absence).

## Axes (déclare, à la fin, ceux que tu as exercés et ceux que tu n'as pas exercés)

1. **Datation.** Chacun des trois gestes passe-t-il `OriginDate` au socle, et seulement eux ? Les formes `_in_tx`
   (`cancel_settlement_in_tx` client et fournisseur) gardent-elles leur paramètre `dating` (dé-rapprochement, tests) ?
   `/journal-entries/{id}/reverse` (R10) reste-t-il daté du jour, octet pour octet ? Le règlement **avec écart** et le
   **solde** (correction de TVA) se contre-passent-ils entiers, à leur date ?
2. **Lire comme on écrit.** `services::factures::reglements`, `get_settlement_view_in_tx` (deux bloqueurs) et
   `supplier_invoice_cancel_blocker` lisent-ils sous la même datation que le clic ? Un bouton peut-il s'afficher que le clic
   refuse, ou l'inverse ? `commit_with_view` dans la transaction d'écriture : la borne du verrou y est lue par une lecture
   ordinaire — est-ce sans effet sur l'instantané et les verrous du geste ?
3. **Retrait de la date du jour (D3).** Plus aucun usage résiduel de `aujourd_hui` sur ces chemins (routes
   `supplier_invoices.rs`, `imported_supplier_invoices.rs`, leurs `_once`, `invoices.rs` qui garde une lecture pour
   `from_parts`) ; aucune autre route n'en dépendait. Relis le journal du retrait mécanique des 232 arguments de tests :
   chaque argument retiré est-il bien une date du jour (aucune fermeture `|_| id` retirée par erreur) ?
4. **Précédence et refus.** Rangs 1, 1 bis, 2, 2 bis, 3, 4, `PeriodLocked`, lot ; listes fermées des gestes inchangées ;
   précédence du lettrage (rang 2 bis avant `PeriodLocked`) ; `LATER_FISCAL_YEAR_CLOSED` (#568) ; seuil inclusif au jour de
   la borne et au lendemain.
5. **Concurrence (AC-12).** Ordre des verrous inchangé ; la sonde `NOWAIT` (`s631_ac12_…`) établit-elle ce qu'elle dit
   (exercice de l'achat tenu, exercice du jour libre) et rien de plus ? La course avec `lock_books` (borne lue sans verrou)
   est-elle écrite comme angle mort ?
6. **Tests.** Chaque test cité au Dev Agent Record existe-t-il et vérifie-t-il ce qu'on lui attribue ? Le jour du test
   diffère-t-il toujours de la date de l'origine (sinon un test de la date passe à vide sous `Today`) ? Les attendus changés
   (AC-11 : `jour_suisse_nuit_e2e`, `lettering_documents`, `open_items`, `supplier_invoices_draft`, les harnais de
   précédence) gardent-ils leur intention, et l'état « d'avant la v0.21.0 » posé en SQL est-il dit tel ? La garde lexicale
   (`cancellation_dating_today_lexical.rs`, AC-10) est-elle vraie par construction, ou prouve-t-elle quelque chose (sonde
   positive R10, échantillons synthétiques, contournements fermés, angle mort #1760) ?
7. **Mutations.** M1 à M8 tuées chacune par le test annoncé ; restauration par copie puis `touch`, `git diff` vide.
8. **Textes et écran.** Les deux motifs neufs et la phrase de borne (`settlement-cancel-blocked-period-locked(-through)`,
   `supplier-invoices-cancel-blocked-period-locked`), les quatre confirmations, quatre locales : même sens, replis inline
   **mot pour mot** le FTL fr-CH ; ce que les deux fiches affichent d'un `400 PERIOD_LOCKED` au clic ; compteurs
   `sitesTotal` (3979) et `CLES_RELEVEES` (481) recomptés ; `lib/shared` importe `formatSwissDate` de `features/reports` —
   acceptable, ou dette ?
9. **Le manuel, PDF compris.** `docs/manual/fr/user-manual.tex` et `admin-manual.tex` contre le code : encadré du verrou,
   `sec:date-du-jour`, exercices en désordre, § règlement client et solde, § règlement fournisseur, § annuler une facture
   fournisseur, la paire achat et contre-passation (annulations d'avant la v0.21.0), § décompte de TVA (solde annulé).
   Aplatir le PDF avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` — le PDF est
   celui de la v0.20.0 (non régénéré : il se régénère à la release) ; vérifier que **chaque** phrase du PDF qui dit
   « datée du jour » pour l'un des trois gestes a sa correction dans le `.tex`. `docs/api-external.md` : datation,
   `PERIOD_LOCKED`, `FISCAL_YEAR_INVALID` retiré, champs `…BlockedLabel`.
10. **Inventaire des sites non résolus.** Pars du symptôme, non de la liste de la fiche : `git grep` de toutes les créations
    de contre-passation (`reverse_in_tx(`, `reverse_owned_in_tx(`, `reverse_at_next_fiscal_year_start_in_tx(`,
    `create_in_tx_inner(` avec `reverses_entry_id`) et de toutes les lectures du jour (`horloge.aujourd_hui()`,
    `lecture.aujourd_hui()`, `CancellationDating::Today`) ; un geste qui annule et resterait daté du jour ?

## Interdits

Tu n'écris **aucun fichier** du dépôt et ne modifies rien : ni `Edit`, ni `Write`, ni `git commit`, ni `git push`, ni
`sed -i`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh --fix`, ni aucun script qui écrit ; aucune commande
`gh` qui écrit ; ni `docker compose`, ni redémarrage du conteneur, ni `GRANT`/`REVOKE`. Rien sous `~/kesh/`. Ton rapport
va dans `$GATE_LOGS/e63/63-1-revue-p1.md` — le seul fichier que tu écris.

## Rendu

Un rapport : findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec fichier:ligne, la prémisse lue au code, le
correctif proposé ; puis la liste des axes **exercés et non exercés**. Un « 0 finding » sans cette liste ne compte pas.
