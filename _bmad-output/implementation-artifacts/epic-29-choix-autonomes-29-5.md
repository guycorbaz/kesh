# Epic 29 — registre des choix autonomes, Story 29-5

*Registre propre à la story 29-5 (#668), à verser par **union** dans `epic-29-choix-autonomes.md` par l'orchestrateur.
Fiche : `29-5-departage-exigibilite.md`.*

---

## Story 29-5 — À score égal, la facture échue la première passe en tête ([#668])

### C-29-5-1 — La story entre dans l'Epic 29 et sort avec la 29-2

- **Contexte** : #668 est relevée pendant les fiches 29-2 et 29-3 (C-29-2-7, C-29-2-11, C-29-3-12), hors de leur
  périmètre minimal. La 29-2 élargit la fenêtre (candidates jusqu'à `F ≥ D − 60` à 30 jours net, factures non échues
  à échéance lointaine) et rend le départage par `invoice_id` plus coûteux.
- **Retenu** : story ajoutée à l'Epic 29 par l'orchestrateur le 2026-10-14, après la 29-3, dans la même v0.13.4
  (mergée avant le tag, C-29-2-13). Indépendante de la 29-4.
- **Écartée** : la reporter après la v0.13.4 (la fenêtre élargie sortirait sans le départage qui la rend sûre).
- **Réversible** : oui.

### C-29-5-2 — Le départage suit l'art. 87 CO : date d'exigibilité, puis date, puis identifiant

- **Contexte** : l'attendu d'origine de #668 (« la récente passe en tête », proximité de la date d'ancrage) contredit
  l'art. 87 CO, comme le relève la rectification de Guy ; l'attendu révisé : d'abord les échues, puis l'ancienneté.
- **Retenu** : `X = max(F, E)` (échéance absente valant la date, art. 75 CO), clé `(X, F, id)` croissante après le
  score. Constat : comme toute échue a une `X` inférieure à toute non échue, ce tri place les échues d'abord **sans**
  lire la date du paiement, puis celle échue la première.
- **Écartées** : (a) proximité de l'ancre au paiement — contraire à l'art. 87, et l'ancre mêle les rappels ;
  (b) clé explicite `(non échue, X, …)` dépendant de `D` — équivalente, plus coûteuse (un paramètre de plus au SQL) ;
  (c) la date de facture seule — ignore l'échéance, c'est le défaut.
- **Réversible** : oui (une fonction, une expression SQL).

### C-29-5-3 — Un rappel ne vaut pas poursuite ; le critère des premières poursuites reste sans donnée

- **Contexte** : la consigne suggère « premières poursuites (en Kesh : rappels ?) ». Kesh n'enregistre aucune
  poursuite (`grep -rni poursuite` : le seul libellé « Dernier rappel avant poursuite »). Un rappel est une
  interpellation (art. 102 CO), une poursuite une réquisition au sens de la LP.
- **Retenu** : ne pas lire les rappels pour le départage ; écrire l'angle mort (manuel, doc-comment) avec le recours :
  enregistrer le règlement depuis la fiche de la facture voulue, puis rejeter la transaction.
- **Écartée** : les rappels comme substitut — juridiquement faux, et il faudrait les charger pour chaque candidate
  (`propose_matches` n'en a pas la donnée).
- **Réversible** : oui (si Kesh enregistre un jour les poursuites).

### C-29-5-4 — L'`ORDER BY` du `LIMIT 50` suit l'ordre d'imputation

- **Contexte** : `ORDER BY i.date ASC, i.id ASC LIMIT 50` choisit les candidates scorées ; avec la fenêtre de la
  29-2, des factures anciennes non échues peuvent pousser hors des 50 une facture échue (C-29-2-7, renvoyé ici ;
  complément de l'orchestrateur du 2026-10-14).
- **Retenu** : `ORDER BY {X} ASC, i.date ASC, i.id ASC`. Les deux ordres coïncidant, dans chaque classe de score les
  candidates gardées forment un préfixe de l'ordre d'imputation : la troncature n'écarte jamais celle que le
  départage placerait en tête. La troncature reste aveugle au score (C-28-7-10, inchangé, écrit).
- **Écartées** : (a) la troncature écrite comme simple limite — le correctif tient en une ligne ; (b) trier par score
  en SQL — dupliquerait le score en SQL (C-28-7-10, exclusion de la 28-7).
- **Réversible** : oui.

### C-29-5-5 — Al. 2 et al. 3 : ordres par défaut écrits

- **Contexte** : échéances identiques → imputation proportionnelle (al. 2) ; aucune échue → la moins garantie
  (al. 3). Kesh propose une facture et ignore les garanties.
- **Retenu** : à `X` égale, la plus ancienne par date puis par identifiant ; parmi les non échues, `X` croissante.
  Écrit au manuel (recours : règlements depuis les fiches) et au doc-comment, avec la condition qui fonde
  l'équivalence de C-29-5-2.
- **Écartée** : proposer plusieurs factures, ou une répartition — un écran et un modèle de proposition neufs, hors
  périmètre.
- **Réversible** : oui.

### C-29-5-6 — Le score reste la première clé

- **Contexte** : une désignation par le débiteur (référence, art. 86) prime l'imputation légale ; elle passe par le
  score de référence.
- **Retenu** : le score n'est pas touché ; l'ordre d'imputation ne départage que des scores égaux. Une facture au
  montant exact non échue passe devant une échue à quelques centimes près (score). Testé (AC5 cas 7, M7).
- **Écartée** : exigibilité avant score — ferait passer une facture échue devant la facture que le paiement nomme.
- **Réversible** : oui.

### C-29-5-7 — Aucun texte d'écran ; le manuel dit la règle et ses limites

- **Contexte** : l'écran affiche une candidate ; la phrase d'aide de la 29-3 (« la facture la mieux notée ») reste
  vraie. Entre clients différents à score égal, l'art. 87 est sans objet.
- **Retenu** : ni clé i18n ni capture ni spec Playwright neuve ; la frontière HTTP est exercée par un test
  `spawn_app` (AC6) ; le manuel FR dit la règle, la désignation qui prime, les poursuites et l'al. 2 absents, et
  l'ordre par défaut entre clients ; E2E complet au dernier commit de code.
- **Écartée** : afficher l'échéance ou la raison du choix à l'écran — demande d'évolution, sans demandeur.
- **Réversible** : oui.

### C-29-5-8 — Deux écritures (Rust, SQL) dans le module de la fenêtre, tenues par un test de parité

- **Contexte** : `propose_matches` (Rust, pur) et l'`ORDER BY` (SQL) doivent appliquer la même `X`. Précédents :
  `payment_date` et `InvoiceDateWindow` (fonctions pures du dépôt, lues par `kesh-api`).
- **Retenu** : `invoice_exigibility_date`, `imputation_order` et `INVOICE_EXIGIBILITY_DATE_SQL` dans
  `kesh-db/src/repositories/reconciliation.rs` ; `kesh-reconciliation` les appelle (il dépend déjà de `kesh-db`) ;
  test de parité SQL ↔ Rust avec valeurs attendues écrites en dur.
- **Écartées** : (a) méthode sur l'entité `Invoice` — éloigne la fonction de l'expression SQL qu'elle double ;
  (b) rendre `X` dans la requête et l'ajouter au triplet candidat — change la signature de `propose_matches` et ses
  deux appelants.
- **Réversible** : oui.

### C-29-5-9 — L'ancre de la 29-2 est récrite en composant l'exigibilité

- **Contexte** : la 29-2 écrit `GREATEST(i.date, COALESCE(i.due_date, i.date), …rappel…)` ; la 29-5 écrit
  `GREATEST(i.date, COALESCE(i.due_date, i.date))`. Règle DRY du `CLAUDE.md`.
- **Retenu** : l'ancre compose l'expression d'exigibilité ; équivalente (`GREATEST` associatif), tests de la 29-2
  verts sans modification d'assertion.
- **Écartée** : laisser deux écritures du même sous-terme.
- **Réversible** : oui.

### C-29-5-10 — Les tests d'ordre existants gardent leur nom

- **Contexte** : `find_unpaid_invoices_orders_by_date_then_id_before_limit` porte des factures sans échéance ; l'ordre
  d'exigibilité s'y réduit à `(date, id)`.
- **Retenu** : nom et assertions inchangés, doc-comment complété ; un test neuf porte l'ordre avec échéances.
- **Écartée** : renommer — bruit de diff sans gain.
- **Réversible** : oui.

[#668]: https://github.com/guycorbaz/kesh/issues/668
