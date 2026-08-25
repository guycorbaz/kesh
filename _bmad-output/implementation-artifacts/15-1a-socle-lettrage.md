# Story 15.1a : Le socle du lettrage — la marque, sa portée, son cycle de vie

## Status

draft

## Story

**As a** indépendant ou fiduciaire qui tient ses comptes dans Kesh,
**I want** qu'une créance et son règlement puissent être marqués comme se soldant l'un
l'autre, et que cette marque survive à tout ce que l'application fait par ailleurs à une
écriture,
**so that** ce que le logiciel affirme sur un solde reste vrai le lendemain.

Première des trois sous-stories issues du **split de la 15-1** (passe 3 de `validate`,
2026-08-25). Couvre la part **FR85/FR86** qui touche la persistance et les gardes.

⚠️ **Socle : les deux autres sous-stories la supposent faite.** 15-1b lit la marque pour
bâtir la vue ; 15-1c la pose depuis un écran. Aucune des deux n'est spécifiable tant que le
porteur de la marque et son cycle de vie ne sont pas tranchés.

## Pourquoi cette story existe séparément

La 15-1 d'origine a subi trois passes de `validate` sans converger : `HIGH → MEDIUM → HIGH`.
Les quatre HIGH de la passe 3 avaient tous la même origine — **aucune passe n'avait suivi les
chemins de code qui écrivent, effacent ou soldent les lignes que le lettrage prétend
porter**. Deux d'entre eux tombent dans cette story, et ils suffisent à la justifier :

| | ce que le code fait aujourd'hui | preuve |
|---|---|---|
| **modifier** une écriture | `update` **supprime toutes ses lignes** puis les réinsère avec de nouveaux `id` | `journal_entries.rs:981`, dans `update` (l. 805) |
| **supprimer** une écriture | deux chemins y mènent, pas un : la route, **et** `invoices::delete` via `delete_in_tx` | `invoices.rs:1338` |

Une marque posée sur une ligne sans traiter ces deux chemins est **perdue au premier
enregistrement**, en laissant sa contrepartie marquée — donc réputée soldée. La facture
quitte la vue des ouverts **en restant impayée** : le défaut est muet, et il fausse le solde
sans rien signaler.

## Décisions tranchées par le Project Lead — 2026-08-25

Les deux décisions que les passes 1 et 3 avaient laissées ouvertes sont **arbitrées**. Elles
se figent l'une et l'autre dans la migration (P8 interdit de revenir sur un fichier appliqué),
et elles conditionnent le DDL de T1.

### Le porteur — une table `letterings`

```sql
CREATE TABLE letterings (
    id          BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id  BIGINT NOT NULL,
    seq         BIGINT NOT NULL
                COMMENT 'Rang du lettrage dans la société — SEUL support du compteur ; `code` en est la projection',
    code        VARCHAR(16) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin NOT NULL
                COMMENT 'Projection base 26 bijective de seq (A, B, … Z, AA) — engendré, jamais saisi',
    created_at  DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    created_by  BIGINT NOT NULL,
    CONSTRAINT fk_letterings_company FOREIGN KEY (company_id)
        REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_letterings_user FOREIGN KEY (created_by)
        REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT uq_letterings_company_seq  UNIQUE (company_id, seq),
    CONSTRAINT uq_letterings_company_code UNIQUE (company_id, code),
    CONSTRAINT chk_letterings_seq_positive CHECK (seq > 0),
    CONSTRAINT chk_letterings_code_nonempty CHECK (CHAR_LENGTH(code) > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

ALTER TABLE journal_entry_lines
    ADD COLUMN lettering_id BIGINT NULL
        COMMENT 'Marque de lettrage — deux lignes soldées partagent la même valeur',
    ADD CONSTRAINT fk_jel_lettering FOREIGN KEY (lettering_id)
        REFERENCES letterings(id) ON DELETE RESTRICT;

CREATE INDEX idx_jel_lettering ON journal_entry_lines (lettering_id);
```

⚠️ **L'index se déclare à part, et ce n'est pas une coquetterie** *(P6-8)* : le seul précédent
d'`ALTER TABLE journal_entry_lines` du dépôt
(`20260702000001_projects_analytics.sql:38-42`) fait `ADD COLUMN` + `ADD CONSTRAINT`, **puis**
un `CREATE INDEX` séparé. Les deux formes sont valides en MariaDB — mais P8 rendra ce fichier
intouchable, et diverger d'un précédent qu'on ne pourra plus aligner coûte pour rien.

⚠️ **Les deux `CHECK` ne sont pas décoratifs** : `seq` est le **porteur unique de la justesse
du compteur**, et les tables voisines en portent toutes (`chk_journal_entries_entry_number_positive`,
`contact_persons`, `invoice_reminders`, `api_keys`).

⚠️ **`utf8mb4_bin` sur `code` est le bon choix** — le code est **engendré**, l'unicité doit
être octet-exacte. Le commentaire ne dit donc pas « saisie » : une recherche `WHERE code = ?`
sur une entrée en minuscules ne rendrait rien, et c'est à 15-1c d'imposer la mise en
majuscules si elle ouvre un champ de recherche.

⛔ **La colonne `seq` n'est pas un confort : sans elle, le compteur est FAUX.** *(P5-1,
CRITICAL, relevé en passe 5.)* Le gap lock d'`entry_number` porte sur un **entier** ; sur du
texte, `MAX()` trie **lexicographiquement**, et la séquence bijective n'est pas ordonnée
ainsi — `MAX('Z', 'AA', … , 'AZ')` rend **`Z`**, quand la suite attend `BA`. **Le compteur
repartirait en boucle après la vingt-sixième lettre**, et `uq_letterings_company_code` ferait
alors échouer le lettrage — bruyamment, mais à chaque tentative.

**Le mécanisme, en trois temps** : le gap lock s'applique à `seq`
(`SELECT COALESCE(MAX(seq), 0) + 1 … WHERE company_id = ? FOR UPDATE`) — **transposition
exacte** du pattern d'`entry_number` ; `code` est **calculé en Rust** depuis `seq` (base 26
bijective) ; les deux contraintes d'unicité tiennent le filet. ⚠️ **La conversion est une
fonction pure : elle se teste en `kesh-core`, sans base** — et ses cas limites sont `26 → Z`,
`27 → AA`, `52 → AZ`, `53 → BA`, `702 → ZZ`, `703 → AAA`.

