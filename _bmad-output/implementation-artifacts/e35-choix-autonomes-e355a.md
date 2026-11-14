# Story 35-5a — choix autonomes de l'agent de développement (clé `e355a`)

*Registre ouvert le 2026-11-13 par l'agent de développement (Opus 5.5), worktree `kesh-e355a`. Mise à jour par union
(C-e350-8).*

### C-e355a-1 — Le squelette se commite vert ; les rouges se relèvent sur l'arbre, non dans un commit rouge

- **Contexte** : la fiche veut le squelette d'AC1 commité d'abord, puis les rouges « déterministes » sur lui.
- **Option retenue** : commit `076d63ed` = squelette seul (`jour_suisse` au corps UTC, `jour()` retirée, compte de la garde à
  1), vert. Les tests neufs sont écrits dans l'arbre, lancés sur ce squelette, et leurs rouges journalisés
  (`kesh-gate-logs/e355a/rouges.log`, `rouges-*.log`) ; ils sont commités avec la bascule, qui les met au vert.
- **Autres options** : un commit de tests rouges entre les deux — l'historique porterait un commit rouge (bisect).
- **Réversibilité** : totale.

### C-e355a-2 — Les cas de nuit d'AC2, AC3, AC4 et AC7 vivent dans un fichier neuf, `jour_suisse_nuit_e2e.rs`

