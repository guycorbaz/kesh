## Story 29-3 — La proposition affichée suit une priorité écrite (#527)

*Registre propre à la story, à verser par union dans `epic-29-choix-autonomes.md`. Fiche :
`29-3-meilleure-proposition.md`. Choix faits le 2026-10-14 en autonomie : C-29-3-1 à 10 à la création de la fiche,
C-29-3-11 à 15 en validation P1 (C-29-3-11 tranché par l'orchestrateur), C-29-3-16 à 18 en validation P2.*

### C-29-3-1 — Une priorité écrite en trois rangs, et non le tri brut par score

- **Contexte** : #527 laisse le choix entre « candidates triées par score, toutes origines confondues » et « une règle
  de priorité écrite et dite à l'écran ». Le score de la règle (1,0) est une convention posée par `get_proposals`, non
  une mesure. Les factures « faibles » (< 0,5) sont toutes à 1–5 centimes du montant ; parmi elles, une facture dont le
  numéro figure dans la référence du paiement (0,4) est pratiquement certaine.
- **Retenu** : rang 0 = facture reconnue (montant au centime **ou** numéro dans la référence) ; rang 1 = la règle ;
  rang 2 = autres factures ; tri stable, l'ordre de `propose_matches` conservé à rang égal.
- **Écartées** : (a) tri brut par score — ferait passer la règle devant une facture référencée : produit compté deux
  fois, facture restée due, sans signal (le faux rattachement muet du `CLAUDE.md`), cas aujourd'hui bien servi ;
  (b) la règle toujours devant — idem ; (c) statu quo et une phrase à l'écran — la règle resterait cachée par une
  facture désignée par le seul nom du client, ce qui est le défaut de l'issue.
- **Réversible** : oui (une fonction pure, aucune donnée écrite).
- ⚠️ **Annotation du 2026-10-14 (validation P1, R1 HIGH)** : **rangs remplacés par C-29-3-11** (décision de
  l'orchestrateur). Le critère du rang 0 n'est plus « montant exact ou référence pleine » mais « tout signal
  d'identité » ; l'écartée (c) est en partie reprise : l'ordre atteignable reste celui d'aujourd'hui, la story l'écrit
  et le dit. Le texte ci-dessus est conservé comme relevé de la fiche initiale.

### C-29-3-2 — L'ordre se fixe au serveur, une seule fois ; le frontend lit `candidates[0]`

- **Contexte** : l'écran affiche et accepte `candidates[0]` du même tableau d'état.
- **Retenu** : `order_candidates` dans `get_proposals` ; le frontend inchangé sur ce point.
- **Écartée** : retrier au frontend (deux définitions de l'ordre, qui divergeraient ; un client d'API recevrait un
  autre ordre que l'écran).
- **Réversible** : oui.

### C-29-3-3 — Le rang 0 se lit sur les sous-scores, pas sur un seuil du total

- **Contexte** : les sous-scores valent des littéraux (`1.0`, `0.5`, `0.0`) ; le total est une somme de flottants.
- **Retenu** : `amount_score == 1.0 || reference_score == 1.0`.
- **Écartée** : `score.total >= 0.4` (équivalent aujourd'hui, mais illisible et fragile à tout changement de poids).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1, C-29-3-11)** : le principe tient (sous-scores, jamais le total) ; la
  condition devient `amount_score == 1.0 || reference_score > 0.0 || contact_score > 0.0`.

### C-29-3-4 — La priorité est dite à l'écran par une phrase au-dessus de la liste

- **Contexte** : #527 demande qu'une priorité soit « dite à l'écran » ; le badge « 100 % » de la règle ou « 40 % »
  d'une facture affichée devant une règle cachée ne s'expliquent pas d'eux-mêmes.
- **Retenu** : clé `reconciliation-labels-priority-hint`, quatre locales, dans la branche « liste non vide », avant
  la table.
- **Écartées** : une infobulle par ligne (invisible au clavier et sur mobile, testée plus mal) ; une phrase hors de
  toute branche, comme l'indication des rejets (elle parle des propositions affichées : sans liste, elle ne dit
  rien de vrai).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1)** : condition ajoutée — la phrase ne paraît que si une ligne a une
  candidate (C-29-3-14) ; formulation refaite (C-29-3-15).

### C-29-3-5 — Le seuil `INVOICE_OVERRIDE_THRESHOLD = 0.5` reste inchangé

- **Contexte** : AC #114 de la 8-5b ; une facture à 50 % ou plus supprime la règle de la liste.
- **Retenu** : inchangé. L'ordre suffit à corriger le défaut ; abaisser le seuil à 0,4 aurait le même effet à
  l'écran mais retirerait la règle de l'API, et change un critère d'une story livrée sans nécessité.
- **Réversible** : oui.

### C-29-3-6 — Le test muet `get_proposals_invoice_candidate_overrides_rule` est réparé dans la story

- **Contexte** : il ne crée aucune facture et n'éprouve donc pas l'effacement de la règle qu'annonce son nom
  (`reconciliation_rules_e2e.rs:961-1004`). La story repose sur ce comportement (C-29-3-5).
- **Retenu** : réparation **mécanique** (créer la facture, asserter l'absence de règle) — périmètre minimal respecté
  (B3 de la rétrospective 28).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1, R2/F3)** : « mécanique » était faux — `reconciliation_rules_e2e.rs` n'a
  aucune aide de facture. Voie retenue : C-29-3-13.

### C-29-3-7 — Le trop-perçu de quelques centimes proposé puis refusé : issue, pas intégration

- **Contexte** : la tolérance de 5 centimes du dépôt charge une facture que la transaction **dépasse** de 1 à 5
  centimes ; l'acceptation la refuse en `RECONCILIATION_OVERPAYMENT` (`classify_payment`). Même famille que #548, sur
  le montant. Réparer touche le SQL de `kesh-db` et la tolérance, que la 29-2 retouche.
- **Retenu** : angle mort écrit à la fiche et au manuel ; **issue demandée** à l'orchestrateur (P2 proposée :
  proposition annoncée puis refusée, sans effet comptable). La story ne l'aggrave pas (une telle facture référencée
  passait déjà devant la règle).