**Les comportements de FK sont tranchés** *(P5-3)*, conformes aux conventions du dépôt
(`companies(id) ON DELETE RESTRICT` partout ; `users(id) ON DELETE RESTRICT` pour un auteur
d'action, cf. `bank_imports.imported_by_user_id`) :

- `journal_entry_lines.lettering_id` → `letterings(id)` **`ON DELETE RESTRICT`** : on ne
  supprime pas une entrée encore référencée. ⚠️ `SET NULL` ferait disparaître une marque des
  deux côtés **sans trace**, exactement le défaut muet qu'AC8 ferme ailleurs.

⛔ **`letterings` est APPEND-ONLY : délettrer ne supprime PAS la ligne d'entrée**, il met les
deux `lettering_id` à `NULL` et laisse l'entrée en place. *(P6-5.)* Sans cette règle, le
compteur **recule** — `MAX(seq)` retombe d'un cran — et **le code est RÉÉMIS** : l'utilisateur
qui a imprimé ou dicté un `F` le retrouverait plus tard sur une paire sans rapport. L'audit
d'AC13 enregistrerait alors `created F` / `removed F` / `created F` sans que rien ne
distingue **deux lettrages différents portant la même désignation** — précisément ce contre
quoi AC13 existe.

⚠️ **Conséquence du choix append-only : le `RESTRICT` n'est jamais dans le chemin nominal.**
Il reste comme garde-fou contre une suppression qu'aucun code ne doit tenter.
- `letterings.company_id` → `companies(id)` **`RESTRICT`**, `letterings.created_by` →
  `users(id)` **`RESTRICT`** : l'auteur d'un lettrage reste identifiable, ce dont AC13 dépend.

**Pourquoi, et ce n'est pas un choix d'élégance** — l'alternative écartée était une colonne
`lettering_code` nullable sur `journal_entry_lines` :

- ⚠️ **`journal_entry_lines` ne porte aucun `company_id`** (zéro occurrence dans les
  migrations ; le scoping y passe par jointure, `journal_entries.rs:1226` le documente). Le
  seul pattern de génération scopée sous concurrence éprouvé dans le dépôt — le gap lock
  d'`entry_number`, `SELECT COALESCE(MAX(…))+1 … WHERE company_id = ? … FOR UPDATE`
  (`journal_entries.rs:232`) — verrouille **directement** une `company_id` de la table cible.
  `letterings` ayant la sienne, **il se transpose à l'identique**.
- ⚠️ **Et il garde un filet au niveau du schéma** : si le verrouillage faiblit,
  `UNIQUE (company_id, code)` rattrape la violation. Sur une colonne de
  `journal_entry_lines`, **aucune contrainte n'était possible** — une contrainte d'unicité y
  aurait d'ailleurs **interdit AC1**, puisque deux lignes lettrées ensemble portent le *même*
  code. Une erreur de verrouillage n'y aurait été détectée par **rien**.
- **`created_at` et `created_by` viennent gratuitement**, et AC13 en a besoin. La colonne
  n'avait nulle part où les mettre.

⚠️ **Conséquence à ne pas oublier** : `reset_demo` a une liste de `DELETE` **explicite** —
`letterings` doit y être ajoutée (cf. T2).

### Le format — une séquence alphabétique par société

**`A`, `B`, … `Z`, `AA`, `AB`, …** — base 26 bijective, engendrée par le serveur, **portée
société** (chaque société repart de `A`).

C'est le format des logiciels comptables suisses, Bexio compris — dont **D6 fait déjà le
modèle de l'écran**. Il est court, se lit à l'œil sur une ligne d'écriture et **se dicte au
téléphone**, ce qui compte pour une fiduciaire qui appelle son client à propos d'une facture.

⚠️ **Écarté : un identifiant sans compteur** (ULID, UUID). Il supprimerait toute contention,
mais 15-1c exige que la marque soit **visible sur la ligne** — une chaîne de 26 caractères y
est inutilisable, et indictable.

⚠️ **`VARCHAR(16)` laisse très large** : la séquence n'atteint `AAA` qu'après 702 lettrages
dans une même société, et seize caractères en admettent bien davantage. Le choix d'un
`VARCHAR` plutôt que d'un entier est ce qui rend la séquence alphabétique possible **sans
conversion à l'affichage**.

## Décisions héritées de la 15-1

### D1 — La marque porte sur la LIGNE, pas sur l'écriture

⚠️ **Pour une raison de fond : le lettrage porte sur un COMPTE.** Une écriture de vente
touche le compte client, un compte de produit et un compte de TVA. « Cette écriture est
lettrée » n'a aucun sens comptable — ce qui est soldé, c'est la **ligne au compte client**.
Une marque posée sur l'écriture rendrait la vue « ce qui reste ouvert **sur le compte
1100** » incalculable sans retrouver la ligne concernée.

### D2 — Un lettrage, une facture, un règlement

Décision de kickoff de l'Epic 15, déclarée **normative pour la 15-1** : ni paiement
**partiel**, ni règlement **groupé**. *(Reportée ici en passe 6 — **D2 n'était nommée par
aucune des trois fiches issues du split**, et c'est la septième récidive du geste « une
décision que ne porte aucun critère ».)*

⚠️ **C'est elle qui fonde AC12** : l'égalité des montants n'est pas une restriction technique
mais la conséquence directe de la borne — tant que le partiel est hors périmètre, lettrer
1000 avec 300 prétendrait qu'une créance est soldée.

⚠️ **Et c'est elle qui justifie la forme « PAIRE » de toute la story** — deux lignes, jamais
plus. *(`epics.md` écrit « deux **ou plusieurs** écritures » : la restriction à deux est
correcte au regard de D2, mais rien ne l'y reliait.)*

### D3 — Lettrer sur un exercice clôturé : OUI. Délettrer : NON

L'asymétrie est délibérée et elle est **la** décision comptable de cette story : lettrer ne
modifie aucun montant, donc n'altère pas un exercice clos ; délettrer rouvrirait une créance
que le bilan porte déjà.

⚠️ **C'est D3, et non « clore quand il n'y a plus d'écritures », qui rend le lettrage à
cheval possible** — une créance ouverte au 31.12 est **normale**, elle figure au bilan.
Attendre que tout soit réglé reviendrait à ne jamais clôturer.

*Note vérifiée en passe 3 : `fiscal_years::reopen` existe (Story 14-2 — Admin, motif
obligatoire, tracé à l'audit). Il ne contredit pas D3, il en est la **soupape** : ce que le
refus de délettrage rend impossible reste atteignable par une réouverture explicite et
tracée.*

## Critères d'acceptation

**AC1** (porte **D1**) — La marque de lettrage se pose sur une **ligne d'écriture**, jamais
sur l'écriture entière. Deux lignes portant la même marque sont lettrées ensemble.

**AC2** — **La portée de la marque est la société, et elle est engendrée par le serveur.**
Deux sociétés doivent pouvoir porter la même valeur sans se voir ni se percuter. ⚠️ **Un
compteur non scopé serait un défaut de multi-tenant** — le dépôt en a déjà payé un (KF-002),
et c'est exactement la classe d'erreur qu'une spécification muette invite à commettre.

**AC3** — **L'unicité tient sous concurrence.** Deux lettrages simultanés dans la même
société ne reçoivent jamais le même code. Un compteur lu puis incrémenté hors transaction ne
le garantit pas. **Le mécanisme est en DEUX temps, et le premier est celui qu'on oublie :**

> **(1)** un **sentinel** `SELECT id FROM companies WHERE id = ? FOR UPDATE` en tête de la
> transaction — **Pattern 5, idiome du dépôt** (`journal_entries.rs:396` le nomme ainsi ;
> `bank_accounts`, `projects`, `invoices` l'emploient) ;
> **(2)** puis `SELECT COALESCE(MAX(seq), 0) + 1 FROM letterings WHERE company_id = ?`.

⛔ **Sans (1), rien ne sérialise, et le dépôt porte déjà le diagnostic écrit.** *(P6-1.)*
`email_templates.rs:16-23` l'énonce mot pour mot : *« un `SELECT ... FOR UPDATE` sur une ligne
absente prend un gap lock InnoDB ; or un gap lock **n'empêche PAS** une autre transaction de
tenir aussi son propre gap lock compatible sur la même lacune […] risquent donc de
deadlocker »*.

⚠️ **Le pattern d'`entry_number` n'a JAMAIS reposé sur son `MAX(…) FOR UPDATE`** : ce qui le
sérialise est le **verrou de ligne pris en amont** — `SELECT fiscal_years … FOR UPDATE`
(`journal_entries.rs:191`). En reprendre la formule sans son amont, c'est copier ce qui se
voit et laisser ce qui fait fonctionner. **`letterings` n'a aucune ligne préexistante à
verrouiller** pour une société encore vierge de lettrage — et c'est justement la fixture
naturelle du test.

⛔ **Et le filet d'unicité ne rattrape PAS ce mode d'échec** : un deadlock est un **1213**,
pas un **1062**. Aucune contrainte ne le voit. Le dépôt a un `retry_on_deadlock`
(`crates/kesh-db/src/retry.rs`) — mais il n'a **aucun site d'appel** : la parade existe et
n'est branchée nulle part.

⚠️ **Respecter l'ordre de verrou global du dépôt** — `companies → projects → fiscal_years`
(`journal_entries.rs:391`). La route de lettrage verrouillant aussi des écritures, une
inversion ABBA est un risque réel.

⛔ **Le gap lock ne doit JAMAIS porter sur `code`** *(P5-1)* : `MAX()` sur du texte trie
**lexicographiquement**, et `MAX('Z','AA')` rend **`Z`** — la collision survient à la
**vingt-huitième** lettre *(P6-7 : après `{A…Z}` le successeur `AA` est juste ; c'est après
`{A…Z, AA}` que `MAX` rend encore `Z`, donc `AA` une seconde fois)*.

⚠️ **Un test de concurrence est exigé, et la spec dit ce qu'il doit CONSTATER** — sans quoi
son auteur, voyant un deadlock intermittent, affaiblirait l'assertion pour le faire passer
*(le dépôt a déjà une KF de ce genre, KF-038)* : **sur une société VIERGE de lettrage**, deux
lettrages simultanés doivent produire **deux `seq` et deux `code` distincts**, tous deux en
succès. Ni un deadlock, ni un échec de contrainte.

⛔ **Moyen EXCLU, et il faut savoir pourquoi il l'est** : porter la contrainte d'unicité sur
`journal_entry_lines` — la colonne `company_id` n'y existe pas, et une contrainte d'unicité
sur le code y **interdirait AC1**, deux lignes lettrées ensemble portant le *même* code.
C'est l'une des raisons du choix de la table.

**AC4** — Le lettrage est **refusé** si les deux lignes ne portent pas sur le **même
compte**, ou si leurs sens (débit/crédit) ne s'opposent pas.

**AC5** (porte **D3**) — Le lettrage est **autorisé** même si l'un des exercices concernés
est clôturé, y compris à cheval sur deux exercices.

**AC6** (porte **D3**) — Le délettrage est possible tant que **les deux** exercices sont
ouverts, et **refusé** dès que l'un est clôturé.

⛔ **Le délettrage prend le MÊME sentinel `companies FOR UPDATE` que le lettrage** *(P7-1)*,
et pour une raison que cette story est la première du dépôt à rencontrer : **elle doit
vérifier DEUX exercices dans une seule transaction.** Tout le code existant n'en verrouille
qu'un — une écriture n'appartient qu'à un exercice — et le lettrage à cheval qu'AC5 organise
brise cette hypothèse.

Les deux issues qu'un développeur prendrait sans cette clause sont **toutes deux mauvaises** :

- **verrouiller les deux `fiscal_years` dans l'ordre où les lignes arrivent** → deux
  délettrages croisés (L1 sur 2024+2025, L2 sur 2025+2024) se bloquent en **ABBA**. Et le
  `retry_on_deadlock` du dépôt n'ayant **aucun site d'appel**, le 1213 remonte tel quel à
  l'utilisateur ;
- **ne pas verrouiller du tout**, pour éviter ce risque → le contrôle « les deux exercices
  sont ouverts » n'est plus sérialisé avec `fiscal_years::close`. Une clôture concurrente
  commit pendant la fenêtre, le délettrage commit après en croyant l'exercice ouvert, et
  **D3 est violée** — la décision comptable centrale de cette story.

⚠️ **Le sentinel les ferme toutes les deux** : il sérialise tout le trafic de lettrage et de
délettrage d'une société, ce qui rend l'ordre des verrous suivants **sans importance**. C'est
l'idiome écrit du dépôt — `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, **Pattern 5**, cité par
`projects.rs:75-78` : *« verrou sentinelle `companies` une seule fois, PUIS `FOR UPDATE` sur
les lignes — évite l'inversion ABBA »*. Le coût est négligeable au volume d'un lettrage.

**AC7** — ⚠️ **La marque survit à la MODIFICATION d'une écriture, ou la modification est
refusée.** *(Relevé en passe 3, P3-1.)* `update` fait aujourd'hui `DELETE FROM
journal_entry_lines` puis réinsère : une implémentation naïve **perd la marque en silence** et
laisse la contrepartie marquée seule, donc réputée soldée.

**TRANCHÉ**, et **re-tranché en passe 3** *(la conduite retenue en passe 2 rouvrait le trou
que cette story existe pour fermer — voir le Change Log)* :

> **(i)** La modification qui **change les lignes** d'une écriture dont une ligne est lettrée
> est **REFUSÉE**, avec un message qui nomme la cause.
> **(ii)** La modification du seul **en-tête** — date, journal, libellé — est **autorisée**,
> **et `update` NE TOUCHE ALORS PAS AUX LIGNES.**

⛔ **La clause (ii) n'est pas un confort : sans elle, (i) ne protège rien.** *(P3-1,
CRITICAL.)* Le `DELETE FROM journal_entry_lines` de `update` (`journal_entries.rs:981`) n'est
gardé que par le court-circuit **no-op complet** — en-tête **et** lignes identiques
(`is_no_op_change`, l. 972). Une modification d'en-tête seul le franchit **systématiquement** :
les lignes sont effacées et réinsérées sans marque, tandis que la contrepartie garde la
sienne. **C'est mot pour mot le mode d'échec décrit au § *Pourquoi cette story existe
séparément*** — réintroduit par le critère censé le fermer.

⚠️ **La condition de la garde est exactement ce qui rend le `DELETE`/`INSERT` inutile** :
quand le comparateur dit les lignes inchangées, il n'y a rien à réécrire. Créer le chemin
« en-tête seul » dans `update` ferme donc P3-1 **et** rend sans objet l'objection de
préservation par position.

**Ce qui coûterait un refus global, et il faut le dire juste** *(P3-2 — la passe 2 s'appuyait
ici sur un fait FAUX)* : une écriture sur exercice **clos** est **déjà** immuable
aujourd'hui, lettrage ou non — `update` la refuse à l'Étape 2 (`journal_entries.rs:892-894`,
`DbError::FiscalYearClosed`). Le cas réel est plus étroit : une écriture sur exercice
**ouvert**, lettrée avec une contrepartie sur exercice **clos** (AC5 le permet — c'est le
lettrage à cheval). Là seulement, AC6 refusant le délettrage, un refus global figerait le
libellé. Quand les deux exercices sont ouverts, il ne coûte qu'un aller-retour.

⚠️ **Le comparateur de lignes existe déjà** : `is_no_op_change` (`journal_entries.rs:782`)
compare l'en-tête **puis** les lignes. C'est sa **seconde moitié seulement** qu'il faut
extraire — la fonction entière retourne `false` dès que l'en-tête diffère, **sans regarder les
lignes**, ce qui donnerait l'exact contraire de la conduite ci-dessus. *(P3-9.)*

⚠️ **Une réserve à porter avec l'extraction** *(P3-9)* : le comparateur teste
`b.project_id == c.project_id` alors que l'`INSERT` écrit `line.project_id.or(updated.project_id)`.
Il n'est exact aujourd'hui que parce que les deux sites de la route passent
`project_id: None` au niveau écriture. Un futur appelant fournissant un projet au niveau
document ferait rapporter un faux changement de lignes — donc **refuser une modification qui
n'en est pas une**.

⚠️ **Corollaire, qui règle la renumérotation sans clause dédiée** : réordonner les lignes
**est** un changement de lignes pour un comparateur positionnel. Une renumérotation
d'écriture lettrée tombe donc sous (i).

**AC8** — ⚠️ **Une écriture dont une ligne est lettrée ne se supprime pas en laissant une
marque orpheline**, et **la garde se pose au point de passage des DEUX chemins**. *(Relevé
en passe 3, P3-2.)* La suppression est **refusée**, avec un message qui nomme la cause —
*« cette écriture est lettrée ; délettrez-la d'abord »*.

⛔ **Le site de la garde n'est pas la route.** `invoices::delete` supprime une facture
`validated` **avec son écriture**, par `journal_entries::delete_in_tx` (`kesh-db/src/repositories/invoices.rs:1338`),
et **aucune de ses trois gardes** (payée, créditée par un avoir, historique de rappels) ne
regarde le lettrage. Poser la garde dans le handler HTTP la laisse contournable par le
chemin le plus courant. Le point de passage unique est **`delete_in_tx`**, où la garde
d'exercice clos est **déjà** posée (`invoices.rs:1235` le documente).

⚠️ **Une facture réglée en espèces et lettrée passe la première garde** : elle n'est pas
`paid_at`, puisque c'est précisément le cas que le lettrage existe pour couvrir.

**AC9** — Le délettrage automatique en cascade est **écarté** : il contournerait AC6 par un
chemin détourné — on obtiendrait sur un exercice clos, en supprimant ou modifiant une
écriture, ce que le délettrage direct interdit. Il reste ouvert comme évolution, **à
condition** d'être borné aux exercices ouverts.

**AC10** — ⚠️ **Le lettrage est REFUSÉ si l'une des deux lignes porte déjà une marque.**
*(Relevé en passe 1 : aucun critère ne l'interdisait.)* Sans cette garde, ré-apparier une
ligne déjà lettrée **écrase sa marque** et laisse son ancien partenaire **seul avec
l'ancienne** — réputé soldé à vie, sans contrepartie, et **personne n'est prévenu**.

C'est la même classe de défaut muet qu'AC7 et AC8 ferment sur les chemins d'écriture, par une
porte que le lettrage ouvre lui-même : AC4 laisserait passer le cas, puisque le compte et les
sens sont bien ceux qu'il exige. **Délettrer d'abord est le geste attendu**, et le message de
refus le dit.

**AC12** (porte **D2**) — ⚠️ **Le lettrage est REFUSÉ si les deux montants ne sont pas
égaux**, et le
message nomme la cause — *« les montants diffèrent ; le lettrage partiel n'est pas encore
géré »*. *(Rapatrié en passe 3 : la garde vivait dans **15-1c**, la story de l'écran, alors
que **la route d'écriture est ici** — T4.)*

⛔ **Le trou que ce rapatriement ferme est réel et daté.** 15-1a et 15-1b mergées, 15-1c
pas encore : un appel direct à la route apparierait une ligne de 1000 et une ligne de 300 —
même compte, sens opposés, aucune déjà lettrée, donc accepté — et la vue de 15-1b retirerait
la créance alors que **700 restent dus**. C'est le finding **F2** de la story mère, coté
**HIGH** deux fois, qui aurait survécu au découpage.

⚠️ Une garde écrite dans le moteur de proposition ne protège **jamais** l'API — même
raisonnement qu'AC11 contre l'IDOR. **Ce qui reste à 15-1c est l'arbitrage de la TOLÉRANCE**
(zéro ou cinq centimes), pas l'existence du contrôle.

**AC13** — ⚠️ **La pose et le retrait de la marque sont tracés à l'audit**
(`lettering.created`, `lettering.removed`). *(Relevé en passe 3.)* Toutes les mutations
voisines le sont — `journal_entry.created` / `.updated` / `.deleted`, `fiscal_year.closed` /
`.reopened` : la convention est uniforme dans le dépôt.

⚠️ **Et AC5 en fait une nécessité, pas une convention** : écrire une marque sur une ligne
d'un exercice **clôturé** est le seul endroit où l'absence de trace est grave — plus rien ne
pourra la défaire (AC6 refuse le délettrage), et rien ne dirait qui l'a posée ni quand. La
note de D3 s'appuie elle-même sur le fait que `fiscal_years::reopen` est *explicite et
tracée* pour justifier l'asymétrie ; le même argument commande de tracer le lettrage.

**AC11** — ⚠️ **Les deux lignes appartiennent à la société de l'appelant, et c'est
vérifié.** *(Relevé en passe 1.)* « Même compte » (AC4) garantit que les deux lignes sont de
la même société **entre elles** — `accounts.company_id` est `NOT NULL` — mais **rien ne les
rattache à l'appelant**. Un utilisateur de la société A envoyant deux identifiants de lignes
de la société B poserait la marque chez B.

⚠️ `journal_entry_lines` n'ayant pas de `company_id`, la vérification passe **par jointure**
sur `journal_entries.company_id`, comme le documente la convention du repository
(`kesh-db/src/repositories/journal_entries.rs:1226`). **C'est un IDOR**, et le dépôt en a déjà payé un (KF-002) — le
même précédent qu'AC2 invoque pour la portée du compteur.

⛔ **Le refus est INDISCERNABLE de « ligne inconnue » — un 404, et AUCUN message dédié.**
*(P6-4 : la rédaction précédente prévoyait une clé i18n « lignes d'une autre société ».)*
Nommer cette cause **révélerait l'existence** de lignes appartenant à un autre tenant : un
attaquant itérant des identifiants obtiendrait un **oracle d'existence**, et l'IDOR serait
partiellement rouvert **par le critère censé le fermer**.

C'est la convention du dépôt, écrite dans le code : *« un compte d'une autre société doit
rester **indiscernable** d'un compte inexistant (garde anti-IDOR) »*
(`kesh-api/src/routes/products.rs:343-345`), et *« inconnu/cross-company → **404** »*
(`kesh-api/src/routes/journal_entries.rs:73`).

## Tasks

- [ ] **T1** — Migration : `CREATE TABLE letterings` **et** la FK nullable sur
      `journal_entry_lines` — le DDL entier est au § *Décisions tranchées*, à reprendre tel
      quel (FK, contraintes d'unicité, `CHARACTER SET`/`COLLATE`, `COMMENT`, `ENGINE`).
      ⛔ **Régénérer le squash de schéma de test** — `scripts/regen-test-schema.sh` — juste
      après avoir écrit la migration *(P6-3)*. Il est monté par **~1100 tests**
      `#[sqlx::test]` : sans régénération, `letterings` n'existe dans **aucune** de leurs
      bases, `test_schema_guard.rs` rougit, et **tout test de lettrage échoue sur table
      inconnue**. Il **se régénère, il ne s'édite jamais**.
      ⚠️ **La conversion `seq` → `code` (base 26 bijective) est une fonction PURE** : elle va
      dans `kesh-core` et **se teste sans base**. Cas limites à couvrir nommément :
      `26 → Z`, `27 → AA`, `52 → AZ`, `53 → BA`, `702 → ZZ`, `703 → AAA`.
      ⚠️ La migration **n'écrit aucune donnée** — `CREATE TABLE` + `ADD COLUMN` nullable —
      donc **non-breaking** : pas de bump `min_required` (P1). Ligne d'audit d'idempotence
      **obligatoire** (P5, `docs/migrations-idempotence-audit.md`), et les **deux** sites du
      total plus les trois compteurs de partition se **recomptent depuis le tableau**.
      ⛔ **Ne PAS l'inscrire à `EXEMPT_MIGRATIONS` (P7)** : *(relevé en passe 3, P3-10)* le
      détecteur ne trie **que** les migrations qui écrivent des données
      (`post_restore.rs:711`) — ni un `CREATE TABLE` ni un `ADD COLUMN` n'y entre. L'y inscrire ajouterait
      du bruit à une liste dont toute la valeur tient à sa lisibilité.
      ⚠️ **P6 — le couplage positionnel** : lancer
      `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspecter chaque
      site. Le filet est *fail-loud* — `migrations_upgrade_path.rs` porte un
      `assert_eq!(total, …)` codé en dur dont le message renvoie au garde-fou P6 — mais
      **l'anticiper coûte une minute, le découvrir au bout du gate en coûte soixante**.
      *(P5-5 : le compteur de ce test bougera, la migration étant un fichier de plus ; en
      revanche **aucun backfill à fenêtre n'est en jeu**, cette migration n'écrivant pas de
      données.)*
- [ ] **T2** — Repository : poser la marque, la retirer, la lire. Gardes d'exercice
      asymétriques (AC5/AC6). **Trace d'audit** (AC13).
      ⛔ **Ajouter `letterings` à `reset_demo`** *(P3-6 — la conduite retenue le rend
      obligatoire)* : sa liste de `DELETE` est **explicite** (`kesh-seed/src/lib.rs`) et une
      table neuve n'y figure pas. Le bloc s'exécute sous `SET FOREIGN_KEY_CHECKS=0`, si bien que le
      `DELETE FROM companies` passerait **malgré la FK** et laisserait des lignes orphelines ;
      et le jour où ce drapeau serait retiré — le fichier dit que les `DELETE` explicites
      existent pour cela — `reset_demo` échouerait. *(La réfutation de passe 1, « le wipe
      efface tout ensemble », ne valait que pour la conduite écartée.)*
- [ ] **T3** — ⚠️ **Gardes sur les chemins d'écriture existants** (AC7, AC8) — c'est le cœur
      de cette story. Recenser les appelants avant d'écrire :
      `grep -rn "delete_in_tx\|DELETE FROM journal_entry_lines" crates/`.
      ⚠️ **Pour AC7, extraire la moitié « lignes » de `is_no_op_change`
      (`journal_entries.rs:782`) plutôt que d'écrire un second comparateur** — deux
      comparateurs divergents donneraient deux réponses à la question « les lignes ont-elles
      changé ? ». La garde de AC8 se pose dans `delete_in_tx`, **pas** dans le handler.
- [ ] **T4** — Routes : `POST` lettrage, `DELETE` délettrage.
      ⛔ **L'ordre d'évaluation des gardes fait partie de la garantie d'AC11** *(P7-2)* : les
      deux lignes se chargent par **une requête unique scopée `company_id = ?`** ; **moins de
      deux lignes trouvées → 404**, **avant toute** évaluation d'AC4, AC10 ou AC12.
      ⚠️ Sinon l'oracle qu'AC11 ferme **rouvre par un autre canal** : un 409 « comptes
      différents » ou « déjà lettrée » révélerait l'existence de la ligne **et un de ses
      attributs**. Le précédent du dépôt écrit cette séquence — *« Critère 1 — exister ET
      appartenir à la société »*, **puis** « Critère 2 » (`kesh-api/src/routes/products.rs:343`).
      Les fonctions voisines documentent d'ailleurs leurs étapes une par une ; celle-ci le
      doit aussi.
- [ ] **T5** — ⚠️ **Export CSV** *(relevé en passe 3 de la story MÈRE)* : le header de
      `journal_entry_lines` est **figé en dur**
      (`crates/kesh-api/src/exports/csv_tables.rs:286`) et il n'a **aucune garde
      d'exhaustivité**, contrairement à `invoices` (même fichier, l. 1031, garde #262). S'y ajoutent **deux** listes de colonnes en dur côté repository
      (`LINE_COLUMNS` l. 44 et le `SELECT` l. 1235) : en oublier une fait échouer l'export au
      runtime. Étendre la garde vaut mieux que se souvenir.
      ⛔ **TRANCHÉ : `lettering_id` entre dans la struct `JournalEntryLine`, dans
      `LINE_COLUMNS` et dans l'export CSV dès cette story** *(P7-4)*. La garde-modèle des
      `invoices` est **structurelle** — elle compare le header aux champs de la **struct**,
      pas au schéma : exposer la marque par une lecture dédiée sans toucher le type partagé
      laisserait la garde **verte tout en n'exportant jamais la marque**. Or l'export existe
      pour que l'utilisateur puisse **vérifier ce que Kesh affirme**, ce qui est l'objet même
      de cette story.
      ⛔ **Et le `.keshbackup` EST concerné** *(P6-2 — l'affirmation précédente, « pas
      concerné », n'était vraie que pour les COLONNES)* : l'**inventaire des tables** est
      **codé en dur**, `TABLES_TO_TRUNCATE` (`kesh-db/src/backup.rs:34`), ordonné enfants →
      parents et réutilisé par `restore_body` et `test_fixtures::truncate_all`. Y inscrire
      `letterings` **après `journal_entry_lines`, avant `users`/`companies`**.
      ⚠️ Le test `backup_inventory_matches_schema` compare des listes **triées** : il ne
      contrôle **pas** la position dans l'ordre FK, et un ajout en queue passerait au vert.
      ⚠️ **À dire dans le manuel** : tout `.keshbackup` produit **avant** cette migration
      devient non importable — `admin_backup/import.rs` compare l'inventaire **dans les deux
      sens**. C'est inhérent à toute table neuve, mais cela s'annonce.
- [ ] **T6** — Tests : **AC7, AC8 et AC10 en priorité** — ce sont les trois endroits où une
      implémentation plausible produit un défaut **muet**. Un test par chemin : modification
      d'écriture lettrée, suppression par la route, **et suppression par `invoices::delete`**.
      Plus **AC10** (ré-apparier une ligne déjà lettrée est refusé, et l'ancien partenaire
      reste apparié), **AC11** (deux lignes d'une autre société sont refusées — test d'IDOR,
      pas de confort) et **AC12** (montants inégaux refusés **à la route**, pas seulement à
      l'écran).
      ⛔ **Le test qui manquait, et qui aurait attrapé le défaut de la passe 2** : modifier
      **le seul libellé** d'une écriture lettrée, puis vérifier que **la marque est toujours
      là, des deux côtés**. Sans lui, la clause (ii) d'AC7 n'est pas couverte — et c'est
      exactement le chemin par lequel la marque se perdait.
- [ ] **T7** — i18n : les clés des messages de refus dans les **quatre** locales dès
      l'écriture. **Un message par refus, énuméré par critère** — et non un total, qui se
      périme au premier critère ajouté *(P3-4 : la rédaction précédente en annonçait trois,
      il en manquait quatre)* : compte différent **et** sens non opposés (AC4 — deux causes
      distinctes), délettrage sur exercice clos (AC6), lignes modifiées sur écriture lettrée
      (AC7), suppression d'écriture lettrée (AC8), ligne déjà lettrée (AC10), lignes d'une
      montants inégaux (AC12). **SEPT messages**, et la ventilation se recompte : AC4 en
      porte **deux** — *« pas le même compte »* et *« les deux lignes vont dans le même
      sens »* sont des causes distinctes *(P5-4)* —, plus AC6, AC7, AC8, AC10, AC12.
      ⛔ **AC11 n'a PAS de message** *(P6-4)* : son refus est un 404 indiscernable de « ligne
      inconnue », et lui donner une clé rouvrirait l'oracle d'existence. *(Le total annoncé
      était huit ; il se recompte depuis sa ventilation, § *Recompter ses propres comptes
      rendus*.)*
      ⛔ **La garde i18n ne rattrapera pas l'oubli.** Son allowlist est bien vide
      (`frontend/src/lib/shared/i18n-keys.test.ts:432`), mais elle vérifie qu'une clé
      **existante** figure dans les quatre catalogues — **pas qu'une clé jamais écrite
      manque**. Un refus sans clé remonte en erreur générique, et l'utilisateur ne peut pas en
      déduire la cause : le défaut même qu'AC8 et AC10 prennent soin de nommer.

## Dev Notes

⚠️ **Gate `kesh-db` : complet, jamais ciblé.** Cette story touche une migration et un
repository — les garde-fous P6 et P7 l'imposent, et le précédent de la Story 16-1a (un test
devenu **muet**, passant à vide) dit pourquoi.

⚠️ **La base de gate se remet à zéro AVANT le gate**, inconditionnellement — sans se demander
comment le run précédent s'est terminé (KF-039, #310).

⚠️ **Une migration appliquée ne se modifie plus, pas même un commentaire** (P8) : le
checksum est enregistré, et le binaire ne boote plus.

## Change Log

### Passe 7 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**0 CRITICAL, 1 HIGH, 2 MEDIUM, 1 LOW.**

⛔ **P7-1 (HIGH) — la même famille de défaut que la passe 6 venait de fermer, mais sur AC6 :
la décision comptable centrale de la story.** La passe 6 avait spécifié la concurrence de la
**création** du code (AC3) et laissé le **délettrage** sans mécanisme.

⚠️ **Cette story est la PREMIÈRE du dépôt à devoir vérifier DEUX exercices dans une seule
transaction.** Tout le code existant n'en verrouille qu'un — une écriture n'appartient qu'à un
exercice — et le lettrage à cheval qu'AC5 organise brise cette hypothèse. Les deux issues
qu'un développeur aurait prises sont **toutes deux mauvaises** : verrouiller les deux
`fiscal_years` dans l'ordre d'arrivée → **ABBA** entre deux délettrages croisés, avec un 1213
qui remonte tel quel puisque `retry_on_deadlock` n'a aucun site d'appel ; ne pas verrouiller
→ le contrôle « les deux exercices sont ouverts » n'est plus sérialisé avec
`fiscal_years::close`, et **D3 est violée**.

→ AC6 prend le **même sentinel** que le lettrage. Il sérialise tout le trafic d'une société,
ce qui rend l'ordre des verrous suivants sans importance — idiome écrit du dépôt,
`docs/MULTI-TENANT-SCOPING-PATTERNS.md` **Pattern 5**, que la passe 6 avait cité **pour AC3
sans en tirer la conséquence pour AC6**.

**P7-2 (MEDIUM) — l'oracle qu'AC11 venait de fermer pouvait rouvrir par un autre canal.** La
spec disait *que* le refus cross-tenant est un 404 indiscernable, jamais *quand* ce contrôle
s'exécute. Charger les données de la ligne avant de vérifier l'appartenance rendrait un **409
distinct** — « comptes différents », « déjà lettrée » — révélant l'existence **et un
attribut**. Pire que l'oracle fermé une passe plus tôt. → T4 impose la requête unique scopée
et le 404 **avant** toute autre garde, comme le précédent `products.rs:343` l'écrit.

**P7-4 (MEDIUM) — la garde CSV pouvait rester verte sans jamais exporter la marque.** La
garde-modèle des `invoices` est **structurelle** : elle compare le header aux champs de la
**struct**, pas au schéma. Exposer `lettering_id` par une lecture dédiée, sans toucher le type
partagé, l'aurait laissée verte tout en n'exportant rien. → **Tranché** : `lettering_id` entre
dans `JournalEntryLine`, `LINE_COLUMNS` et l'export dès cette story — l'export existe pour
que l'utilisateur **vérifie ce que Kesh affirme**, l'objet même de la story.

**P7-3 (LOW) — le jumeau resté dans `epics.md`.** La passe 6 avait corrigé la ligne 1368
(colonne `lettering_code` écartée) et laissé la 1363 — « deux **ou plusieurs** écritures » —
que D2 contredit, **dans le même paragraphe**. C'est mot pour mot le mode d'échec de la
§ *Propagation post-patch* : corriger un site et laisser son jumeau. Corrigé.

**Vérifié et réfuté** : la croissance non bornée de `letterings` est cohérente avec
`audit_log` (append-only assumé) ; aucune entrée orpheline possible après rollback, tout étant
dans la même transaction ; l'ordre dans `reset_demo` est indifférent (`FOREIGN_KEY_CHECKS=0`),
contrairement à `TABLES_TO_TRUNCATE` ; les mentions résiduelles dans le fichier de kickoff
sont un **instantané antérieur** aux décisions, pas un artefact vivant ; et **le décompte de
sept messages est exact**, recompté depuis la ventilation — 2+1+1+1+1+1.

**Verdict : passe 8 due — la dernière avant le plafond de budget.**

### Passe 6 de `validate` — 2026-08-25 (Opus, contexte frais)

**0 CRITICAL, 1 HIGH, 5 MEDIUM, 3 LOW.** La sévérité décroît (`CRIT → HIGH`) : le critère de
re-split n'est pas déclenché.

⛔ **P6-1 (HIGH) — le correctif de la passe 5 reproduisait, sur un autre plan, la faute qu'il
corrigeait.** Il avait repris de la formule d'`entry_number` **ce qui se voit** — le
`MAX(…) FOR UPDATE` — et laissé **ce qui la fait fonctionner** : le verrou de ligne pris en
amont (`SELECT fiscal_years … FOR UPDATE`, `journal_entries.rs:191`).

`letterings` n'a **aucune ligne préexistante à verrouiller** pour une société encore vierge
de lettrage — et c'est la fixture naturelle du test de concurrence qu'AC3 exige. Deux
transactions y prennent des gap locks **compatibles**, calculent le même `seq`, puis
deadlockent à l'`INSERT`. ⚠️ **Le dépôt porte déjà ce diagnostic écrit**, à propos du même
geste : `email_templates.rs:16-23` — *« un gap lock n'empêche PAS une autre transaction de
tenir aussi son propre gap lock compatible sur la même lacune […] risquent donc de
deadlocker »*.

Et le filet annoncé ne rattrape rien : **un deadlock est un 1213, pas un 1062**. Aucune
contrainte d'unicité ne le voit. Le dépôt a un `retry_on_deadlock` — vérifié : **aucun site
d'appel**, la parade existe et n'est branchée nulle part.

→ AC3 nomme désormais le mécanisme **en deux temps** : sentinel `companies FOR UPDATE`
(Pattern 5, idiome du dépôt), **puis** `MAX(seq)`. Plus l'ordre de verrou global
(`companies → projects → fiscal_years`) et **ce que le test doit constater** — sans quoi son
auteur, voyant un deadlock intermittent, aurait affaibli l'assertion.

**Cinq MEDIUM, dont trois oublis de propagation que la story elle-même prétendait éviter :**

| | défaut | remède |
|---|---|---|
| **P6-2** | *« Le `.keshbackup` n'est pas concerné »* — **vrai pour les colonnes, faux pour les tables** : `TABLES_TO_TRUNCATE` est **codé en dur** et ordonné enfants → parents | T5 : inscrire `letterings` à sa place FK-correcte ; le test compare des listes **triées** et ne verrait pas une position fausse ; et **tout backup antérieur devient non importable** — à dire |
| **P6-3** | **le squash de schéma de test n'était pas mentionné** — T1 anticipait P5, P6, P7 et le compteur d'`upgrade_path`, et oubliait le geste le plus mécanique. Sans `regen-test-schema.sh`, `letterings` n'existe dans **aucune** des ~1100 bases `#[sqlx::test]` | ligne ajoutée à T1 |
| **P6-4** | **le message de refus d'AC11 contredisait la garde anti-IDOR** qu'AC11 pose : nommer « lignes d'une autre société » rend un **oracle d'existence**. La convention du dépôt est écrite dans le code — *« indiscernable d'un compte inexistant »*, 404 | AC11 : verdict indiscernable, **aucune clé** ; T7 repasse de huit à **sept** messages |
| **P6-5** | **délettrer supprimait l'entrée, donc `MAX(seq)` reculait et le code était RÉÉMIS** — un `F` imprimé ou dicté reviendrait sur une paire sans rapport, et l'audit porterait deux lettrages sous une même désignation | `letterings` est **append-only** : délettrer met les `lettering_id` à `NULL` et laisse l'entrée |
| **P6-6** | **D2 n'était nommée par aucune des trois fiches du split** — septième récidive du geste. Et `epics.md:1368` prescrivait encore la colonne `lettering_code` écartée | D2 héritée et citée par AC12 ; `epics.md` corrigé |

**Trois LOW** : l'index se déclare par un `CREATE INDEX` séparé, conforme au seul précédent
d'`ALTER TABLE journal_entry_lines` du dépôt ; deux `CHECK` ajoutés (`seq > 0`, code non
vide) comme les tables voisines en portent ; le commentaire de `code` ne dit plus « saisie »,
`utf8mb4_bin` étant octet-exact. Et **P6-7 : la collision est à la vingt-huitième lettre, pas
la vingt-septième** — après `{A…Z}` le successeur `AA` est juste ; c'est après `{A…Z, AA}` que
`MAX` rend encore `Z`. Le diagnostic de la passe 5 restait entier, seul son ordinal était
décalé.

**Vérifié et réfuté — dont ce qui rassure sur l'arbitrage** : les six cas limites de la
conversion sont **exacts, recalculés** (`26→Z`, `27→AA`, `52→AZ`, `53→BA`, `702→ZZ`,
`703→AAA`) ; **`VARCHAR(16)` couvre tout le domaine `BIGINT`** — `seq = 2⁶³−1` donne
`CRPXNLSKVLJFHG`, quatorze caractères ; le `RESTRICT` sur `lettering_id` ne bloque rien
qu'AC8 attende en succès ; `created_by NOT NULL … RESTRICT` ne casse aucun chemin existant ;
et les compteurs d'idempotence sont sains — **61 partout**, T1 n'hérite rien à réparer.

**Verdict : passe 7 due.**

### Passe 5 — contrôle d'arbitrage — 2026-08-25 (Haiku, contexte frais)

⛔ **1 CRITICAL, 1 HIGH, 2 MEDIUM, 1 LOW.** La passe était due parce que l'arbitrage fixait
une règle structurante **après** la convergence de la passe 4. Elle a bien fait de l'être.

**P5-1 (CRITICAL) — le compteur aurait été FAUX à la vingt-septième lettre.** Le § *Décisions
tranchées* affirmait que le gap lock d'`entry_number` « se transpose à l'identique ». Il ne se
transpose pas : `entry_number` est un **entier**, `code` est du **texte**, et `MAX()` sur du
texte trie **lexicographiquement**. Vérifié : `MAX('A','B','Z','AA','AB','AZ','BA')` rend
**`Z`**, quand la séquence bijective attend `BA`.

⚠️ **Le défaut n'aurait pas été muet — il aurait été bruyant et tardif** : après `Z`, le
compteur serait reparti sur des valeurs déjà prises, et `uq_letterings_company_code` aurait
fait échouer **tout lettrage** de la société. En développement, avec une société de
démonstration à moins de vingt-six lettrages, **rien ne l'aurait révélé**.

**Le correctif ajoute une colonne `seq BIGINT`** : le gap lock porte sur elle — transposition
**réellement** exacte du pattern éprouvé —, et `code` en est la **projection** base 26
bijective, calculée en Rust. ⚠️ **La conversion est une fonction pure : elle se teste dans
`kesh-core`, sans base**, et T1 nomme ses cas limites (`26 → Z`, `27 → AA`, `53 → BA`,
`702 → ZZ`, `703 → AAA`). Les deux contraintes d'unicité, sur `seq` et sur `code`, tiennent
le filet.

**P5-2 (HIGH) — le DDL était un fragment.** Il portait un `...` et laissait au développeur les
FK, les `ON DELETE`, le `CHARACTER SET`/`COLLATE`, les `COMMENT` et l'`ENGINE` — alors que les
migrations du dépôt les déclarent toutes explicitement (comparé à
`20260814000001_contacts_client_number_canonical.sql`). Le DDL est désormais **entier et à
reprendre tel quel**.

**P5-3 (MEDIUM) — les comportements de FK sont tranchés**, conformes aux conventions relevées
au sol (`companies(id) ON DELETE RESTRICT` partout ; `users(id) RESTRICT` pour un auteur
d'action, cf. `bank_imports.imported_by_user_id`). ⚠️ Le choix de **`RESTRICT`** sur
`journal_entry_lines.lettering_id` impose l'ordre du délettrage — mettre les lignes à `NULL`
**puis** supprimer l'entrée — et c'est voulu : un `SET NULL` ferait disparaître une marque des
deux côtés **sans trace**, le défaut muet même qu'AC8 ferme ailleurs.

**P5-4 (MEDIUM)** : T7 énumérait les critères sans dire qu'AC4 porte **deux** causes
distinctes — huit messages, pas sept. **P5-5 (LOW)** : T1 précise que le compteur du test P6
bougera, mais qu'aucun backfill à fenêtre n'est en jeu.

**Ce que cette passe apprend sur le processus** : un arbitrage du Project Lead **n'est pas
exempt de revue**. Celui-ci était juste dans ses deux choix — la table et le format restent
les bons — mais sa **traduction en DDL** portait un défaut critique que ni l'arbitrage ni les
quatre passes précédentes ne pouvaient contenir, puisqu'il n'existait pas encore. Une passe
6 est due.

### Arbitrage du Project Lead — 2026-08-25

Les deux décisions que les passes avaient laissées ouvertes sont **tranchées par Guy**, et
inscrites au § *Décisions tranchées*.

**Le porteur : une table `letterings`.** Elle transpose à l'identique le gap lock éprouvé
d'`entry_number` — sa propre `company_id` le permet —, garde un **filet au niveau du schéma**
(`uq_letterings_company_code`) si le verrouillage faiblit, et fournit `created_at` /
`created_by` dont AC13 a besoin. La colonne sur `journal_entry_lines` n'offrait aucun des
trois : pas de `company_id` à verrouiller, aucune contrainte possible — une contrainte
d'unicité y aurait même **interdit AC1** — et nulle part où porter l'auteur.

**Le format : une séquence alphabétique par société**, `A`, `B`, … `Z`, `AA` — base 26
bijective. C'est le format des logiciels comptables suisses, Bexio compris, dont **D6 fait
déjà le modèle de l'écran** ; il est court, lisible à l'œil sur une ligne, et **se dicte au
téléphone**. Un identifiant sans compteur (ULID, UUID) supprimerait la contention mais 15-1c
exige que la marque soit **visible sur la ligne** : vingt-six caractères y sont
inutilisables.

**Propagé dans le même patch** : AC3 (la génération sous gap lock, le filet de schéma, et un
**test de concurrence désormais exigé** — sans lui le filet reste une intention), T1 (le DDL
est connu, `CREATE TABLE` + FK nullable), T2 (l'ajout de `letterings` à `reset_demo` devient
**obligatoire** — sa liste de `DELETE` est explicite, et le bloc s'exécute sous
`FOREIGN_KEY_CHECKS=0`), et le triage P7 (ni `CREATE TABLE` ni `ADD COLUMN` n'entre dans le
détecteur).

⚠️ **Une passe de contrôle reste due.** Cet arbitrage fixe une règle structurante **après**
la convergence de la passe 4 : la passe ciblée à une lentille codifiée ce matin
(§ A9, PR #355) **ne s'applique pas** — sa borne exclut les patches qui changent une règle
métier.

### Passe 4 de `validate` — 2026-08-25 (Sonnet, contexte frais)

✅ **CONVERGENCE. Un seul finding, de sévérité LOW** — le critère d'arrêt de la § *Review
Iteration Rule* est atteint (« uniquement des findings de sévérité `LOW` »).

**Trajectoire complète** : `2 HIGH` → `2 MED` → `1 CRIT + 2 HIGH` → **`1 LOW`**.

**Ce que la passe a vérifié en premier, et c'était sa raison d'être** : la correction du
CRITICAL par la passe 3 est-elle elle-même juste ? Elle a relu `update` **en entier**
(l. 782-1123) plutôt que les extraits cités, et conclut que **la clause (ii) d'AC7 est
implémentable sans conflit** :

- le `UPDATE` de l'en-tête, le `version + 1` et le snapshot d'audit restent valides —
  `entry_snapshot_json` relit les lignes après coup, et des lignes non touchées se relisent
  correctement, mêmes `id` et même `line_order` ;
- ⚠️ **`is_no_op_change` doit rester ENTIÈRE pour le court-circuit KF-004** ; c'est une
  **seconde** fonction, tirée de sa moitié « lignes », qui pilote la clause (ii). Aucun
  risque de régression sur KF-004 ;
- le comparateur de lignes est **exhaustif** vis-à-vis de `NewJournalEntryLine`, dont les
  champs sont exactement `account_id`, `debit`, `credit`, `project_id` — rien n'y échappe. Un
  changement du **nombre** de lignes est classé « changé » avant toute comparaison
  positionnelle ;
- ⚠️ **aucun chemin ne fait survivre une marque à une modification qui aurait dû la
  détruire.** Toute modification touchant une ligne bascule sous la clause (i).

**Une conséquence assumée, relevée par la passe et vraie** : modifier une ligne **non
lettrée** dans une écriture qui en compte trois dont **une seule** est lettrée est refusé
aussi. C'est la conséquence directe du mode d'écriture par `DELETE`/`INSERT` global — pas un
défaut, mais une restriction à connaître.

**P4-1 (LOW)** — la garde de clôture était citée `journal_entries.rs:892` ; le
`return Err(DbError::FiscalYearClosed)` est en **894**, le test en 892. La citation porte
désormais la garde entière, `892-894`. **Toutes les autres citations de la passe 3 sont
vérifiées exactes** (`:981`, `:782`, `:1226`, `:232`, `invoices.rs:1338`, `:1235`).

**Contrôles et réfutations propres à cette passe** : les deux seuls appelants de
`delete_in_tx` dans tout le workspace sont confirmés (la route et `invoices::delete`) ; AC12
est sans ambiguïté de devise ni d'arrondi — pas de colonne de devise, `Decimal` exact, débit
et crédit mutuellement exclusifs par contrainte ; AC13 se conforme au motif `entité.verbe` du
dépôt, et la question de l'`entity_id` d'un événement portant sur **deux** lignes **a été
posée puis refermée** par D1 elle-même — la marque portant sur la ligne, l'entité d'audit est
la ligne, un événement par ligne, que le porteur soit (A) ou (B). Enfin les compteurs de
`docs/migrations-idempotence-audit.md` ont été recomptés depuis la source : **61 partout**,
aucune dérive préexistante que T1 hériterait.

**Verdict : la spécification est prête pour le développement**, une fois arbitrées les deux
décisions réservées au Project Lead — le **porteur** de la marque (A ou B) et son **format**.

### Passe 3 de `validate` — 2026-08-25 (Opus, contexte frais)

⛔ **1 CRITICAL, 2 HIGH, 4 MEDIUM, 2 LOW. Trajectoire : `HIGH → MEDIUM → CRITICAL`.**

⚠️ **Le critère de non-convergence est déclenché une seconde fois — mais il ne commande PAS
un re-split ici, et la raison importe.** 15-1a est **étroite** : ce n'est pas un défaut de
largeur. C'est que **la passe 2 a tranché une règle métier sur un fait qu'elle n'avait pas
vérifié**, et que la conduite retenue rouvrait le trou que la story existe pour fermer. Le
remède est ciblé.

**P3-1 (CRITICAL) — la conduite tranchée en passe 2 détruisait la marque sur le chemin
qu'elle ouvrait.** AC7 autorisait la modification de l'en-tête. Or le `DELETE FROM
journal_entry_lines` de `update` (`journal_entries.rs:981`) n'est gardé que par le
court-circuit **no-op complet** — en-tête **et** lignes identiques. Une modification de
libellé seul le franchit **systématiquement** : les lignes sont effacées et réinsérées sans
marque, la contrepartie garde la sienne. **Mot pour mot le mode d'échec que cette story
existe pour fermer**, réintroduit par le critère censé le fermer.

Et le lien n'avait jamais été fait : l'argument (2) de la passe 2 écartait la préservation de
la marque comme « pas fiable », alors qu'autoriser l'en-tête *l'exigeait* dans ce cas précis.
→ **AC7 clause (ii)** : quand les lignes sont inchangées, `update` **ne les touche pas**. La
condition de la garde est exactement ce qui rend le `DELETE`/`INSERT` inutile — le même patch
ferme le CRITICAL et vide l'objection de son objet.

**P3-2 (HIGH) — le premier des « deux faits au sol » de la passe 2 était FAUX.** Elle
affirmait qu'un refus global rendrait une écriture lettrée sur exercice clos « figée jusqu'à
sa description ». **Elle l'est déjà** : `update` refuse à l'Étape 2
(`journal_entries.rs:892-894`, `DbError::FiscalYearClosed`), lettrage ou non. Le cas réel est
plus étroit — écriture sur exercice **ouvert** lettrée contre une contrepartie sur exercice
**clos**. ⚠️ Un développeur écrivant le test d'AC7 depuis ce texte aurait obtenu un 409
inattendu, et aurait pu conclure que **la garde de clôture est l'obstacle et l'assouplir** :
une régression sur l'immuabilité post-clôture introduite par un patch de lettrage.

**P3-3 (HIGH) — le split avait laissé tomber l'égalité des montants.** La garde vivait dans
**15-1c**, la story de l'écran, alors que **la route d'écriture est dans 15-1a**. 15-1a et
15-1b mergées sans 15-1c, un appel direct appariait 1000 avec 300 et la vue retirait la
créance en laissant 700 dus — le finding **F2** de la story mère, coté HIGH deux fois,
survivant au découpage. Le tableau des décisions du split assignait à 15-1c « la **tolérance**
de montant » : la tolérance et l'existence du contrôle sont deux questions, et le split a
déplacé la seconde en croyant ne déplacer que la première. → **AC12**.

**Quatre MEDIUM** : **P3-4** T7 annonçait « trois messages », il en manquait **quatre** — et
la garde i18n ne rattrape pas une clé *jamais écrite* ; **P3-5** le **format** de la marque
n'était fixé nulle part alors que T1 doit écrire le DDL et que P8 interdit d'y revenir — la
story mère nommait pourtant le trou dans ses deux dimensions, « ni format, ni portée », et le
split a **perdu le format** ; **P3-6** la réfutation du wipe `reset_demo` ne vaut que pour la
conduite (A) — une table `letterings` n'est pas dans sa liste explicite de `DELETE` ;
**P3-7** aucune trace d'audit n'était exigée, alors qu'AC5 autorise une écriture sur exercice
**clos**, seul endroit où l'absence de trace est grave. → **AC13**.

**Deux LOW** : chemins de fichier imprécis (P3-8) ; et **P3-9**, qui compte plus que sa
cote — « sa seconde moitié EST la garde » invitait à réutiliser `is_no_op_change` **entière**,
laquelle retourne `false` dès que l'en-tête diffère **sans regarder les lignes**, soit
l'exact contraire de la conduite. S'y ajoute une réserve sur `project_id`, dont le
comparateur n'est exact que parce que les deux sites de la route passent `None`.

**Pistes réfutées, et l'une répondait à une inquiétude explicite** : ⚠️ **changer la DATE ne
peut pas déplacer une écriture lettrée d'exercice** — `update` ne réécrit jamais
`fiscal_year_id`, et confine `entry_date` aux bornes de l'exercice de l'écriture. La garantie
d'AC5/AC6 survit aux modifications d'en-tête. Changer le **journal** est inerte. Les
`DELETE FROM journal_entries` d'`invoices.rs:3866` et l'`INSERT` de `journal_entries.rs:2297`
sont dans des `mod tests`. Le point de passage unique d'AC8 tient. Le lettrage d'une ligne
avec elle-même est déjà fermé par la contrainte `chk_jel_debit_credit_exclusive`. Et **les
patches de la passe 1 tiennent sans régression** — AC10 et AC11 purement additifs, numéros de
ligne exacts.

La spec passe de **11 à 14 critères**. **Verdict : passe 4 due**, modèle différent.

### Passe 2 de `validate` — 2026-08-25 (Haiku, contexte frais)

**0 HIGH, 2 MEDIUM, 1 LOW** — et les trois portent sur **le même point**. La sévérité
décroît (`HIGH → MEDIUM`) : convergence monotone, le critère de split n'est pas déclenché.

**Aucune régression des patches de la passe 1** : AC10 et AC11 sont jugés cohérents et
testables, et le bloc sur l'asymétrie (A)/(B) exact au sol.

⚠️ **Le finding, et il est embarrassant : AC7 disait « à trancher à la spécification et non à
l'implémentation », puis ne tranchait pas.** C'est **mot pour mot** le défaut relevé sur AC12
de la story mère en passe 2 — reproduit par celui-là même qui l'avait relevé, dans la story
née de ce relevé. Les deux MEDIUM et le LOW en découlent en cascade : le décompte des messages
i18n de T7 était indéterminé, et le sort d'une renumérotation restait ouvert.

**AC7 est tranchée, et deux faits au sol l'ont tranchée — pas une préférence :**

> La modification est **refusée si elle change les LIGNES** d'une écriture dont une ligne est
> lettrée. **L'en-tête reste modifiable.**

**(1) Un refus global figerait l'écriture jusqu'à son libellé.** `update` réécrit
systématiquement toutes les lignes dès que le payload diffère — aucun chemin « en-tête
seul » n'existe. Refuser en bloc obligerait à délettrer pour corriger une faute de frappe,
**or le délettrage est interdit sur exercice clos (AC6)** : une écriture lettrée sur un
exercice clos deviendrait définitivement figée, description comprise. Cet effet de bord
n'avait été vu par aucune des deux conduites que la spec proposait.

**(2) Préserver la marque à travers le `DELETE`/`INSERT` n'est pas fiable** : la réinsertion
numérote par position, rien ne rattache une ligne nouvelle à l'ancienne si l'ordre change ou
si une ligne est insérée. La marque atterrirait sur la mauvaise ligne — **pire qu'une marque
perdue, puisque plausible**.

⚠️ **La conduite retenue n'était dans aucune des deux options offertes** — c'est une
troisième, trouvée en cherchant *pourquoi* les deux premières coûtaient cher. Et elle
n'invente rien : `is_no_op_change` (`journal_entries.rs:782`) compare déjà l'en-tête **puis**
les lignes ; **sa seconde moitié EST la garde demandée**. Elle règle aussi la renumérotation
(P2-3) sans clause dédiée : réordonner *est* un changement de lignes pour un comparateur
positionnel.

La spec reste à **11 critères**. **Verdict : passe 3 due** — deux MEDIUM au rapport.

### Passe 1 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**2 HIGH, 1 MEDIUM, 1 LOW.** Tous vérifiés au sol par l'orchestrateur avant application.

⚠️ **Et un résultat POSITIF, qui vaut d'être écrit** : la passe a **refait le recensement des
chemins d'écriture en indépendant** — `grep -rn "INSERT INTO journal_entry_lines\|DELETE FROM
journal_entry_lines\|UPDATE journal_entry_lines" crates/` — et conclut qu'il est **exhaustif** :
`create`, `update` et le wipe de `reset_demo`, rien d'autre. Elle a aussi vérifié que
`update`/`delete_by_id`/`delete_in_tx` n'ont **aucun appelant** hors les deux routes et
`invoices::delete` — la réconciliation, les avoirs et le règlement fournisseur créent tous des
écritures **nouvelles** via `create_in_tx`, jamais de modification. **Le socle de raisonnement
de cette story est confirmé, pas seulement non réfuté.**

| | défaut | gravité | remède |
|---|---|---|---|
| **PA-1** | **Aucun critère ne refusait de lettrer une ligne DÉJÀ lettrée.** Ré-apparier écrase la marque et laisse l'ancien partenaire **seul avec l'ancienne** — réputé soldé à vie, sans contrepartie, et rien ne le signale. AC4 laissait passer le cas, compte et sens étant corrects | **HIGH** | **AC10** |
| **PA-2** | **(A) et (B) n'offrent pas la même garantie sur AC3**, et la spec ne le disait pas. Le seul pattern éprouvé du dépôt (gap lock d'`entry_number`, `journal_entries.rs:232`) verrouille une `company_id` que `journal_entry_lines` **n'a pas** : (B) le transpose et garde un filet `UNIQUE`, (A) doit verrouiller par jointure **sans aucun filet de schéma** | **HIGH** | l'asymétrie est nommée au § *Décision à trancher* — c'est un **input d'arbitrage**, pas un blocage |
| **PA-3** | **Aucun critère n'exigeait que les deux lignes appartiennent à la société de l'APPELANT.** « Même compte » les lie entre elles, pas à l'appelant : c'est un **IDOR**, et le dépôt en a déjà payé un (KF-002) | MEDIUM | **AC11** |
| **PA-4** | T1 citait P1, P5 et P7 mais **pas P6** | LOW | le grep de recensement ajouté à T1 |

⚠️ **PA-1 est le finding qui compte, et son intérêt dépasse la story** : les passes
précédentes avaient fermé les chemins par lesquels un **autre** dispositif détruit la marque
(modification, suppression). Celui-ci est une porte que **le lettrage ouvre lui-même** — le
défaut muet vient de la fonctionnalité, pas de son environnement. Aucune des quatre passes de
la story mère ne l'avait vu.

**Pistes réfutées** : le wipe de `reset_demo` (il efface tout ensemble, cohérent) ; le
`.keshbackup` (ses colonnes viennent d'`information_schema`) ; l'inutilité de l'exemption P7
(confirmée — `writes_data()` ne matche que le premier mot-clé, `ADD COLUMN` n'y entre jamais) ;
`fiscal_years::close`/`reopen`, lus en entier, ne touchent **jamais** `journal_entry_lines`.
Et **aucune troisième conduite** ne manque au § *Décision à trancher* : les deux présentées
sont faisables, un `UPDATE`-par-id supposerait de faire porter un `id` par
`NewJournalEntryLine`, changement de contrat qui déborde le socle.

La spec passe de **9 à 11 critères**. **Verdict : passe 2 due** — deux HIGH et un MEDIUM au
rapport.

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**, décidé par le Project Lead après que la passe 3 de
`validate` eut déclenché le critère de non-convergence (`MEDIUM → HIGH`). Cette story
recueille les deux HIGH de cycle de vie (**P3-1**, **P3-2**), le MEDIUM d'unicité
(**P3-5**) et deux LOW (**P3-9** export CSV, **P3-10** exemption P7 inutile).

⚠️ **Une décision reste ouverte et bloque le développement** : le porteur de la marque
(colonne ou table `letterings`). Elle est posée au § *Décision à trancher*, avec les deux
conduites et ce que chacune coûte.
