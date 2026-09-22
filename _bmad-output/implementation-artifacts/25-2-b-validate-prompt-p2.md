# Prompt — passe 2 de `bmad-create-story validate`, Story 25-2-b

*Versionné le 2026-09-19. Une lentille en contexte frais (Opus), modèle différent des deux lentilles
de la passe 1 (Sonnet, Haiku 4.5). ⚠️ L'auteur des patches est aussi un Opus : ne fais AUCUNE
confiance au fait qu'un passage « a déjà été revu ».*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-2-b-devalidation-facture` (tirée de `main` au
`44c6842f`). Ta mission n'est pas d'approuver : c'est de **trouver ce qui ferait échouer, dévier ou
mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole, surtout pas cette spec.** Elle a été écrite le 2026-09-16, et
**trois PR ont été mergées depuis** : #439 (25-1c-a, le registre `crates/kesh-api/src/audit_labels.rs`
et sa garde), #441 (25-2-a, le retypage), #442 (25-2-c, le compteur d'écritures
`journal_entry_number_sequences`). Chaque affirmation de la spec — chemin, numéro de ligne,
signature, comportement, nombre, état d'une PR — se **vérifie dans le code actuel** avant d'être
retenue ou contestée.

⚠️ **Ne conteste PAS les arbitrages du Project Lead**, consignés dans la spec : la dévalidation
existe (plutôt qu'une contre-passation, inapplicable) ; deux cycles, dévalider-effacer **et**
dévalider-corriger-revalider ; `emailed_at` en **refus sec** ; le numéro de facture **conservé**.
Conteste leur **mise en œuvre**, et tout ce que la spec affirme au-delà.

## Ce qui a changé depuis la passe 1 — et où regarder d'abord

La passe 1 a produit 1 CRITICAL, 3 HIGH, 2 MEDIUM, et l'orchestrateur a trouvé en vérifiant **deux
trous réels de `main`** (verrou de période non contrôlé par `delete_in_tx` ; règlement partiel
ignoré par la garde `paid_at`). Les patches (commits `0bf31cac`, `ddfe5c51`, `c4cb2656`) ont
**réécrit l'AC 3** (huit empêchements, un enum sur le modèle de `ReversalBlocker`, statuts HTTP,
précédence), **l'AC 5** (cinq tests à réécrire), **l'AC 6**, **l'AC 9** (rôle arbitré :
Administrateur et Comptable, clés API admises), **l'AC 10** et **l'AC 11**, et les tâches de T1.

⛔ **Le motif mesuré de ce dépôt : la sévérité se déplace vers ce qu'on vient d'écrire.** Relis
d'abord ces passages réécrits, puis le reste. En particulier :
- la garde du verrou de période **dans `delete_in_tx`** : quels AUTRES appelants de `delete_in_tx`
  existent, et que change pour eux une garde neuve ? Une écriture d'extourne, d'ouverture ou de
  règlement peut-elle y passer ? Le rapport TVA, la contre-passation, la clôture ?
- l'enum des empêchements : comment se combine-t-il avec les variantes existantes
  (`FiscalYearClosed`, `PeriodLocked`, `EntryIsReversed`) qui gardent leurs codes — une seule erreur
  ou plusieurs chemins d'erreur ? La précédence annoncée est-elle réalisable si certaines gardes
  vivent dans `delete_in_tx` et d'autres dans `unvalidate` ?
- les clés API admises : la route dans `comptable_routes` est-elle réellement ouverte aux clés
  `read-write`, et fermée aux `read` ? Quel test, quelle garde du dépôt (`audit_route_registry`,
  `admin_pat_denied_e2e`, un registre des scopes de clés) devra bouger ?