- **Contexte** : AC2 à AC4 et AC7 demandent un montage commun (exercices 2025 et 2026, horloge arrêtée la nuit) que n'a
  aucun fichier existant, chacun ayant le sien (exercice de l'année courante, ou 2020-2030).
- **Option retenue** : un fichier neuf, un `spawn_app(pool, instant)` à horloge arrêtée (par le helper
  `common/horloge.rs::avec_horloge` de la fiche, D5), un montage `monter(pool, instant, exercice_2026)` sur
  `seed_accounting_company` (exercice ramené à 2025 en SQL, exercice 2026 ouvert, clos ou absent). Les pièces sont datées
  en dates fixes (décembre 2025) : aucune lecture de l'heure réelle ni de `KESH_TEST_INSTANT`. AC5 reste dans
  `fiscal_years_e2e.rs`, dont il réemploie le parcours du chemin B (`spawn_app_a`), et le test retourné reste dans
  `horloge_routes_e2e.rs`.
- **Autres options** : un test par fichier existant de chaque route (sept `spawn_app` à doter d'une horloge, sept montages
  à refaire pour deux exercices).
- **Réversibilité** : totale.

### C-e355a-3 — AC7 : « aucun exercice ouvert au jour suisse » se monte sans exercice 2026, non avec un 2026 clos

- **Contexte** : AC7 dit « un règlement dont l'exercice du jour suisse est clos ». Deux montages essayés sur le squelette :
  2026 **clos d'avance** → la validation de la facture de 2025 est refusée (`LATER_FISCAL_YEAR_CLOSED`, `rouges-api.log`) ;
  2026 clos **après** le montage → la liste rend `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (rang 2 bis : un exercice suivi d'un
  exercice clos est une période close), **quel que soit le jour** (`rouges-api-2.log`) — le test ne distinguerait plus le
  jour suisse du jour UTC.
- **Option retenue** : pas d'exercice 2026 (titre ci-dessus). Le rang 5 juge `has_open_covering_date(2026-01-01)` : faux,
  donc `FISCAL_YEAR_INVALID` ; au jour UTC (31 décembre, 2025 ouvert), aucun motif. Même fait que vise l'AC (le rang 5 au
  jour suisse), par la seule voie qui ne fige pas 2025. Écart à la lettre de l'AC, déclaré à la fiche.
- **Réversibilité** : totale.

### C-e355a-4 — E3 (la date lue une fois avant l'attente d'un verrou) n'entre pas au manuel

- **Contexte** : transmis par la 35-5a0 — une contre-passation entrée dans les toutes dernières secondes d'une année, qui
  attend le verrou d'une clôture, est refusée là où l'ancien code, relisant l'heure, la datait du lendemain. Avec le jour
  suisse, la frontière passe à minuit de Zurich.
- **Option retenue** : pas de phrase au manuel. La fenêtre est de quelques secondes, le refus est nommé (« aucun exercice
  ouvert… » ou exercice clos) et se rejoue d'un clic ; l'écrire ferait douter d'une règle simple (« la date du jour est
  celle de Zurich ») pour un cas qu'aucun utilisateur ne provoque à dessein. Le comportement reste tracé par #1105 (la
  lecture unique n'est tenue par aucun filet).
- **Autres options** : une note au § *La date du jour est celle de Zurich*.
- **Réversibilité** : totale (une phrase à ajouter).

### C-e355a-5 — L'écran du verrou : la veille calculée sur les champs de `todayLocal`

- **Contexte** : D7 — `maxLockDate` = la veille du jour local.
- **Option retenue** : `todayLocal()` découpé en année, mois, jour ; `new Date(y, m - 1, d - 1)` (le `Date` local normalise
  le 0 du mois et le 0 janvier), rendu par `todayLocal(…)`. Aucun `toISOString`, si bien que l'entrée d'allowlist de
  `local-date-guard.test.ts` tombe (huit → sept). Vitest à fuseau épinglé sur quatre instants : 11 mars, 1er mars et
  1er janvier à 00 h 30, un jour d'été.
- **Réversibilité** : totale.

### C-e355a-6 — Aucune spec E2E ne passe à `todayInZurich()` dans cette story

- **Contexte** : Dev Notes, *La nuit* : « les specs qui assertent une date d'un site de la 35-5a passent au helper
  `todayInZurich()` — au moins, à confirmer : `invoices-settlement-cancel`, `journal-entries`, `invoice-write-off`,
  `supplier-invoice-draft-cycle` ». Lu (`grep -nE "today\(\)|toHaveText\(.*today|toContainText\(.*today|entryDate"`) :
  leurs dates UTC (`invoices-settlement-cancel.spec.ts:39-40` `today()`, `:63`, `:64`, `:107` ; `journal-entries.spec.ts:646` ;
  `invoice-write-off.spec.ts:88` ; `supplier-invoice-draft-cycle.spec.ts:577-584`) datent les **pièces envoyées** (facture,
  règlement, écriture, lot) ; aucune n'est comparée à une date que pose un site de la 35-5a (la contre-passation n'est
  jamais lue par sa date). `supplier-invoice-draft-cycle.spec.ts:685-696` lit déjà la date locale du poste.
- **Option retenue** : aucune spec modifiée ; la liste (vide) au Change Log, la 35-5c part d'elle. La nuit, une pièce datée
  du jour UTC D et contre-passée au jour suisse D+1 reste valide (la contre-passation suit). Le E2E ne peut pas prouver la
  nuit par exécution : son serveur est le binaire servi, qui ne lit jamais `KESH_TEST_INSTANT` — angle mort écrit.
- **Réversibilité** : totale.

### C-e355a-7 — AC9 : la campagne d'instants, et les rouges qui ne viennent pas de la nuit

- **Contexte** : AC9 demande la nuit réelle reproduite de jour (tous binaires) et deux instants fixés (`2025-12-31T23:30Z`,
  `2026-03-10T23:30Z`) « verts » pour les binaires des fichiers 35-5a ; la 35-5a0 transmet F3 (un instant à plus de trois
  jours, minuit, bascule d'exercice). Sous les deux instants de la fiche, cinq tests rougissent — dont `reconciliation_e2e`
  (fichier 35-5a) et `supplier_invoices_draft` (converti par la 35-5a0) — parce que leurs montages posent des **dates fixes de
  2026** (verrou au `2026-05-15`, borne au `2026-03-31`…) que l'instant précède ; le même relevé, **de jour** au 11 mars 2026
  (`nuit-C9.log`), rend les mêmes cinq.
- **Option retenue** : campagne de onze passages (`kesh-gate-logs/e355a/nuit.log`) : référence, nuit réelle, minuit ±1 s, −3 et
  −45 jours, les deux instants de la fiche, un passage d'année **postérieur** aux dates fixes (`2026-12-31T23:30Z`), et deux sondes
  de jour (15 janvier 2027, 11 mars 2026) qui séparent ce qui tient à la nuit de ce qui tient à l'instant. Les cinq rouges des
  instants de la fiche sont laissés tels quels : ils ne disent rien de la nuit, ne rougissent jamais à l'heure réelle (qui ne
  recule pas), et les « corriger » reviendrait à réécrire des dates fixes voulues par leurs tests. Écart à la lettre d'AC9,
  déclaré. Les rouges de 2027 (montages à exercice fixe 2026) sont une dette hors story : issue demandée.
- **Autres options** : convertir les dates fixes en dates relatives au jour de test (cinq tests, hors du périmètre « sites de la
  35-5a », risque de changer ce qu'ils prouvent).
- **Réversibilité** : totale.
- *(Annoté le 2026-11-13, remédiation de la revue P1, A5 : l'écart est sans effet sur ce qu'AC9 veut prouver. Pour les deux
  tests de fichiers 35-5a qui rougissent sous les instants de la fiche — `reconciliation_e2e::unreconcile_of_a_closed_lettering_is_refused_in_its_family`
  et `supplier_invoices_draft::cancelled_invoice_returns_to_draft` —, la **nuit d'hiver** est prouvée par C1 (nuit réelle du
  13 novembre, `nuit-C1.log`, vert) et le **passage d'année** par C2b (`2026-12-31T23:30:00Z`, `nuit-C2b.log` : aucun des deux
  parmi les `FAIL`).)*
