# Story 15.4 : Journaux personnalisables

## Status

backlog

⚠️ **Cette story n'existait qu'en ligne de suivi.** Elle occupait une entrée de `sprint-status.yaml` **sans aucune spécification** — ni dans `epics.md`, ni ailleurs. Trou de traçabilité relevé le 2026-08-12 en réponse à une demande d'inventaire des epics restants, et comblé ici.

⚠️ **Lire la § « Recommandation de découpage » avant de lancer `validate`.** L'analyse conclut que cette story est probablement trop large pour converger d'un seul tenant.

## Story

**As a** comptable ou fiduciaire dont le plan de journaux ne se réduit pas aux cinq journaux standards,
**I want** créer, renommer et organiser mes propres journaux comptables,
**so that** mes écritures se rangent selon **ma** tenue de comptes, et non selon une liste figée dans le code.

Dernière story de l'**Epic 15 « Justificatifs, Lettrage & Compléments »**.

## Contexte — ce qu'est un journal aujourd'hui, vérifié dans le code

**Un enum Rust fermé, dupliqué, doublé de deux contraintes SQL.**

| Où | Quoi |
|---|---|
| `kesh-core/src/accounting` | `enum Journal` — 5 variantes : `Achats`, `Ventes`, `Banque`, `Caisse`, `OD` |
| `kesh-db/src/entities/journal_entry.rs:33` | **Le même enum, dupliqué**, avec les traits SQLx. Son doc-comment le dit : *« miroir de `kesh_core::accounting::Journal` […] toute modification doit être synchronisée avec kesh-core ET la contrainte DB »* |
| `journal_entries.journal` | `VARCHAR(10) NOT NULL` + `CHECK (BINARY journal IN (…))` |
| `company_invoice_settings.default_sales_journal` | `VARCHAR(10)` + **une seconde** contrainte `CHECK` identique |

**Volume à traiter**, recompté :

