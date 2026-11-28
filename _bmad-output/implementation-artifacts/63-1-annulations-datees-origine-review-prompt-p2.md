# Story 63-1 — prompt de la revue de code P2 (Sonnet, passe complète, contexte frais)

Tu conduis la **revue de code P2 de la story 63-1 de Kesh** avec le workflow BMAD `bmad-code-review` : invoque la compétence et suis-la.
Tu travailles en contexte frais, avec le modèle **Sonnet** (la P1 était Opus). C'est une passe **complète**, non ciblée : la P1 a rendu un
MEDIUM, et sa remédiation peut toucher plusieurs modules (manuel, frontend, `kesh-db`).

## Contexte

Kesh est un logiciel de comptabilité suisse ; Guy y tient ses livres réels en production. La story corrige **trois P1** (#1752, #1753,
#1754). Annuler depuis la fiche un règlement ou un solde client, le paiement d'une facture fournisseur, ou une facture fournisseur contre-passe
désormais l'écriture **à sa date, dans son exercice** (`CancellationDating::OriginDate`, mécanisme de la 53-6a) :
- un exercice d'origine clos est refusé (`409 FISCAL_YEAR_CLOSED`) ;
- une écriture sous la borne du verrou est annoncée par les fiches (`PERIOD_LOCKED`) et refusée au clic (`400 PERIOD_LOCKED`).

**Une erreur ici fausse les livres.**

## Où travailler

- Worktree `kesh-63-1`, branche `story/63-1-annulations-datees-origine` ; relis la tête par `git log -1`.
- Diff de la story : `a9b551d5a..HEAD`. Diff de la remédiation de la P1 : depuis `13f7d6177` (la tête relue par la P1).
- Commits de la remédiation de la P1 : `c12c7a09b` (manuels : M1, L12), `1bade8c97` (code : L1 à L9, L11, L13), `8891c1e7c` (E2E et captures
  `$GATE_LOGS/e63/captures/63-1-refus-au-clic-*`), puis le commit de la fiche (Dev Agent Record, Change Log). Journaux : `$GATE_LOGS/e63/rem/`.
- Fiche : `_bmad-output/implementation-artifacts/63-1-annulations-datees-origine.md` (décisions D1 à D8, AC-1 à AC-17, Dev Agent Record,
  Change Log). Les décisions du Project Lead (Q1 à Q6, Q-A à Q-D) **ne se rouvrent pas** : en particulier, le refus en période verrouillée des
  trois gestes est voulu (Q-A).
- Rapport de la P1 : `$GATE_LOGS/e63/63-1-revue-p1.md`. Il contient 1 MEDIUM (M1, manuel) et 13 LOW (L1 à L13), ainsi que neuf constats écartés **avec
  leur raison** : ne les rapporte pas de nouveau sans fait neuf.
- Preuves : `$GATE_LOGS/e63/` (`rouge/`, `mutations/`, `captures/` — **regarde les images** —, journaux de gates) et `$GATE_LOGS/idefix/<id>/`.
- `CLAUDE.md` du worktree : *Review Iteration Rule*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*,
  *Propagation post-patch — grep du symptôme*, *Recette visuelle*, *Recompter*, *Haiku-specific guardrails* (vérifier par `grep -nF` tout
  finding CRITICAL ou HIGH qui affirme une absence).

## Axes (déclare, à la fin, ceux que tu as exercés et ceux que tu n'as pas exercés)

1. **Remède de M1.** `admin-manual.tex` et `user-manual.tex` disent-ils maintenant, partout où ils conseillent de faire reculer le verrou pour
   annuler, que la période déjà déclarée change (charge, impôt préalable, TVA), que Kesh ne garde pas le décompte déposé, et ce qu'on fait de
   l'écart (correction du décompte ou décompte suivant, avec la fiduciaire) ? Les cinq endroits : § verrou de l'admin, § *Annuler une facture
   fournisseur*, § règlement fournisseur, § *Enregistrer et annuler un règlement* (solde), § décompte de TVA (« Le solde annulé »). Le remède
   ne doit ni changer une règle ni contredire Q-A.
2. **Remède des LOW traités dans la story**, s'il y en a (à lire au Change Log) : chacun se relit au code, avec la propagation du symptôme
   (`git grep` de la valeur, non de la formulation). Les LOW renvoyés en issue doivent avoir leur numéro au Change Log, après la vérification
   « déjà couvert » de CLAUDE.md.
3. **Datation, de bout en bout.** Les trois gestes passent `OriginDate`, et eux seuls ; `/journal-entries/{id}/reverse` (R10) reste daté du
   jour ; les formes `_in_tx` gardent `dating` ; les lecteurs (`services::factures::reglements`, `get_settlement_view_in_tx`,
   `supplier_invoice_cancel_blocker`) lisent sous la même datation que le clic.
4. **Inventaire des sites non résolus.** Pars du symptôme : `git grep` de `reverse_in_tx(`, `reverse_owned_in_tx(`,
   `reverse_at_next_fiscal_year_start_in_tx(`, `CancellationDating::Today`, `.aujourd_hui()` dans `crates/*/src/*`, et des doc-comments qui
   disent encore « datée du jour » ou « les gestes depuis la fiche passent `Today` » (L5 de la P1).
5. **Précédence et refus**, y compris la place du rang « lot de paiement » face à `PeriodLocked` (L2) et l'ordre écrit dans
   `docs/api-external.md`.
6. **Écran.** Ce que les deux fiches affichent d'un `400 PERIOD_LOCKED` au clic (L3, L4) ; les replis inline **mot pour mot** le FTL fr-CH ;
   les quatre locales, même sens ; les compteurs `sitesTotal` et `CLES_RELEVEES` **recomptés depuis la source** s'ils ont bougé ; la
   dépendance `lib/shared` → `features/reports` (L1).
7. **Tests.** Les attendus changés (AC-11) gardent leur intention ; le jour du test diffère toujours de la date de l'origine ; la garde
   lexicale d'AC-10 n'est pas vraie par construction (sonde positive, échantillons) ; tout test neuf de la remédiation rougit sans elle.
8. **Le manuel, PDF compris.** Recompile `docs/manual` **dans une copie** placée sous `/tmp/claude-1000/` (jamais dans le worktree), puis
   aplatis : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`. Greppe `datée du jour`, `date du jour`, `aujourd'hui`, `reculer le verrou`,
   `déclar`. Relis aussi `docs/api-external.md`.
9. **Gates.** Le Dev Agent Record n'affirme que ce qui a tourné. Le gate complet est-il vert au dernier commit de code de la remédiation ?
   L'E2E complet, condition de clôture, a-t-il tourné ?

## Interdits

Tu n'écris **aucun fichier** du dépôt. Sont interdits : `Edit`, `Write`, `git commit`, `git push`, `sed -i`, `scripts/prepare-release.sh`,
`scripts/test-fast.sh --fix`, tout script qui écrit, toute commande `gh` qui écrit, `docker` et `docker compose`, le redémarrage du conteneur,
`GRANT` et `REVOKE`, et tout `pkill`, `killall` ou `kill` par motif. Rien sous `~/kesh/`. Les tests Rust ciblés se lancent sous les verrous
de la station (`flock /tmp/station-heavy.lock flock $GATE_LOGS/.gate.lock`), sur la base `kesh_e631` ; Vitest d'un fichier, sous
`flock $GATE_LOGS/.gate-front.lock`. Pas d'E2E complet sur la station. Ton rapport va dans `$GATE_LOGS/e63/63-1-revue-p2.md` : c'est le seul
fichier que tu écris.

## Rendu

Le rapport classe chaque finding CRITICAL, HIGH, MEDIUM ou LOW, avec pour chacun :
- fichier:ligne ;
- la prémisse lue au code ;
- le correctif proposé.

Il donne ensuite la liste des axes **exercés et non exercés**. Un « 0 finding » sans cette liste ne compte pas.
