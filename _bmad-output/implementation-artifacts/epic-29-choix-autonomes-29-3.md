## Story 29-3 — La proposition affichée suit une priorité écrite (#527)

*Registre propre à la story, à verser par union dans `epic-29-choix-autonomes.md`. Fiche :
`29-3-meilleure-proposition.md`. Choix faits le 2026-10-14 à la création de la fiche, en autonomie.*

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

### C-29-3-4 — La priorité est dite à l'écran par une phrase au-dessus de la liste

- **Contexte** : #527 demande qu'une priorité soit « dite à l'écran » ; le badge « 100 % » de la règle ou « 40 % »
  d'une facture affichée devant une règle cachée ne s'expliquent pas d'eux-mêmes.
- **Retenu** : clé `reconciliation-labels-priority-hint`, quatre locales, dans la branche « liste non vide », avant
  la table.
- **Écartées** : une infobulle par ligne (invisible au clavier et sur mobile, testée plus mal) ; une phrase hors de
  toute branche, comme l'indication des rejets (elle parle des propositions affichées : sans liste, elle ne dit
  rien de vrai).
- **Réversible** : oui.

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

### C-29-3-8 — `candidate_rank` est un `match` exhaustif, sans bras `_`

- **Contexte** : la 29-2 peut introduire un nouveau type de candidate.
- **Retenu** : un variant non classé ne compile pas — le classement devient une décision explicite au rebase.
- **Écartée** : un bras par défaut (un nouveau type prendrait un rang par accident, en silence).
- **Réversible** : oui.

### C-29-3-9 — L'écran reste à une candidate par ligne (KF-026, #76)

- **Contexte** : la vraie fin du masquage serait d'afficher toutes les candidates ; c'est #76, ouverte depuis la 8-4.
- **Retenu** : hors périmètre ; la story rend la candidate unique choisie selon une règle dite.
- **Réversible** : oui (#76 reste le lieu de cette évolution).

### C-29-3-10 — Les tests HTTP datent la facture du jour de la transaction

- **Contexte** : la 29-2 change la fenêtre de dates des propositions ; un test de la 29-3 qui en dépendrait casserait
  au rebase pour une raison étrangère.
- **Retenu** : facture et transaction du même jour, en plein exercice 2026 — dans toute fenêtre raisonnable.
- **Réversible** : oui.
