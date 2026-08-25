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

Deux conduites acceptables, à trancher à la spécification et **non à l'implémentation** :
refuser la modification d'une écriture dont une ligne est lettrée (cohérent avec AC8), ou
préserver explicitement la marque à travers le cycle `DELETE`/`INSERT`. ⚠️ **La seconde ne
va pas de soi** : la réécriture se fait par `line_order`, qui ne rattache pas une ligne
nouvelle à l'ancienne si l'ordre a changé.

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
      asymétriques (AC5/AC6).
- [ ] **T3** — ⚠️ **Gardes sur les chemins d'écriture existants** (AC7, AC8) — c'est le cœur
      de cette story. Recenser les appelants avant d'écrire :
      `grep -rn "delete_in_tx\|DELETE FROM journal_entry_lines" crates/`.
- [ ] **T4** — Routes : `POST` lettrage, `DELETE` délettrage.
- [ ] **T5** — ⚠️ **Export CSV** *(relevé en passe 3, P3-9)* : le header de
      `journal_entry_lines` est **figé en dur** (`exports/csv_tables.rs:286`) et il n'a
      **aucune garde d'exhaustivité**, contrairement à `invoices` (`csv_tables.rs:1031`,
      garde #262). S'y ajoutent **deux** listes de colonnes en dur côté repository
      (`LINE_COLUMNS` l. 44 et le `SELECT` l. 1235) : en oublier une fait échouer l'export au
      runtime. Étendre la garde vaut mieux que se souvenir.
      *(Le `.keshbackup` n'est pas concerné : ses colonnes viennent d'`information_schema`.)*
- [ ] **T6** — Tests : **AC7, AC8 et AC10 en priorité** — ce sont les trois endroits où une
      implémentation plausible produit un défaut **muet**. Un test par chemin : modification
      d'écriture lettrée, suppression par la route, **et suppression par `invoices::delete`**.
      Plus **AC10** (ré-apparier une ligne déjà lettrée est refusé, et l'ancien partenaire
      reste apparié) et **AC11** (deux lignes d'une autre société sont refusées — test
      d'IDOR, pas de confort).
- [ ] **T7** — i18n : les clés des messages de refus dans les **quatre** locales dès
      l'écriture. L'allowlist de la garde i18n est **vide** (`i18n-keys.test.ts:432`) — une
      clé manquante rougit au gate.

## Dev Notes

⚠️ **Gate `kesh-db` : complet, jamais ciblé.** Cette story touche une migration et un
repository — les garde-fous P6 et P7 l'imposent, et le précédent de la Story 16-1a (un test
devenu **muet**, passant à vide) dit pourquoi.

⚠️ **La base de gate se remet à zéro AVANT le gate**, inconditionnellement — sans se demander
comment le run précédent s'est terminé (KF-039, #310).

⚠️ **Une migration appliquée ne se modifie plus, pas même un commentaire** (P8) : le
checksum est enregistré, et le binaire ne boote plus.

## Change Log

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