| Module | Sites `Journal::` |
|---|---|
| `kesh-db` | 39 |
| `kesh-report` | 23 |
| `kesh-core` | 10 |
| `kesh-api` | 7 |
| `frontend` | 287 mentions de `journal` (à trier : toutes ne concernent pas l'énumération) |

### Le point qui gouverne toute la conception

⚠️ **Les cinq journaux ne sont pas une liste arbitraire : le code en dépend SÉMANTIQUEMENT.**

- `opening_balances.rs:333` code en dur `CoreJournal::OD` pour l'écriture d'ouverture ;
- la validation d'une facture poste au journal des **ventes** ;
- les rapports énumèrent `Ventes`, `Banque`, `OD` (`kesh-report/src/pdf.rs:1540`, `:1969`) ;
- `company_invoice_settings.default_sales_journal` désigne **le** journal des ventes.

**Ouvrir la liste sans rien d'autre casserait tout cela** : le code ne saurait plus lequel des journaux de l'utilisateur est celui des ventes. Une story qui se contenterait de remplacer l'enum par une table libre livrerait un logiciel qui ne sait plus valider une facture.

## Le précédent du dépôt — et il est exact

**L'Epic 14 a résolu ce problème même, pour les comptes.** Sa formule est inscrite dans la feuille de route : *« rôles de comptes explicites — le rôle d'un compte ne se déduit plus de son numéro »*.

Le patron, dans `20260722000001_accounts_role_postable.sql` :

- une colonne `role VARCHAR(32) NULL` à **liste fermée** par `CHECK BINARY` ;
- une colonne **générée** `singleton_role` et un `UNIQUE (company_id, singleton_role)` pour les rôles dont il ne peut exister qu'un par société — exploitant la convention SQL selon laquelle « `NULL` n'est jamais égal à `NULL` » ;
- le code résout **par rôle**, jamais par numéro ni par libellé.

**C'est exactement la transformation qu'il faut appliquer aux journaux**, et le fait qu'elle ait déjà été conduite une fois dans ce dépôt en réduit beaucoup le risque : le patron est éprouvé, ses pièges sont documentés, et sa migration sert de modèle.

## Décisions

**D1 — Une table `journals` par société, et le code résout PAR RÔLE.** Chaque journal porte un libellé libre et, facultativement, un **rôle** issu d'une liste fermée (`sales`, `purchases`, `bank`, `cash`, `misc`). Tout site qui dépend aujourd'hui de `Journal::Ventes` interrogera *« le journal de rôle `sales` de cette société »*.

**D2 — Un rôle est unique par société, et cela se tient par la base.** Colonne générée + `UNIQUE (company_id, <rôle>_uniq)`, sur le patron des comptes. Un journal **sans** rôle est un journal purement utilisateur, et il peut y en avoir autant qu'on veut.

**D3 — Les cinq journaux standards sont créés pour chaque société, avec leur rôle.** Ils sont **renommables** — c'est l'objet de la story — mais leur **rôle** ne se supprime pas : le logiciel doit toujours pouvoir poster une vente quelque part.

**D4 — Un journal ne se supprime pas tant qu'une écriture le référence.** Il s'**archive**, comme les comptes et les contacts. Le dépôt a déjà ce vocabulaire et cette mécanique.

**D5 — La migration est BREAKING : procédure P3 en entier.** Elle transforme `journal_entries.journal` d'un `VARCHAR` contraint en référence, et retire deux contraintes `CHECK`. Donc : bump de `kesh_version_min_required` **en dernière instruction** de la migration (P2), bump de version Cargo de **tous** les crates dans le **même commit** (P2-bis), et **gate runtime** complet — `admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install`.

**D6 — Le backfill écrit des données : garde-fou P7.** Créer les cinq journaux de chaque société existante et rattacher les écritures est un `INSERT` suivi d'un `UPDATE`. À trier au registre `POST_RESTORE_BACKFILLS` ou à exempter **avec justification écrite** — sans quoi restaurer une sauvegarde antérieure rouvrirait le trou définitivement.

**D7 — L'enum dupliqué disparaît, et c'est un gain à noter.** Aujourd'hui `kesh-core` et `kesh-db` maintiennent **deux** enums identiques à la main, plus deux contraintes SQL : quatre sources de vérité pour une seule notion. La table les remplace toutes. Ce qui subsiste — la liste fermée des **rôles** — n'a plus qu'un seul site faisant foi.

## Recommandation de découpage

⚠️ **À arbitrer avant de lancer `validate`.**

La § *Règle de splitting préventif* du `CLAUDE.md` déclenche sur « plus de 5 modules ». Cette story en touche **exactement cinq** — `kesh-core`, `kesh-db`, `kesh-api`, `kesh-report`, `frontend` — donc **au seuil, pas au-delà**. Je ne prétends pas que la règle est mécaniquement franchie.

**Mais trois éléments plaident pour découper quand même :**

1. **79 sites `Journal::` côté Rust**, à convertir d'un enum vers une résolution par rôle.
2. **Une migration breaking** avec sa procédure P3 et son backfill P7 — à elle seule, un sujet de story.
3. **Le précédent direct** : la même transformation, pour les comptes, a été découpée en **trois** stories (14-3a socle, 14-3b consommateurs, 14-3c fonds propres par rôle) — et elles ont convergé.

**Découpage proposé, sur ce précédent :**

| Sous-story | Contenu | Visible par l'utilisateur |
|---|---|---|
| **15-4a — socle** | Table `journals`, rôles, migration breaking, backfill, résolution par rôle dans le backend. **Comportement inchangé de bout en bout.** | non |
| **15-4b — consommateurs** | Les 79 sites Rust, les rapports, les exports. Mécanique, revue au fichier. | non |
| **15-4c — surface** | Écran de gestion : créer, renommer, archiver, réordonner. Sélecteur de journal à la saisie d'écriture. | **oui** |

L'intérêt de ce découpage n'est pas le confort : c'est que **15-4a et 15-4b ne changent rien pour l'utilisateur**, donc leur gate est un pur test de non-régression, mesurable sans ambiguïté. Le risque se concentre alors sur 15-4c, qui est la plus petite.

## Acceptance Criteria — pour la story entière

*Si le découpage est retenu, ces critères se répartissent entre les trois sous-stories.*

**AC1 — Les cinq journaux standards existent pour chaque société, avec leur rôle.**
*Preuve* : après migration, chaque société porte cinq journaux, un par rôle ; les écritures existantes sont rattachées au journal correspondant à leur ancien libellé, **sans exception**.
⚠️ Le test doit compter les écritures **avant et après** : une écriture orpheline ne se voit pas, elle disparaît simplement des rapports.

**AC2 — Le code ne connaît plus les journaux par leur nom.**
*Preuve* : `grep -rn "Journal::Ventes\|Journal::OD\|Journal::Banque" crates/ --include=*.rs` ne rend **aucun site de production**. La validation de facture et l'écriture d'ouverture résolvent par rôle.

**AC3 — Renommer un journal ne casse rien.**
*Preuve* : renommer le journal de rôle `sales` en « Facturation », puis valider une facture — elle se poste au bon journal, et le rapport l'affiche sous son nouveau nom.
⚠️ **C'est l'assertion centrale de la story** : sa mutation — résoudre par libellé au lieu du rôle — doit la faire tomber.

**AC4 — Un journal utilisateur, sans rôle, fonctionne comme les autres.**
*Preuve* : créer un journal « Salaires », y poster une écriture manuelle, la retrouver au grand livre et dans les exports.

**AC5 — Un rôle reste unique par société.**
*Preuve* : tenter de donner le rôle `sales` à deux journaux → refus par la base, remonté en erreur métier avec son code propre.

**AC6 — Un journal référencé ne se supprime pas.**
*Preuve* : archivage accepté, suppression refusée avec un code d'erreur dédié — pas un `500`.

**AC7 — Les rapports et exports suivent.**
*Preuve* : grand livre, journal, exports PDF et CSV affichent les libellés de l'utilisateur. ⚠️ `kesh-report/src/pdf.rs` énumère aujourd'hui les journaux **en dur** à deux endroits (`:1540` et `:1969`) — **les deux, ou aucun**.

**AC8 — La procédure P3 est appliquée en entier.**
*Preuve* : `min_required` bumpé, les 10 crates à la même version dans le même commit, ligne ajoutée à `docs/migrations-idempotence-audit.md` avec ses **cinq** compteurs recomptés depuis le tableau, et le **gate runtime** vert.

**AC9 — La documentation dit ce que le logiciel fait.**
*Preuve* : manuel utilisateur — comment créer et renommer un journal, et pourquoi les cinq rôles ne se suppriment pas. CHANGELOG dans les mots de l'utilisateur.

## Dev Notes

### Les pièges que le précédent des comptes a déjà révélés

Lire `20260722000001_accounts_role_postable.sql` **avant** d'écrire la migration, et les Change Logs des stories **14-3a / 14-3b / 14-3c** avant de découper. Ce que ce lot a coûté est écrit ; le refaire serait dommage.

⚠️ **P6 — couplage positionnel des migrations.** Toute migration ajoutée oblige à `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et à **inspecter chaque site**. Précédent : sur la Story 16-1a, un test s'est mis à **passer à vide** — un test muet ne signale rien.

⚠️ **P8 — une migration appliquée ne se modifie plus**, pas même un commentaire : le checksum `sqlx` change et le backend refuse de démarrer sur toute base l'ayant appliquée.

### Ce qui ne doit pas être « simplifié » au passage

Le `CHECK BINARY` des rôles est **binaire** à dessein, comme celui des comptes : la comparaison de chaîne insensible à la casse laisserait passer `Sales` pour `sales`.

### Conventions de test

Mutations **jouées, pas raisonnées**. Pour AC3, la mutation est explicite : résoudre par libellé doit faire tomber le test.
Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites.
Les décomptes se **recomptent depuis la source**, avec leur **périmètre de mesure déclaré**.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 15 (numéroté **14** dans ce fichier, resté à l'ancienne numérotation).
- Stories **14-3a**, **14-3b**, **14-3c** — le précédent des rôles de comptes, et son découpage en trois.
- `CLAUDE.md` — § *Migration breaking policy* (P2, P2-bis, P3, P5, P6, P7, P8), § *Règle de splitting préventif*.

## Questions ouvertes

1. **Le découpage** — à arbitrer avant `validate`. Cf. la recommandation ci-dessus.
2. **Le périmètre par société** — les journaux sont-ils par société, ou globaux à l'instance ? Le dépôt est aujourd'hui mono-société par instance, mais tout y est scopé par `company_id` ; suivre cette convention plutôt que l'exception.
3. **La numérotation des écritures** — certaines tenues numérotent les écritures **par journal** (`VE-2026-0001`, `AC-2026-0001`) et non globalement. Est-ce dans le périmètre, ou une story à part ? **À trancher explicitement**, faute de quoi une passe de revue le rouvrira : c'est le prolongement naturel de « journaux personnalisables », et ce n'est pas le même travail.
4. **Les 287 mentions de `journal` côté frontend** — à trier. Toutes ne concernent pas l'énumération ; le chiffre est un majorant, pas une charge.