- la numérotation : la facture brouillon portant un numéro (AC 2) — la **suppression** d'un tel
  brouillon (sortie « effacer » de l'AC 4) laisse-t-elle un trou dans la séquence des factures, et la
  spec le dit-elle ?

## Sources

- l'issue **#440** (`gh issue view 440`), et **#219** (fermée) qu'elle remplace ;
- `_bmad-output/planning-artifacts/epic-25-vague1-suite.md` ;
- `CLAUDE.md` — § *Pattern batch*, § *Propagation post-patch*, § *Migration breaking policy* (si une
  migration s'avérait nécessaire), § *Le prompt d'une passe doit NOMMER le manuel*, § *Règle de
  splitting préventif* ;
- **le code**, qui prime.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Exactitude factuelle.** Chaque `fichier:ligne` de la spec, vérifié contre `main` actuel.
   Chaque affirmation sur l'état du dépôt (« aucun chemin de retour vers `draft` », « le seul site
   passant `enforce_immutability = false` », l'état de #439…). Signale toute dérive.
2. **L'inventaire des empêchements (AC 3) est-il vraiment CLOS ?** La spec affirme l'avoir établi
   « depuis le schéma — tout ce qui référence `invoices(id)` ou l'écriture ». **Refais cet
   inventaire toi-même** depuis les migrations et le code : toutes les FK vers `invoices` et vers
   `journal_entries`, et tout **état** (pas seulement une FK) qui interdit aujourd'hui de supprimer
   ou de modifier une écriture — gel de l'Epic 24, verrou de période (`books_locked_through`),
   exercice clos, lettrage s'il existe, pièces jointes, documents, avoirs, règlements, rapprochement,
   rappels, extourne. Pour chaque site manquant ou en trop, dis pourquoi. Le décompte « les quatre
   derniers ne sont PAS gardés aujourd'hui » est-il exact au regard de `invoices::delete` actuel ?
3. **La numérotation (AC 2, 4) et la 25-2-c.** Le numéro de facture conservé puis repris : que fait
   `validate_invoice` aujourd'hui, et que faudrait-il changer exactement ? Une facture brouillon
   **portant un numéro** est-elle un état que le reste du code sait traiter — listes, filtres,
   PDF, QR-bill, unicité, `chk_*`, import/export, sauvegarde ? Côté écritures, la dévalidation
   supprime une écriture : avec le compteur de la 25-2-c, quel est l'effet exact sur la séquence, et
   la spec le dit-elle juste ?
4. **La transaction et la concurrence (AC 1, 5, 8).** Ordre des opérations, verrous, verrou
   optimiste : une dévalidation concurrente d'un règlement, d'un envoi d'e-mail, d'un rapprochement
   ou d'une revalidation peut-elle passer entre l'inventaire et la suppression ? Le retrait de la
   branche `validated` d'`invoices::delete` casse-t-il un appelant, un test, une route, l'écran ?
5. **L'audit (AC 6).** `invoice.unvalidated` doit entrer au registre `audit_labels.rs`, et la garde
   `crates/kesh-api/tests/audit_label_registry.rs` impose ses règles (littéral ou `SITES_INDIRECTS`,
   libellés dans les quatre locales). La spec le dit-elle correctement **maintenant** ?
6. **API, i18n, écran (AC 7, 9, 11).** Codes d'erreur : un par empêchement, statut HTTP de chacun,
   cohérence avec les conventions existantes (`error-*`, codes des 409 voisins). L'écran : où est
   aujourd'hui le bouton « supprimer » d'une facture validée, et que devient-il ? Un E2E existant
   rougira-t-il ?
7. **Les manuels (AC 10).** `docs/manual/fr/*.tex` **et les PDF aplatis**
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) : quels passages cette story rend-elle faux —
   suppression d'une facture validée, immutabilité, avoir comme seule correction, numérotation ?
   ⛔ En LaTeX le souligné s'écrit `\_`. Cherche aussi dans `README.md`, `website/` et
   `docs/api-external.md` un site que la spec **ne nomme pas**.
8. **Tests, décomptes, périmètre et découpage.** Chaque test annoncé tranche-t-il ? Les nombres
   (sept empêchements, quatre locales, douze AC…) sont-ils cohérents avec leur ventilation ? La
   story franchit-elle le seuil de la *Règle de splitting* (plus de 5 modules) une fois
   l'inventaire réel fait ?

## Ce que tu rends

- **Les findings**, chacun avec : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact,
  **la commande ou l'extrait qui l'établit**, et ce qu'il faut changer. Pour un scénario, montre que
  **son état de départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien. Ne qualifie jamais de « vérifié » un point que tu n'as pas
  exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une
base persistante** — nommément `scripts/prepare-release.sh` (il bumpe les versions Cargo),
`scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/`checkout`/`add`/
`stash`/`reset`/`rebase`, `sqlx migrate` sur `kesh` ou `kesh_e2e`, `cargo test`/`cargo nextest`.
Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` et `cargo check` sont
autorisés.
