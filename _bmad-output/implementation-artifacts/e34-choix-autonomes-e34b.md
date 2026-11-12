# Epic 34 — choix autonomes du développement de la 34-11b (clé e34b)

*Story 34-11b (#972), le relevé d'une carte de crédit. Développement du 2026-11-12, Opus 5.5, worktree
`/home/gcorbaz/devel/kesh-e34b`, branche `story/34-11b-carte-credit-releve` (base `3c1510d8`, 34-4 fusionnée en `a8dce4a9`).
Les choix de la fiche sont au registre `e34-choix-autonomes-e341.md` (C-e341-22 à 28, 35 à 37) ; ceux-ci les précisent
ou s'en écartent, et le disent.*

### C-e34b-1 — La condition 4 suit la règle de sens contraire de la 34-4, et non le lien actuel au grand livre

- **Contexte** : la fiche (C-e341-22 précisé à la P1, cas (i), (ii), (iii)) lisait le lien **actuel** de l'autre compte
  bancaire à son grand livre. La 34-4 a depuis adopté, pour le second côté, la règle de **sens contraire** (C-e344-11 :
  `bt.bank_account_id <> ? AND bt.amount = -m.movement`), sans lire aucun lien, parce qu'un lien se modifie après coup
  (transmission `e34-transmission-34-11b.md`, mise à jour du 2026-11-12, recommandation).
- **Retenu** : la condition 4 (`linkable_side`) admet une écriture si **aucune** transaction liée n'est autre chose que
  l'autre côté (autre compte bancaire, montant opposé à `M`) **et** si une transaction au plus est liée. Un seul prédicat,
  `other_side_predicate`, dont dérivent la condition 4, le second côté compté par la page et l'autre côté de l'annulation.
  Les cas (i), (ii), (iii) de la fiche sont tous refusés par cette règle : même compte → `<>` ; même grand livre, délié ou
  relié ailleurs → même sens (un mouvement rapproché a le signe de la transaction qui l'a rapproché). AC-4c vérifié par
  `s34_11b_ac4c_comptes_relies`.
- **Corrigé à la remédiation P1 (A1-2, 2026-11-12)** — la phrase précédente est inexacte. (i) et (ii) sont refusés ; (iii)
  n'est refusé que pour une transaction du **même** grand livre que le côté déjà rapproché (la forme d'AC-4c) ; pour
  **l'autre** mouvement de l'écriture, de sens contraire, il est **admis** — à juste titre : le signe de la transaction
  rapprochée prouve le mouvement (`s34_11b_a1_2_relie_ailleurs_l_autre_mouvement_admis`). Et un cas que la fiche admettait
  est désormais **refusé** : le virement éclaté sur plus de deux comptes bancaires — aucun montant n'est opposé à celui du
  premier côté (`s34_11b_a1_2_virement_eclate_sur_trois_comptes_refuse`) ; écrit au manuel (« Un virement éclaté… n'a plus de
  second côté »). Les écritures de **règlement** sont exclues du prédicat (C-e34b-8, récrit).
- **Écartée** : la forme de la fiche (deux règles pour « l'autre côté », la 34-4 et la 34-11b, que `m` pourrait faire
  diverger : `k − m < 0`, transmission point 2).
- **Écart à la fiche** : AC-1 dit « au plus une transaction par compte du grand livre » ; la règle dit « au plus une par
  côté, de comptes et de sens opposés ». Les doc-comments disent la seconde.
- **Réversible** : oui (aucune migration, un prédicat).

### C-e34b-2 — `m` = les candidates déjà rapprochées

- **Contexte** : AC-10 et la transmission (point 2, première option).
- **Retenu** : `EntrySqlMode::MatchedElsewhere` = la liste **plus** `HAS_LINK` ; `count_already_booked_for_page` lit
  deux colonnes (`candidate`, `linked`) sur le même texte. `m ≤ k` par construction. Test d'égalité AC-8 de la 34-4 vert
  sans modification ; le test unitaire du texte est récrit (`the_matched_elsewhere_query_is_the_list_query_plus_the_link`).
- **Conséquence assumée** : une écriture rapprochée de l'autre côté **mais non candidate** (l'écriture déjà rapprochée de
  ses deux côtés, vue d'une troisième transaction) n'est plus signalée — la P4 de validation l'avait relevé et laissé.
  L'écran lit `m > 0, k − m ≤ 0` comme (b′) : un `k = 0, m = 1` que le serveur ne rend pas n'est jamais une ligne muette
  (cas Vitest demandé par la transmission).
- **Réversible** : oui.

### C-e34b-3 — La décision « autre côté » en tête d'`entry_cancel_blocker`, qui reçoit `&BankTransaction`

- **Contexte** : C-e341-35 demandait de remplacer `unlinking: Option<i64>` par `concerned: Option<&BankTransaction>` dans
  `entry_cancel_blocker` et la garde de règlement (et, LOW F6 de la P4, dans les deux `…_unlinking`), `None` gardant son
  sens strict.
- **Retenu** : `entry_cancel_blocker(conn, company_id, entry_id, concerned: &BankTransaction)` — **non optionnel** :
  ses deux appelants (`cancel_blocker`, les candidates) ont toujours une transaction. La décision (`transfer_other_side`)
  est son étape 0, avant le classement et les rangs 1 à 5 ; ensuite `unlinking = Some(concerned.id)` comme avant. Les
  `…_unlinking` et `settlement_entry_cancel_blocker` gardent `Option<i64>` : les écrans de facture qui passent `None`
  (`invoice_settlements_write.rs:806`, `supplier_invoices.rs:2462`) ne traversent **jamais** `entry_cancel_blocker` —
  `None` reste strict par construction.
- **Conséquence pour M7c** : « `None` traité comme autre côté possible » n'a plus de site dans `entry_cancel_blocker` ;
  la mutation est rejouée sous la forme « la décision déplacée dans la queue commune (`settlement_entry_cancel_blocker`),
  lue sans transaction concernée », que la non-régression facture client doit tuer.
- **Corrigé à la remédiation P1 (A1-8)** : la mutation **jouée** est « le rang 3 sauté sous `None` » (`None => None` dans
  `settlement_entry_cancel_blocker`), non la forme écrite ci-dessus ; elle est rejouée avec la dévalidation fournisseur
  (`supplier_invoice_draft_e2e`) dans son lot. Depuis la remédiation P1 (B1-4), la fonction s'appelle
  `entry_cancel_outlook` et rend `CancelOutlook { blocker, unlink_only }` (une seule lecture, C-e34b-10).
- **Pour les candidates**, `Some(t.id)` remplace `None` au rang 3 : `t` n'étant pas liée, l'exemption n'exclut rien, et
  une transaction liée qui n'est pas l'autre côté aurait écarté la candidate (condition 4) — équivalent.
- **Réversible** : oui.

### C-e34b-4 — `unlinkOnly` aussi sur les candidates, et un texte au rattachement

- **Contexte** : le dialogue de rattachement dit, pour une candidate sans obstacle, « annuler ce rapprochement
  contre-passerait cette écriture » — faux pour l'écriture d'un virement déjà rapprochée de l'autre côté.
- **Retenu** : `EntryCandidateResponse.unlinkOnly` (même fonction `transfer_other_side`) ; `LinkSettlementModal` dit alors
  « annuler ce rattachement ne retirerait que son lien », sans case à cocher. Clé `reconciliation-link-entry-cancel-unlink-only`.
- **Réversible** : oui.

### C-e34b-5 — Message de succès sur le détail d'import

- **Contexte** : AC-4b demande un message de succès distinct pour le retrait seul ; le détail d'un import n'affichait aucun
  message après l'annulation.
- **Retenu** : `reconciliationCancelDoneMessage` (« Rapprochement annulé. » ou « Le lien est retiré ; l'écriture reste
  celle de l'autre côté du virement. »), notifié sur le détail d'import pour **toute** annulation. La fiche facture garde
  son message (un règlement client n'est jamais un retrait seul).
- **Réversible** : oui.

### C-e34b-6 — La dette d'ouverture de l'E2E par une écriture

- **Contexte** : AC-5 suppose le solde d'ouverture *Dette* de la 34-2, qui n'est pas dans la branche (34-2 `ready-for-dev`).
- **Retenu** : l'E2E pose la dette par une écriture `D charge / C passif`. Le manuel renvoie au paragraphe « Solde
  d'ouverture » de la carte (34-11a2) et aux soldes de départ.
- **Réversible** : oui — à l'intégration, l'E2E peut passer par la 34-2.

### C-e34b-7 — Le compte de dette de l'exemple : un sous-compte de 2100, sans numéro

- **Contexte** : la fiche (AC-8) proposait « 2030 sous le 20 » ; la 34-11a2 a retenu 2100 *Dettes bancaires à court terme*
  des plans livrés (`9a3904f5`).
- **Retenu** : « par exemple *Carte de crédit*, sous-compte à créer de 2100 », pour un exemple cohérent avec l'écran. Un numéro
  (« 2101 ») a d'abord été écrit : la garde `textes_coherents::les_comptes_cites_en_exemple_existent_dans_les_plans_livres` l'a refusé
  (absent des plans livrés) au gate complet ; retiré.
- **Réversible** : oui.

### C-e34b-8 — Le retrait seul ne lit pas le genre de l'écriture

- **Contexte** : `cancel_in_tx` classe l'écriture (règlement client, fournisseur, propre) avant de juger.
- **Retenu** : la décision « autre côté » s'applique quel que soit le genre ; elle vient après le verrou de l'écriture
  (étape 3) et l'`ensure_open`. Une écriture de règlement n'a qu'une ligne de banque : elle n'a pas d'autre côté dans les
  états atteignables. Le classement verrouillant de l'étape 2 reste (inoffensif).
- **Récrit à la remédiation P1 (E1-3)** : faux — un compte bancaire relié au compte débiteurs ou créanciers rend la ligne
  de la créance « mouvement » d'une banque. Le prédicat de l'autre côté (`other_side_predicate`) **exclut** désormais
  toute écriture de règlement (client, ou fournisseur payée, comme le classement) : elle n'est ni candidate une seconde
  fois, ni retirée seule. Le retrait seul ne vaut que pour le genre `Entry`. Test
  `s34_11b_e1_3_une_ecriture_de_reglement_n_a_pas_d_autre_cote`, mutation M13.
- **Réversible** : oui.

### C-e34b-9 — Le geste d'annulation porte la décision montrée (remédiation P1, E1-1, E1-2)

- **Contexte** : la confirmation (« aucune écriture inverse ») était choisie sur la vue, et la décision refaite au geste :
  une annulation de l'autre côté entre-temps faisait contre-passer l'écriture que l'écran excluait (et l'inverse).
- **Retenu** : `POST …/cancel` accepte un corps facultatif `{ "expectUnlinkOnly": bool }` ; `cancel_in_tx` (étape 3 ter)
  refuse `DbError::ReconciliationCancelChanged` → `409 RECONCILIATION_CANCEL_CHANGED` si la décision du geste diffère, sans
  rien écrire ; l'écran envoie toujours `view.unlinkOnly`, relit la vue sur ce refus et affiche le message. Sans le champ
  (clients d'avant), aucun contrôle. Tests `s34_11b_e1_1_…`, `s34_11b_e1_2_…` (déterministes, les deux sens), Vitest, M12, F4.
- **Écartées** : la version de l'autre côté dans le corps (plus lourde, même effet) ; un refus systématique sans corps
  (rupture du contrat des clés API).
- **Réversible** : oui (champ facultatif).

### C-e34b-10 — Une seule lecture de l'autre côté par candidate et par vue (remédiation P1, B1-4)

- **Retenu** : `entry_cancel_outlook` rend `CancelOutlook { blocker, unlink_only }` ; `cancel_blocker` aussi ; `get_view`, la
  route des candidates et `cancel_in_tx` (étapes 3 bis et 4) n'en font qu'une lecture.
- **Réversible** : oui.

### C-e34b-11 — (c′) compte à part les non rapprochées et les écritures de virement (remédiation P1, B1-1)

- **Retenu** : (a) se dit pour les `k − m` non rapprochées seules (`-one`/`-other`), puis `-transfer-among-one` (« s'y
  ajoute l'écriture d'un virement… ») ou `-transfer-among-other` (« s'y ajoutent N écritures de virement… »). La clé
  `-transfer-among` est retirée. Quatre locales, replis, manuel.
- **Réversible** : oui.

### C-e34b-12 — L'E2E sur un compte bancaire propre (remédiation P1, B1-8)

- **Retenu** : `credit-card-statement.spec.ts` crée (une fois par base) un compte bancaire à IBAN synthétique réservé
  (`CH7600762341100249817`, clé juste) relié à un actif propre : aucune écriture d'une autre spec ne peut signaler le débit.
- **Réversible** : oui.

### C-e34b-13 — Le corps de `POST …/cancel` se lit brut (remédiation P2, B2-1)

- **Contexte** : `Option<Json<…>>` refusait un corps vide accompagné d'un `Content-Type: application/json` (400), `null` (400) et tout autre
  type (415, `curl -d ''`) — la route acceptait tout avant la 0.17.6.
- **Retenu** : `body: Bytes`, `parse_cancel_expectation` : corps vide, `null`, `{}`, champ absent ou `null`, ou corps qui n'est pas du JSON →
  aucun contrôle ; `expectUnlinkOnly` booléen → contrôle ; `expectUnlinkOnly` d'un autre type → `400 VALIDATION_ERROR` (une attente mal formée
  ne doit pas devenir muette). `null` vaut absent. Test `s34_11b_b2_1_…` (six formes acceptées, une refusée), mutation M14.
- **Écartée** : refuser un corps non JSON (rupture du contrat des clés d'API).
- **Réversible** : oui.

### C-e34b-14 — (b′) au pluriel (remédiation P2, E2-3)

- **Retenu** : clé `reconciliation-labels-already-booked-transfer-other` quand `m ≥ 2` et `k − m ≤ 0` ; quatre locales ; Vitest ; mutation F5.
- **Réversible** : oui.

### C-e34b-15 — L'exclusion des règlements bornée par société et rapprochée du classement (remédiation P2, B2-3, E2-4)

- **Retenu** : les sous-requêtes de `other_side_predicate` portent `company_id = bt.company_id` ; un test unitaire rapproche leurs critères de
  `SUPPLIER_SETTLEMENT_CLASSIFY_SQL` (mêmes colonnes, même statut, même nombre de critères), et les tests HTTP `s34_11b_e1_3_…` (fournisseur)
  et `s34_11b_b2_2_…` (client) vérifient que l'écriture exclue est celle que la vue classe règlement. Le texte n'est pas partagé : la constante
  du classement porte des paramètres `?` que le prédicat corrélé ne peut pas reprendre tel quel.
- **Réversible** : oui.
