# Epic 36 — registre des choix autonomes de la story 36-10 (clé `36-10`)

Registre propre à la story 36-10 (#1147, #1053), **versé par union**. Chaque choix : contexte, option retenue, options
écartées, réversibilité. Code lu à `0bf3c3b4b` (`origin/epic/36-0.18.0`). Fiche : `36-10-dernieres-dates-utc.md`. Le
registre du kickoff (`e36-choix-autonomes-e360.md`) n'a pas d'étape 0 pour cette story (phase 2) ; C-e360-11 (CHANGELOG)
s'applique. Hérités de l'Epic 35 : C-e350-16 (le fuseau est une constante), C-e350-25 (le jour d'un instant enregistré),
C-e355b-6 (`paid_at` est une date).

---

### C-36-10-1 — Les bornes d'un jour de Zurich, définies par `jour_suisse` (fiche, D1)

- **Contexte** : le filtre du journal d'audit demande le sens inverse de `jour_suisse` (un jour → ses instants), qui
  n'existe pas au dépôt. Ce sens est ambigu en général (trous et doublons d'heure), non à minuit à Zurich depuis 1894.
- **Retenu** : `debut_du_jour_suisse` et `fin_du_jour_suisse` dans `kesh-core/src/horloge.rs`, à côté de `jour_suisse`,
  contractés et testés par elle (± 1 ms), sans panique sur un `LocalResult::None`.
- **Écartée** : poser `+01:00`/`+02:00` dans `kesh-db` : une seconde écriture du fuseau.
- **Réversible** : oui (fonctions pures, aucune donnée).

### C-36-10-2 — Le journal d'audit filtre en jours de Zurich dans le dépôt ; l'an 9999 perd sa dernière heure (fiche, D2, D3)

- **Contexte** : `push_where_clauses` est la clause unique de la liste et de l'export ; `dateTo=9999-12-31` rendait une
  entrée posée à `23:59:59.999` UTC, qui est le `10000-01-01` à Zurich.
- **Retenu** : la conversion dans `push_where_clauses`, `AuditLogListQuery` inchangé ; l'entrée de la dernière heure de
  l'an 9999 n'est plus atteinte par un filtre de date (assumé, écrit) ; la borne basse liée est **toujours plafonnée** à
  `1000-01-01 00:00:00.000` (celle de `1000-01-01`, `0999-12-31` UTC, sortirait de la plage de `DATETIME`).
  *(Annoté le 2026-11-15, validation P1, V1-3 : ce choix disait « se vérifie au développement, plafonnée sinon ».)*
  *(Annoté le 2026-11-15, validation P2, V2-1 : les deux bornes, plafond compris, se calculent dans une fonction pure de
  `kesh-db`, `bornes_du_filtre`, testée sans base ; c'est le seul test qui voit le plafond.)*
- **Écartées** : (a) convertir dans la route ; (b) étendre la plage admise à `10000-01-01` : hors de ce que la base stocke.
- **Réversible** : oui.

### C-36-10-3 — L'ancre de la réconciliation : le jour du rappel calculé hors du SQL (fiche, D4)

- **Contexte** : #1053 propose `CONVERT_TZ` ou un calcul hors SQL. `CONVERT_TZ` fonctionne sur le MariaDB de dev, mais
  dépend des tables de fuseaux du serveur de chaque installation, et rend `NULL` sans elles — `COALESCE` en ferait la date
  de facture, sans erreur.
- **Retenu** : proposition réécrite `X >= e OR last_sent_at >= debut_du_jour_suisse(e)` (équivalence exacte, aucun
  élargissement avant `LIMIT 50`) ; acceptation `max(X, jour_suisse(r))` en Rust ; un test d'accord des deux.
- **Écartées** : (a) `CONVERT_TZ` ; (b) le décalage d'été en SQL ; (c) pré-filtre élargi puis tri en Rust (le `LIMIT`
  écarterait une candidate).
- **Réversible** : oui (aucun schéma ; la règle ne gagne qu'un jour la nuit).

### C-36-10-4 — La garde de la date du jour s'étend au jour d'un instant enregistré (fiche, D5)

- **Contexte** : `date_du_jour_garde.rs` ne voit que la date du jour ; `DATE(rmd.last_sent_at)` lui échappait.
- **Retenu** : une famille de plus, sur `crates/*/src` : `\bDATE\s*\(` SQL à 0 (motif sans lookahead, la crate `regex`
  n'en ayant pas), `.date()` en liste close de trois ; sondes ; angles morts écrits (dont les bornes `and_hms*` de #1147,
  tenues par le seul test du dépôt). *(Annoté le 2026-11-15, validation P1, V1-2, V1-4.)*
- **Écartée** : n'écrire qu'un angle mort.
- **Réversible** : oui (un test).

### C-36-10-5 — Les libellés des filtres perdent « jour UTC » ; le CSV garde ses heures UTC (fiche, D6)

- **Contexte** : le libellé disait vrai d'un comportement qui disparaît ; le CSV porte des instants (#669).
- **Retenu** : « Du » / « Au » (et les trois autres langues), sans clé neuve ; l'export inchangé — en-tête `Date (UTC)`
  et lignes `AAAA-MM-JJ HH:MM:SS.mmm` sans fuseau écrit (`routes/audit_log.rs:374`). *(Annoté le 2026-11-15, validation
  P1, V1-1 : ce choix disait « lignes `…Z` », forme de la réponse JSON, non du CSV.)*
- **Écartées** : (a) « Du (jour suisse) » ; (b) le CSV à l'heure de Zurich : changement d'un export hors de #1147.
- **Réversible** : oui (quatre valeurs de catalogue).