- **Écartée** : intégrer (non mécanique ; conflit avec la 29-2 ; `kesh-db` touché imposerait le gate complet en
  boucle).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1, R7/F6)** : l'issue existe, **#665** (ouverte, P2, bug). Sous C-29-3-11,
  une telle facture porte toujours un signal (sinon elle n'est pas candidate) : rang 0, toujours devant la règle. Les
  autres refus d'une facture proposée sont de configuration, déjà au manuel (`user-manual.tex:2056`).
- ⚠️ **Annotation du 2026-10-14 (validation P2, R2/F1, F9/R9)** : #665 est traitée par la **Story 29-6** (même
  release v0.13.4). Le contournement prescrit en P1 au manuel (« enregistrez alors le règlement depuis la fiche de la
  facture ») est **faux** : la fiche refuse aussi le trop-perçu (`SettleInvoiceDialog.svelte:179-184`,
  `user-manual.tex:1308`) — retiré, voir C-29-3-16. Et « tous de configuration » était trop fort : les autres refus
  sont aussi d'état comptable, de course ou d'intégrité (`:2060` du manuel).

### C-29-3-8 — `candidate_rank` est un `match` exhaustif, sans bras `_`

- **Contexte** : la 29-2 peut introduire un nouveau type de candidate.
- **Retenu** : un variant non classé ne compile pas — le classement devient une décision explicite au rebase.
- **Écartée** : un bras par défaut (un nouveau type prendrait un rang par accident, en silence).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P2, R6)** : la 29-2 **n'ajoute aucun** variant (sa fiche : ni source
  ni `CandidateType` neufs) ; c'est le choix manuel de #666 qui pourrait en ajouter un. Le choix tient pour lui.

### C-29-3-9 — L'écran reste à une candidate par ligne (KF-026, #76)

