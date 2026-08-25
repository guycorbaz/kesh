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

## Décision à trancher AVANT le développement

⛔ **Le porteur de la marque n'est pas décidé, et ce n'est pas au développeur de le
faire.** La 15-1 prescrivait une colonne `lettering_code` sur `journal_entry_lines`. La
passe 3 a montré que le moyen d'unicité qu'on lui associait est **impossible** :

- `journal_entry_lines` **ne porte aucun `company_id`** — zéro occurrence dans les
  migrations ; le scoping multi-tenant y passe par jointure sur `journal_entries.company_id`,
  et le repository le documente lui-même (`journal_entries.rs:1226`) ;
- une contrainte `UNIQUE (company_id, lettering_code)` **interdirait ce que le lettrage
  exige** — deux lignes lettrées ensemble portent le *même* code.

**Deux conduites réalisables, à arbitrer :**

**(A) Colonne `lettering_code` nullable** sur `journal_entry_lines`, indexée. Migration
`ADD COLUMN` **non-breaking** (P1 : pas de bump `min_required`). L'unicité du code à
l'engendrement se tient alors **applicativement**, sous transaction.

**(B) Table `letterings`** (`id`, `company_id`, `code`, `created_at`, `created_by`) avec
`UNIQUE (company_id, code)`, que les lignes référencent par une FK nullable. Plus coûteuse
d'une table, elle donne **l'unicité par le schéma**, la portée société **sans jointure
supplémentaire**, et un point d'accroche pour la date de lettrage et son auteur — que
(A) n'a nulle part où mettre.

⚠️ **(A) et (B) n'offrent PAS la même garantie sur AC3, et c'est l'élément de poids de
l'arbitrage** *(relevé en passe 1)*. Le dépôt a **un** pattern éprouvé de génération scopée
sous concurrence — le gap lock d'`entry_number` :

```sql
SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries
 WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE   -- journal_entries.rs:232
```

Il verrouille **directement** une colonne `company_id` de la table cible.

- **(B) le transpose à l'identique** — `letterings` a sa propre `company_id` — **et** garde
  un **filet au niveau du schéma** : une violation résiduelle est rattrapée par
  `UNIQUE (company_id, code)`.
- **(A) n'a aucune colonne à verrouiller** : `journal_entry_lines` ne porte pas de
  `company_id`, il faudrait verrouiller **par jointure**, mécanisme jamais éprouvé ici pour
  cet usage. Et **aucune contrainte de schéma n'est possible** — la spec l'établit plus haut.

⛔ **Conséquence : dans (A), une erreur de verrouillage n'est détectée par RIEN.** Elle
produit deux paires de lignes partageant la même marque, en silence — précisément le mode
d'échec qu'AC3 existe pour empêcher. Ce n'est pas un argument décisif contre (A), mais le
Project Lead doit l'avoir en main.

⛔ **Et une SECONDE décision, distincte de la première : le FORMAT de la marque.**
*(Relevé en passe 3 — la story mère nommait le trou dans ses **deux** dimensions, « ni format,
ni portée » ; le split a conservé la portée et **perdu le format**.)* Compteur numérique,
séquence alphabétique `A, B, … AA` à la manière de Bexio, longueur, type de colonne : rien
n'est fixé, et T1 doit pourtant écrire le DDL.

⚠️ **Elle se fige à la migration** : P8 interdit de retoucher un fichier déjà appliqué. Et
elle se voit à l'écran — 15-1c exige que la marque soit **visible** sur la ligne, or un
`BIGINT` global et un `VARCHAR(8)` par société ne donnent pas la même interface. C'est
exactement la classe de décision que ce paragraphe existe pour ne pas laisser au développeur.

⚠️ **Un élément d'arbitrage qui vaut pour les deux décisions** : l'option (B) fournit
**gratuitement** `created_at` et `created_by`, dont AC13 a besoin. L'option (A) n'a nulle
part où les mettre.

⚠️ **Ce choix conditionne les critères ci-dessous**, qui sont écrits pour rester vrais dans
les deux cas : ils parlent de « la marque » et non d'une colonne.

## Décisions héritées de la 15-1

### D1 — La marque porte sur la LIGNE, pas sur l'écriture

⚠️ **Pour une raison de fond : le lettrage porte sur un COMPTE.** Une écriture de vente
touche le compte client, un compte de produit et un compte de TVA. « Cette écriture est
lettrée » n'a aucun sens comptable — ce qui est soldé, c'est la **ligne au compte client**.
Une marque posée sur l'écriture rendrait la vue « ce qui reste ouvert **sur le compte
1100** » incalculable sans retrouver la ligne concernée.

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
société ne reçoivent jamais la même marque. Un compteur lu puis incrémenté hors transaction
ne le garantit pas. ⛔ **Moyen EXCLU** : une contrainte `UNIQUE (company_id, lettering_code)`
sur `journal_entry_lines` — la colonne `company_id` n'y existe pas, et la contrainte
interdirait AC1.