- **Contexte** : la vraie fin du masquage serait d'afficher toutes les candidates ; c'est #76, ouverte depuis la 8-4.
- **Retenu** : hors périmètre ; la story rend la candidate unique choisie selon une règle dite.
- **Réversible** : oui (#76 reste le lieu de cette évolution).

### C-29-3-10 — Les tests HTTP datent la facture du jour de la transaction

- **Contexte** : la 29-2 change la fenêtre de dates des propositions ; un test de la 29-3 qui en dépendrait casserait
  au rebase pour une raison étrangère.
- **Retenu** : facture et transaction du même jour, en plein exercice 2026 — dans toute fenêtre raisonnable.
- **Réversible** : oui.

### C-29-3-11 — Une facture qui porte un signal d'identité passe devant la règle (choix de l'orchestrateur)

- **Contexte** : validation P1, R1 HIGH — le rang 2 de C-29-3-1 plaçait la règle devant des factures que le nom du
  client (0,1) ou un préfixe de référence (0,2, 0,3) rapproche du paiement. Accepter la règle pour un paiement de
  client le porte en produit : produit compté deux fois, facture restée due, sans signal — le faux rattachement muet
  du `CLAUDE.md` (*Un appariement automatique propose, il ne crée jamais*), contre lequel le manuel met déjà en garde
  (`user-manual.tex:2911`). L'argument qui fondait le rang 0 vaut pour toute facture identifiée.
- **Retenu — choix de l'orchestrateur**, appliqué : rang 0 = facture avec `amount_score == 1.0 ||
  reference_score > 0.0 || contact_score > 0.0`, quel que soit le total ; rang 1 = la règle ; rang 2 = facture sans
  aucun signal (reconnue par le seul montant à quelques centimes).
- **Examen au code avant application** (consigne de l'orchestrateur) : **aucun défaut du produit démontré.** Une
  **conséquence démontrée** : une facture sans signal a un total nul et n'est pas candidate (`matching.rs:81`) ; quand
  la règle est calculée, aucune facture n'atteint 0,5, donc aucune n'a un montant exact, donc toutes portent un
  sous-score de référence ou de contact non nul. **Le rang 2 est vide sur toute entrée atteignable, et l'ordre affiché
  reste celui d'aujourd'hui.** La story n'ajoute donc aucun changement de comportement : elle écrit l'ordre (fonction
  pure, testée), le dit à l'écran, le documente pour l'API, et répare un test muet. L'appel `order_candidates` dans
  `get_proposals` est un mutant équivalent (M1), déclaré tel ; les mutations du **critère** (M4, M5, M7) restent tuées
  par les tests HTTP.
- **Risque résiduel nommé** : le faux rattachement symétrique (une contrepartie cliente et visée par une règle, dont le
  montant tombe à 5 centimes d'une facture ouverte) fait proposer la facture ; c'est l'ordre d'aujourd'hui, visible au
  lettrage.
- **Écartées** : (a) le rang 0 de C-29-3-1 (montant exact ou référence pleine) — écarté par l'orchestrateur ;
  (b) **ne pas écrire de fonction**, un commentaire juste à la place de celui de `:722-724` et les tests HTTP seuls —
  comportement identique et moins de code ; écartée parce que la priorité resterait un effet de la place du `push`
  (c'est ce que #527 relève), que le `match` exhaustif force le classement d'un variant que la 29-2 ajouterait
  (C-29-3-8), et que le rang 2 garde une source future de placer en silence une facture sans indice devant une
  règle. **Arbitrage réversible, signalé à l'orchestrateur** : si le surcoût (≈ 20 lignes, onze cas unitaires) paraît
  injustifié pour un ordre inchangé, (b) est l'alternative la plus proche.
- ⚠️ **Annotation du 2026-10-14 (validation P2) — arbitrage rendu : la fonction est GARDÉE**, décision de
  l'orchestrateur, les deux lentilles ayant confirmé l'argument principal (la priorité écrite, et non un effet de la
  place du `push`, est le mot de #527 ; `candidate_rank` serait le seul `match` sur `CandidateType`, donc le seul site
  qui fasse refuser la compilation d'un variant neuf). Deux raisons rectifiées (R6) : le variant neuf viendrait de
  #666, non de la 29-2 ; le rang 2 ne se peuplerait que par une source hors de `propose_matches` ou un sous-score non
  binaire, non par « un poids changé ». La mutation M11 éprouve désormais ce rang (F5). Le **risque résiduel** est
  élargi aux trois voies du code (contrepartie cliente visée par une règle ; référence courte ou préfixe commun ; nom
  contenu dans un autre), et « visible au lettrage » borné au cas où la facture choisie est celle du payeur réel (R7).
- **Réversible** : oui (une fonction pure, aucune donnée écrite).

### C-29-3-12 — Le départage à score égal par proximité de date : angle mort, issue demandée

- **Contexte** : C-29-2-7 (fiche voisine non mergée) — la 29-2 fait entrer des factures échues et relancées ; à score
  égal, `propose_matches` place la plus petite `invoice_id` (la plus ancienne) en tête. Un départage par proximité de
  la date d'ancrage à la date du paiement est proposé.
- **Retenu** : non intégré ; angle mort écrit à la fiche et au doc-comment ; **issue demandée** à l'orchestrateur.
- **Écartée** : l'intégrer — non mécanique : la date d'ancrage est définie par la 29-2 (non mergée), le départage vit
  dans `kesh-reconciliation` (`matching.rs:97-105`), hors périmètre, et `order_candidates` ne réordonne pas à
  l'intérieur d'un rang (stabilité voulue).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P2, R3/F4)** : l'issue existe, **#668** (P2), traitée par la **Story
  29-5** dans la même release v0.13.4 ; attendu rectifié par l'orchestrateur selon l'art. 87 CO : à score égal, les
  factures **échues** d'abord, puis la plus ancienne. Le contexte ci-dessus recopiait une rédaction périmée de
  C-29-2-7 : la 29-2 fait entrer des factures échues, relancées **ou pas encore échues** — dont des factures anciennes
  à échéance lointaine, candidates sans limite de temps (C-29-2-11). La 29-3 ne traite pas ce départage et ne touche
  ni l'`ORDER BY` ni le `LIMIT 50` (C-29-3-17).

### C-29-3-13 — Le test muet est déplacé dans `reconciliation_e2e.rs`, non ses aides extraites

- **Contexte** : validation P1, R2/F3 — `get_proposals_invoice_candidate_overrides_rule` vit dans
  `reconciliation_rules_e2e.rs`, qui n'a aucune aide de facture ; celles-ci (`seed_validated_invoice` et ses
  dépendances, `:279-440`) sont privées à `reconciliation_e2e.rs`.
- **Retenu** : déplacer et réécrire le test dans `reconciliation_e2e.rs`, où vivent aussi les deux tests HTTP neufs de
  l'AC3 ; y ajouter `create_rule_ok` (une dizaine de lignes, patron `reconciliation_rules_e2e.rs:356-370`), dont les
  trois tests ont besoin ; un renvoi d'une ligne dans l'ancien fichier. Tâche T3, comptée comme test neuf.
- **Écartées** : (a) extraire les aides de facture vers `tests/common/` — ~150 lignes déplacées, tous les tests du
  fichier touchés, conflit assuré avec les tests de la 29-2 dans le même fichier ; (b) dupliquer les aides dans
  `reconciliation_rules_e2e.rs` — contraire au DRY du `CLAUDE.md`.
- **Réversible** : oui.

### C-29-3-14 — La phrase de l'écran n'apparaît que si une ligne a une candidate

- **Contexte** : validation P1, R10b/F8b — une liste dont toutes les lignes sont « Aucune correspondance » rendrait
  « une seule proposition est affichée par transaction » sans proposition.
- **Retenu** : condition `proposals.some((p) => p.candidates.length > 0)` dans la branche « liste non vide » ; un test
  Vitest et la mutation M9 la tiennent.
- **Écartée** : la phrase dès que la liste est non vide (C-29-3-4 initial) — texte creux dans ce cas.
- **Réversible** : oui.

### C-29-3-15 — La phrase de l'écran ne dit rien de plus fort que le code

- **Contexte** : validation P1, R5/F5 — `reference_score` vaut 1 quand la référence et le numéro se contiennent dans
  un sens **ou** dans l'autre, sans longueur minimale (`matching.rs:132-134`), et 0,5 pour un préfixe commun ; le
  contact est bidirectionnel aussi (`:148-161`). « Dont le numéro figure dans la référence » était plus étroit que le
  code.
- **Retenu** : « une facture passe devant une règle dès que sa référence, le nom de son client ou son montant exact la
  rapproche du paiement » (et l'équivalent dans les trois autres langues, termes de l'écran : *Zuordnungsregel*,
  *regola di assegnazione*, *reconciliation rule*).
- **Écartées** : « dont la référence et le numéro se contiennent l'un l'autre » — exact mais technique, et muet sur le
  préfixe et le contact ; ajouter une longueur minimale dans `matching.rs` — hors périmètre.
- **Réversible** : oui (une clé de catalogue).
- ⚠️ **Annotation du 2026-10-14 (validation P2, R1)** : la seconde phrase retenue ci-dessus affirmait une condition
  **suffisante** que le code ne donne pas — une facture n'est candidate que si son reste dû égale le montant à 5
  centimes près, dans la fenêtre de dates (`kesh-db` `reconciliation.rs:250-259`) ; un acompte dont la référence
  porte le numéro laisse la règle s'afficher. Remplacée, aux quatre langues, par une phrase **bornée à la facture
  proposée** : « Une facture proposée passe toujours devant une règle. » (de « Eine vorgeschlagene Rechnung geht
  einer Regel immer vor. », it « Una fattura proposta precede sempre una regola. », en « A proposed invoice always
  comes before a rule. »). La Story (« devant une facture proposée pour ce paiement ») suit.

### C-29-3-16 — Le manuel ne prescrit aucun contournement du trop-perçu de quelques centimes (#665)

- **Contexte** : validation P2, R2/F1. La remédiation P1 prescrivait au manuel « enregistrez alors le règlement
  depuis la fiche de la facture (issue \#665) » ; la fiche refuse le même trop-perçu (`SettleInvoiceDialog.svelte:
  179-184` ; `classify_payment` au serveur, `invoice_settlements_write.rs:217-228` ; manuel `:1308`). Seul le reste dû
  s'y enregistre, l'écart de quelques centimes n'est écrit nulle part, et la transaction reste à rejeter.
- **Retenu** : le manuel s'en tient au constat et au renvoi (« … son acceptation est refusée en trop-perçu (issue
  \#665) ») ; le contrôle du PDF vérifie l'absence du contournement. #665 est la Story 29-6, même release : au
  rebase, si elle est mergée, l'ajout s'adapte à ce qu'elle livre ou disparaît.
- **Écartée** : écrire le contournement complet (« saisir le reste dû depuis la fiche, puis rejeter la
  transaction ; l'écart n'est pas comptabilisé ») — exact au code, mais il enseignerait un geste qui laisse un écart
  de banque non comptabilisé, pour une durée de vie d'une release (la 29-6 le rend caduc).
- **Réversible** : oui (une phrase du manuel).

### C-29-3-17 — L'`ORDER BY` et le `LIMIT 50` du dépôt restent hors de la 29-3, la remarque va à la 29-5

- **Contexte** : validation P2, R3. La fiche 29-2 (version du 2026-10-14) annonce que la 29-3 « touchera probablement
  l'`ORDER BY` / `LIMIT 50` » ; au-delà de 50 candidates de même reste dû, `ORDER BY i.date ASC, i.id ASC` puis
  `LIMIT 50` (`kesh-db` `reconciliation.rs:258-259`) écartent les plus récentes, sans égard à l'exigibilité.
- **Retenu** : la 29-3 n'y touche pas (`kesh-db` hors périmètre, gate complet en boucle) ; la remarque est
  **transmise à la 29-5** (#668), qui ordonne les égalités et peut juger si la sélection avant `LIMIT 50` doit suivre
  la même règle — rapport à l'orchestrateur pour l'y noter.
- **Écartée** : l'intégrer ici — non mécanique, change un critère de la 28-7 (C-28-7-10), même famille que le
  départage.
- **Réversible** : oui.

### C-29-3-18 — L'E2E règle sa facture en `afterAll` si le parcours a échoué avant « Accepter »

- **Contexte** : validation P2, R8. Ce qui protège les autres specs de la base partagée, c'est le reste de 0,01
  laissé par l'acceptation, non le contact unique ; un échec avant « Accepter » laisse une facture due à 972,90,
  candidate à montant exact pour tout autre spec qui importe 972,90 dans sa fenêtre.
- **Retenu** : l'`afterAll` lit le reste dû et, s'il vaut 972,90, règle 972,89 par
  `POST /api/v1/invoices/{id}/settlements` ; le relevé du spec remplace toutes les références du gabarit par une
  valeur unique de préfixe `PRIO`, sans préfixe commun de quatre caractères avec un numéro de facture.
- **Écartées** : (a) écrire le risque sans parade — un échec isolé ferait rougir d'autres specs, d'un run à l'autre,
  loin de sa cause ; (b) une facture d'un autre montant — `createAndValidateInvoiceViaApi` a un TTC fixe, et une aide
  neuve élargirait le périmètre.
- **Réversible** : oui.