**AC4** — Le lettrage est **refusé** si les deux lignes ne portent pas sur le **même
compte**, ou si leurs sens (débit/crédit) ne s'opposent pas.

**AC5** (porte **D3**) — Le lettrage est **autorisé** même si l'un des exercices concernés
est clôturé, y compris à cheval sur deux exercices.

**AC6** (porte **D3**) — Le délettrage est possible tant que **les deux** exercices sont
ouverts, et **refusé** dès que l'un est clôturé.

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
`validated` **avec son écriture**, par `journal_entries::delete_in_tx` (`invoices.rs:1338`),
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

**AC12** — ⚠️ **Le lettrage est REFUSÉ si les deux montants ne sont pas égaux**, et le
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
(`journal_entries.rs:1226`). **C'est un IDOR**, et le dépôt en a déjà payé un (KF-002) — le
même précédent qu'AC2 invoque pour la portée du compteur.

## Tasks

- [ ] **T1** — Migration selon le porteur arbitré (A ou B). ⚠️ Dans les deux cas, la
      migration **n'écrit aucune donnée** : `ADD COLUMN` nullable ou `CREATE TABLE`, donc
      **non-breaking** — pas de bump `min_required` (P1). Ligne d'audit d'idempotence
      **obligatoire** (P5, `docs/migrations-idempotence-audit.md`), et les **deux** sites du
      total plus les trois compteurs de partition se **recomptent depuis le tableau**.
      ⛔ **Ne PAS l'inscrire à `EXEMPT_MIGRATIONS` (P7)** : *(relevé en passe 3, P3-10)* le
      détecteur ne trie **que** les migrations qui écrivent des données
      (`post_restore.rs:711`) — un `ADD COLUMN` n'est jamais atteint. L'y inscrire ajouterait
      du bruit à une liste dont toute la valeur tient à sa lisibilité.
      ⚠️ **P6 — le couplage positionnel** : lancer
      `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspecter chaque
      site. Le filet est *fail-loud* — `migrations_upgrade_path.rs` porte un
      `assert_eq!(total, …)` codé en dur dont le message renvoie au garde-fou P6 — mais
      **l'anticiper coûte une minute, le découvrir au bout du gate en coûte soixante**.
- [ ] **T2** — Repository : poser la marque, la retirer, la lire. Gardes d'exercice
      asymétriques (AC5/AC6). **Trace d'audit** (AC13).
      ⚠️ **Si la conduite (B) est retenue, ajouter `letterings` à `reset_demo`** *(P3-6)* :
      sa liste de `DELETE` est **explicite** (`kesh-seed/src/lib.rs`) et une table neuve n'y
      figurerait pas. Le bloc s'exécute sous `SET FOREIGN_KEY_CHECKS=0`, si bien que le
      `DELETE FROM companies` passerait **malgré la FK** et laisserait des lignes orphelines ;
      et le jour où ce drapeau serait retiré — le fichier dit que les `DELETE` explicites
      existent pour cela — `reset_demo` échouerait. *(La réfutation de passe 1, « le wipe
      efface tout ensemble », ne vaut que pour la conduite (A).)*
- [ ] **T3** — ⚠️ **Gardes sur les chemins d'écriture existants** (AC7, AC8) — c'est le cœur
      de cette story. Recenser les appelants avant d'écrire :
      `grep -rn "delete_in_tx\|DELETE FROM journal_entry_lines" crates/`.
      ⚠️ **Pour AC7, extraire la moitié « lignes » de `is_no_op_change`
      (`journal_entries.rs:782`) plutôt que d'écrire un second comparateur** — deux
      comparateurs divergents donneraient deux réponses à la question « les lignes ont-elles
      changé ? ». La garde de AC8 se pose dans `delete_in_tx`, **pas** dans le handler.
- [ ] **T4** — Routes : `POST` lettrage, `DELETE` délettrage.
- [ ] **T5** — ⚠️ **Export CSV** *(relevé en passe 3 de la story MÈRE)* : le header de
      `journal_entry_lines` est **figé en dur**
      (`crates/kesh-api/src/exports/csv_tables.rs:286`) et il n'a **aucune garde
      d'exhaustivité**, contrairement à `invoices` (même fichier, l. 1031, garde #262). S'y ajoutent **deux** listes de colonnes en dur côté repository
      (`LINE_COLUMNS` l. 44 et le `SELECT` l. 1235) : en oublier une fait échouer l'export au
      runtime. Étendre la garde vaut mieux que se souvenir.
      *(Le `.keshbackup` n'est pas concerné : ses colonnes viennent d'`information_schema`.)*
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
      autre société (AC11), montants inégaux (AC12).
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
